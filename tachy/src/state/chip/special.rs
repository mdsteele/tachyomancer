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

use super::super::eval::{ChipEval, CircuitInteraction, CircuitState};
use super::data::{AbstractConstraint, ChipData};
use crate::geom::{Coords, Direction};
use crate::state::{PortColor, PortFlow, WireSize};
use rand;
use std::cell::RefCell;
use std::rc::Rc;

//===========================================================================//

pub const BREAK_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[AbstractConstraint::Equal(0, 1)],
    dependencies: &[(0, 1)],
};

pub struct BreakChipEval {
    input: usize,
    output: usize,
    coords: Coords,
}

impl BreakChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
        coords: Coords,
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), BREAK_CHIP_DATA.ports.len());
        let chip_eval =
            BreakChipEval { input: slots[0].0, output: slots[1].0, coords };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for BreakChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input) {
            state.send_event(self.output, value);
            state.breakpoint(self.coords);
        }
    }
}

//===========================================================================//

pub const BUTTON_CHIP_DATA: &ChipData = &ChipData {
    ports: &[(PortFlow::Send, PortColor::Event, (0, 0), Direction::East)],
    constraints: &[AbstractConstraint::Exact(0, WireSize::Zero)],
    dependencies: &[],
};

pub struct ButtonChipEval {
    output: usize,
    coords: Coords,
    press_count: i32,
    interact: Rc<RefCell<CircuitInteraction>>,
}

impl ButtonChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
        coords: Coords,
        interact: Rc<RefCell<CircuitInteraction>>,
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), BUTTON_CHIP_DATA.ports.len());
        let chip_eval = ButtonChipEval {
            output: slots[0].0,
            coords,
            press_count: 0,
            interact,
        };
        vec![(0, Box::new(chip_eval))]
    }
}

impl ChipEval for ButtonChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(count) =
            self.interact.borrow_mut().buttons.remove(&self.coords)
        {
            debug_log!(
                "Button at ({}, {}) was pressed {} time(s)",
                self.coords.x,
                self.coords.y,
                count
            );
            self.press_count += count;
        }
        if self.press_count > 0 {
            self.press_count -= 1;
            state.send_event(self.output, 0);
        }
    }

    fn needs_another_cycle(&mut self, _state: &CircuitState) -> bool {
        self.press_count > 0
    }
}

//===========================================================================//

pub const COMMENT_CHIP_DATA: &ChipData =
    &ChipData { ports: &[], constraints: &[], dependencies: &[] };

//===========================================================================//

pub const DISPLAY_CHIP_DATA: &ChipData = &ChipData {
    ports: &[(PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West)],
    constraints: &[],
    dependencies: &[],
};

//===========================================================================//

pub const RAM_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::North),
        (PortFlow::Send, PortColor::Behavior, (0, 1), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (1, 1), Direction::East),
        (PortFlow::Recv, PortColor::Event, (1, 1), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (1, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::AtMost(0, WireSize::Eight),
        AbstractConstraint::AtMost(3, WireSize::Eight),
        AbstractConstraint::AtLeast(1, WireSize::One),
        AbstractConstraint::AtLeast(4, WireSize::One),
        AbstractConstraint::Equal(0, 3),
        AbstractConstraint::Equal(1, 2),
        AbstractConstraint::Equal(1, 4),
        AbstractConstraint::Equal(1, 5),
        AbstractConstraint::Equal(2, 4),
        AbstractConstraint::Equal(2, 5),
        AbstractConstraint::Equal(4, 5),
    ],
    dependencies: &[
        (0, 2),
        (1, 2),
        (3, 2),
        (4, 2),
        (0, 5),
        (1, 5),
        (3, 5),
        (4, 5),
    ],
};

pub struct RamChipEval {
    input_b1: usize,
    input_e1: usize,
    output1: usize,
    input_b2: usize,
    input_e2: usize,
    output2: usize,
    values: Vec<u32>,
}

impl RamChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), RAM_CHIP_DATA.ports.len());
        let addr_size = slots[0].1;
        let num_addrs = 1usize << addr_size.num_bits();
        let chip_eval = RamChipEval {
            input_b1: slots[0].0,
            input_e1: slots[1].0,
            output1: slots[2].0,
            input_b2: slots[3].0,
            input_e2: slots[4].0,
            output2: slots[5].0,
            values: vec![0u32; num_addrs],
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for RamChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let addr1 = state.recv_behavior(self.input_b1).0 as usize;
        let addr2 = state.recv_behavior(self.input_b2).0 as usize;
        if let Some(value1) = state.recv_event(self.input_e1) {
            self.values[addr1] = value1;
        }
        if let Some(value2) = state.recv_event(self.input_e2) {
            self.values[addr2] = value2;
        }
        state.send_behavior(self.output1, self.values[addr1]);
        state.send_behavior(self.output2, self.values[addr2]);
    }
}

//===========================================================================//

pub const RANDOM_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Zero),
        AbstractConstraint::AtLeast(1, WireSize::One),
    ],
    dependencies: &[(0, 1)],
};

pub struct RandomChipEval {
    input: usize,
    output: usize,
    size: WireSize,
}

impl RandomChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), RANDOM_CHIP_DATA.ports.len());
        let chip_eval = RandomChipEval {
            input: slots[0].0,
            output: slots[1].0,
            size: slots[1].1,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for RandomChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input) {
            let value = rand::random::<u32>() & self.size.mask();
            state.send_event(self.output, value);
        }
    }
}

//===========================================================================//

pub const TOGGLE_CHIP_DATA: &ChipData = &ChipData {
    ports: &[(PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East)],
    constraints: &[AbstractConstraint::Exact(0, WireSize::One)],
    dependencies: &[],
};

pub struct ToggleChipEval {
    output: usize,
    value: bool,
    coords: Coords,
    interact: Rc<RefCell<CircuitInteraction>>,
}

impl ToggleChipEval {
    pub fn new_evals(
        value: bool,
        slots: &[(usize, WireSize)],
        coords: Coords,
        interact: Rc<RefCell<CircuitInteraction>>,
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), TOGGLE_CHIP_DATA.ports.len());
        let chip_eval =
            ToggleChipEval { output: slots[0].0, value, coords, interact };
        vec![(0, Box::new(chip_eval))]
    }
}

impl ChipEval for ToggleChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(count) =
            self.interact.borrow_mut().buttons.remove(&self.coords)
        {
            debug_log!(
                "Toggle at ({}, {}) was pressed {} time(s)",
                self.coords.x,
                self.coords.y,
                count
            );
            if count % 2 != 0 {
                self.value = !self.value;
            }
        }
        state.send_behavior(self.output, self.value.into());
    }
}

//===========================================================================//
