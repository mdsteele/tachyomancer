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
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow, WireId};

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
    input: WireId,
    output: WireId,
    received: bool,
    should_send: bool,
}

impl ClockChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
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
        if self.should_send {
            state.send_event(self.output, 0);
            self.should_send = false;
        }
    }

    fn needs_another_cycle(&mut self, state: &CircuitState) -> bool {
        if state.has_event(self.input) {
            self.received = true;
        }
        false
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
    input: WireId,
    output: WireId,
    value: Option<u32>,
}

impl DelayChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
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
            state.send_event(self.output, value);
        }
    }

    fn needs_another_cycle(&mut self, state: &CircuitState) -> bool {
        if let Some(value) = state.recv_event(self.input) {
            self.value = Some(value);
            true
        } else {
            false
        }
    }
}

//===========================================================================//

pub const EGG_TIMER_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (1, 0), Direction::North),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Exact(2, WireSize::Zero),
    ],
    dependencies: &[(0, 1), (0, 2)],
};

pub struct EggTimerChipEval {
    set: WireId,
    remain: WireId,
    alarm: WireId,
    time: u32,
    should_send: bool,
}

impl EggTimerChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), EGG_TIMER_CHIP_DATA.ports.len());
        let chip_eval = EggTimerChipEval {
            set: slots[0].0,
            remain: slots[1].0,
            alarm: slots[2].0,
            time: 0,
            should_send: false,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for EggTimerChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.set) {
            self.time = value;
            if value == 0 {
                self.should_send = true;
            }
        }
        state.send_behavior(self.remain, self.time);
        if self.should_send {
            state.send_event(self.alarm, 0);
            self.should_send = false;
        }
    }

    fn on_time_step(&mut self) {
        if self.time > 0 {
            self.time -= 1;
            if self.time == 0 {
                self.should_send = true;
            }
        }
    }
}

//===========================================================================//

pub const STOPWATCH_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::South),
        (PortFlow::Recv, PortColor::Event, (1, 0), Direction::North),
        (PortFlow::Recv, PortColor::Event, (1, 0), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Zero),
        AbstractConstraint::Exact(1, WireSize::Zero),
        AbstractConstraint::Exact(2, WireSize::Zero),
    ],
    dependencies: &[(0, 3), (1, 3), (2, 3)],
};

pub struct StopwatchChipEval {
    start: WireId,
    stop: WireId,
    reset: WireId,
    output: WireId,
    size: WireSize,
    time: u32,
    running: bool,
}

impl StopwatchChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), STOPWATCH_CHIP_DATA.ports.len());
        let chip_eval = StopwatchChipEval {
            start: slots[0].0,
            stop: slots[1].0,
            reset: slots[2].0,
            output: slots[3].0,
            size: slots[3].1,
            time: 0,
            running: false,
        };
        vec![(3, Box::new(chip_eval))]
    }
}

impl ChipEval for StopwatchChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let start = state.has_event(self.start);
        let stop = state.has_event(self.stop);
        if start && !stop {
            self.running = true;
        } else if stop && !start {
            self.running = false;
        }
        if state.has_event(self.reset) {
            self.time = 0;
        }
        state.send_behavior(self.output, self.time);
    }

    fn on_time_step(&mut self) {
        if self.running {
            self.time = self.time.wrapping_add(1) & self.size.mask();
        }
    }
}

//===========================================================================//
