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
use crate::geom::{Coords, Direction};
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
        let addr1 = state.recv_behavior(self.input_b1) as usize;
        let addr2 = state.recv_behavior(self.input_b2) as usize;
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

pub const SCREEN_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (2, 0), Direction::North),
        (PortFlow::Recv, PortColor::Event, (3, 0), Direction::North),
        (PortFlow::Send, PortColor::Behavior, (1, 0), Direction::North),
        (PortFlow::Recv, PortColor::Behavior, (0, 2), Direction::West),
        (PortFlow::Recv, PortColor::Event, (0, 1), Direction::West),
        (PortFlow::Send, PortColor::Behavior, (0, 3), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (2, 4), Direction::South),
        (PortFlow::Recv, PortColor::Event, (1, 4), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (3, 4), Direction::South),
        (PortFlow::Send, PortColor::Event, (4, 2), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Eight),
        AbstractConstraint::Exact(1, WireSize::Eight),
        AbstractConstraint::Exact(2, WireSize::Eight),
        AbstractConstraint::Exact(3, WireSize::Eight),
        AbstractConstraint::Exact(4, WireSize::Eight),
        AbstractConstraint::Exact(5, WireSize::Eight),
        AbstractConstraint::Exact(6, WireSize::Eight),
        AbstractConstraint::Exact(7, WireSize::Eight),
        AbstractConstraint::Exact(8, WireSize::Eight),
        AbstractConstraint::Exact(9, WireSize::Eight),
    ],
    dependencies: &[
        (0, 2),
        (1, 2),
        (3, 2),
        (4, 2),
        (6, 2),
        (7, 2),
        (0, 5),
        (1, 5),
        (3, 5),
        (4, 5),
        (6, 5),
        (7, 5),
        (0, 8),
        (1, 8),
        (3, 8),
        (4, 8),
        (6, 8),
        (7, 8),
        (0, 9),
        (1, 9),
        (3, 9),
        (4, 9),
        (6, 9),
        (7, 9),
    ],
};

pub struct ScreenChipEval {
    coords: Coords,
    input_b1: usize,
    input_e1: usize,
    output1: usize,
    input_b2: usize,
    input_e2: usize,
    output2: usize,
    input_b3: usize,
    input_e3: usize,
    output3: usize,
    touch: usize,
    values: Vec<u8>,
    pressed: Option<u32>,
}

impl ScreenChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
        coords: Coords,
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), SCREEN_CHIP_DATA.ports.len());
        let chip_eval = ScreenChipEval {
            coords,
            input_b1: slots[0].0,
            input_e1: slots[1].0,
            output1: slots[2].0,
            input_b2: slots[3].0,
            input_e2: slots[4].0,
            output2: slots[5].0,
            input_b3: slots[6].0,
            input_e3: slots[7].0,
            output3: slots[8].0,
            touch: slots[9].0,
            values: vec![0u8; 256],
            pressed: None,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for ScreenChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let addr1 = state.recv_behavior(self.input_b1) as usize;
        let addr2 = state.recv_behavior(self.input_b2) as usize;
        let addr3 = state.recv_behavior(self.input_b3) as usize;
        if let Some(value1) = state.recv_event(self.input_e1) {
            self.values[addr1] = value1 as u8;
        }
        if let Some(value2) = state.recv_event(self.input_e2) {
            self.values[addr2] = value2 as u8;
        }
        if let Some(value3) = state.recv_event(self.input_e3) {
            self.values[addr3] = value3 as u8;
        }
        state.send_behavior(self.output1, self.values[addr1] as u32);
        state.send_behavior(self.output2, self.values[addr2] as u32);
        state.send_behavior(self.output3, self.values[addr3] as u32);
        if let Some(value) = self.pressed.take() {
            state.send_event(self.touch, value);
        }
    }

    fn coords(&self) -> Option<Coords> {
        Some(self.coords)
    }

    fn display_data(&self) -> &[u8] {
        &self.values
    }

    fn on_press(&mut self, sublocation: u32) {
        self.pressed = Some(sublocation);
    }
}

//===========================================================================//
