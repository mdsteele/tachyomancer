// +--------------------------------------------------------------------------+
// | Copyright 2018 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of Tachyomancer.                                       |
// |                                                                          |
// | Tachyomancer is free software: you can redistribute it and/or modify it  |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | Tachyomancer is distributed in the hope that it will be useful, but      |
// | WITHOUT ANY WARRANTY; without even the implied warranty of               |
// | MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU        |
// | General Public License for details.                                      |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with Tachyomancer.  If not, see <http://www.gnu.org/licenses/>.          |
// +--------------------------------------------------------------------------+

use super::data::{AbstractConstraint, ChipData};
use super::super::eval::{ChipEval, CircuitState};
use tachy::geom::Direction;
use tachy::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const CLOCK_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Zero),
        AbstractConstraint::Exact(1, WireSize::Zero),
    ],
    dependencies: &[],
};

pub struct ClockChipEval {
    input: usize,
    output: usize,
    received: bool,
    should_send: bool,
}

impl ClockChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
        debug_assert_eq!(slots.len(), CLOCK_CHIP_DATA.ports.len());
        let chip_eval = ClockChipEval {
            input: slots[0].0,
            output: slots[1].0,
            received: false,
            should_send: false,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for ClockChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input) {
            self.received = true;
        }
        if self.should_send {
            state.send_event(self.output, 0);
            self.should_send = false;
        }
    }

    fn on_time_step(&mut self) {
        self.should_send = self.received;
        self.received = false;
    }
}

//===========================================================================//

pub const DELAY_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[AbstractConstraint::Equal(0, 1)],
    dependencies: &[],
};

pub struct DelayChipEval {
    input: usize,
    output: usize,
    value: Option<u32>,
}

impl DelayChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
        debug_assert_eq!(slots.len(), DELAY_CHIP_DATA.ports.len());
        let chip_eval = DelayChipEval {
            input: slots[0].0,
            output: slots[1].0,
            value: None,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for DelayChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = self.value.take() {
            debug_log!("Delay chip is sending value {}", value);
            state.send_event(self.output, value);
        }
    }

    fn needs_another_cycle(&mut self, state: &CircuitState) -> bool {
        if let Some(value) = state.recv_event(self.input) {
            debug_log!("Delay chip is storing value {}", value);
            self.value = Some(value);
            true
        } else {
            false
        }
    }
}

//===========================================================================//

pub const DISCARD_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::AtLeast(0, WireSize::One),
        AbstractConstraint::Exact(1, WireSize::Zero),
    ],
    dependencies: &[(0, 1)],
};

pub struct DiscardChipEval {
    input: usize,
    output: usize,
}

impl DiscardChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
        debug_assert_eq!(slots.len(), DISCARD_CHIP_DATA.ports.len());
        let chip_eval = DiscardChipEval {
            input: slots[0].0,
            output: slots[1].0,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for DiscardChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input) {
            state.send_event(self.output, 0);
        }
    }
}

//===========================================================================//

pub const JOIN_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct JoinChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl JoinChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
        debug_assert_eq!(slots.len(), JOIN_CHIP_DATA.ports.len());
        let chip_eval = JoinChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for JoinChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input1) {
            state.send_event(self.output, value);
        } else if let Some(value) = state.recv_event(self.input2) {
            state.send_event(self.output, value);
        }
    }
}

//===========================================================================//

pub const LATEST_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
    ],
    constraints: &[AbstractConstraint::Equal(0, 1)],
    dependencies: &[(0, 1)],
};

pub struct LatestChipEval {
    input: usize,
    output: usize,
}

impl LatestChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
        debug_assert_eq!(slots.len(), LATEST_CHIP_DATA.ports.len());
        let chip_eval = LatestChipEval {
            input: slots[0].0,
            output: slots[1].0,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for LatestChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input) {
            state.send_behavior(self.output, value);
        }
    }
}

//===========================================================================//

pub const SAMPLE_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Zero),
        AbstractConstraint::Equal(1, 2),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct SampleChipEval {
    input_e: usize,
    input_b: usize,
    output: usize,
}

impl SampleChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
        debug_assert_eq!(slots.len(), SAMPLE_CHIP_DATA.ports.len());
        let chip_eval = SampleChipEval {
            input_e: slots[0].0,
            input_b: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for SampleChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input_e) {
            let (value, _) = state.recv_behavior(self.input_b);
            state.send_event(self.output, value);
        }
    }
}

//===========================================================================//
