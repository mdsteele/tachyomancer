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

use super::super::eval::{ChipEval, CircuitState};
use super::data::{AbstractConstraint, ChipData};
use crate::geom::Direction;
use crate::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const COUNTER_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::South),
        (PortFlow::Recv, PortColor::Event, (1, 0), Direction::North),
        (PortFlow::Recv, PortColor::Event, (1, 0), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 3),
        AbstractConstraint::Exact(1, WireSize::Zero),
        AbstractConstraint::Exact(2, WireSize::Zero),
    ],
    dependencies: &[(0, 3), (1, 3), (2, 3)],
};

pub struct CounterChipEval {
    size: WireSize,
    set: usize,
    inc: usize,
    dec: usize,
    output: usize,
    value: u32,
}

impl CounterChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), COUNTER_CHIP_DATA.ports.len());
        let chip_eval = CounterChipEval {
            size: slots[0].1,
            set: slots[0].0,
            inc: slots[1].0,
            dec: slots[2].0,
            output: slots[3].0,
            value: 0,
        };
        vec![(3, Box::new(chip_eval))]
    }
}

impl ChipEval for CounterChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.set) {
            self.value = value;
        }
        if state.has_event(self.inc) {
            self.value = self.value.wrapping_add(1) & self.size.mask();
        }
        if state.has_event(self.dec) {
            self.value = self.value.wrapping_sub(1) & self.size.mask();
        }
        state.send_behavior(self.output, self.value);
    }
}

//===========================================================================//

pub const DEMUX_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
        AbstractConstraint::Exact(3, WireSize::One),
    ],
    dependencies: &[(0, 1), (0, 2), (3, 1), (3, 2)],
};

pub struct DemuxChipEval {
    input: usize,
    output1: usize,
    output2: usize,
    control: usize,
}

impl DemuxChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), DEMUX_CHIP_DATA.ports.len());
        let chip_eval = DemuxChipEval {
            input: slots[0].0,
            output1: slots[1].0,
            output2: slots[2].0,
            control: slots[3].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for DemuxChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input) {
            if state.recv_behavior(self.control) != 0 {
                state.send_event(self.output1, value);
            } else {
                state.send_event(self.output2, value);
            }
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
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), DISCARD_CHIP_DATA.ports.len());
        let chip_eval =
            DiscardChipEval { input: slots[0].0, output: slots[1].0 };
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

pub const FILTER_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Exact(2, WireSize::One),
    ],
    dependencies: &[(0, 1), (2, 1)],
};

pub struct FilterChipEval {
    input: usize,
    output: usize,
    control: usize,
}

impl FilterChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), FILTER_CHIP_DATA.ports.len());
        let chip_eval = FilterChipEval {
            input: slots[0].0,
            output: slots[1].0,
            control: slots[2].0,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for FilterChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input) {
            if state.recv_behavior(self.control) == 0 {
                state.send_event(self.output, value);
            }
        }
    }
}

//===========================================================================//

pub const INC_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct IncChipEval {
    size: WireSize,
    input1: usize,
    input2: usize,
    output: usize,
}

impl IncChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), INC_CHIP_DATA.ports.len());
        let chip_eval = IncChipEval {
            size: slots[2].1,
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for IncChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(input1) = state.recv_event(self.input1) {
            let input2 = state.recv_behavior(self.input2);
            let output = (input1 + input2) & self.size.mask();
            state.send_event(self.output, output);
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
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
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
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), LATEST_CHIP_DATA.ports.len());
        let chip_eval =
            LatestChipEval { input: slots[0].0, output: slots[1].0 };
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
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
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
            let value = state.recv_behavior(self.input_b);
            state.send_event(self.output, value);
        }
    }
}

//===========================================================================//
