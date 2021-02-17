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

pub const AADD_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Analog, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Analog, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Analog, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::ANALOG),
        AbstractConstraint::Exact(1, WireSize::ANALOG),
        AbstractConstraint::Exact(2, WireSize::ANALOG),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct AAddChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl AAddChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), AADD_CHIP_DATA.ports.len());
        let chip_eval = AAddChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for AAddChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_analog(self.input1);
        let input2 = state.recv_analog(self.input2);
        state.send_analog(self.output, input1 + input2);
    }
}

//===========================================================================//

pub const ADD_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct AddChipEval {
    size: WireSize,
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl AddChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), ADD_CHIP_DATA.ports.len());
        let chip_eval = AddChipEval {
            size: slots[2].1,
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for AddChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.behavior_changed(self.input1)
            || state.behavior_changed(self.input2)
        {
            let input1 = state.recv_behavior(self.input1);
            let input2 = state.recv_behavior(self.input2);
            let sum = input1.wrapping_add(input2) & self.size.mask();
            state.send_behavior(self.output, sum);
        }
    }
}

//===========================================================================//

pub const ADD_2BIT_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Two),
        AbstractConstraint::Exact(1, WireSize::Two),
        AbstractConstraint::Exact(2, WireSize::Two),
        AbstractConstraint::Exact(3, WireSize::Two),
    ],
    dependencies: &[(0, 2), (1, 2), (0, 3), (1, 3)],
};

pub struct Add2BitChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
    carry: WireId,
}

impl Add2BitChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), ADD_2BIT_CHIP_DATA.ports.len());
        let chip_eval = Add2BitChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
            carry: slots[3].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for Add2BitChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.behavior_changed(self.input1)
            || state.behavior_changed(self.input2)
        {
            let input1 = state.recv_behavior(self.input1);
            let input2 = state.recv_behavior(self.input2);
            let sum = input1 + input2;
            let lo = sum & 0b11;
            let hi = (sum >> 2) & 0b11;
            state.send_behavior(self.output, lo);
            state.send_behavior(self.carry, hi);
        }
    }
}

//===========================================================================//

pub const AMUL_CHIP_DATA: &ChipData = AADD_CHIP_DATA;

pub struct AMulChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl AMulChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), AMUL_CHIP_DATA.ports.len());
        let chip_eval = AMulChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for AMulChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_analog(self.input1);
        let input2 = state.recv_analog(self.input2);
        state.send_analog(self.output, input1 * input2);
    }
}

//===========================================================================//

pub const HALVE_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
    ],
    constraints: &[AbstractConstraint::Equal(0, 1)],
    dependencies: &[(0, 1)],
};

pub struct HalveChipEval {
    input: WireId,
    output: WireId,
}

impl HalveChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), HALVE_CHIP_DATA.ports.len());
        let chip_eval =
            HalveChipEval { input: slots[0].0, output: slots[1].0 };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for HalveChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input = state.recv_behavior(self.input);
        state.send_behavior(self.output, input >> 1);
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
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl IncChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
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

pub const MUL_CHIP_DATA: &ChipData = ADD_CHIP_DATA;

pub struct MulChipEval {
    size: WireSize,
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl MulChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), MUL_CHIP_DATA.ports.len());
        let chip_eval = MulChipEval {
            size: slots[2].1,
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for MulChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.behavior_changed(self.input1)
            || state.behavior_changed(self.input2)
        {
            let input1 = state.recv_behavior(self.input1);
            let input2 = state.recv_behavior(self.input2);
            let prod = input1.wrapping_mul(input2) & self.size.mask();
            state.send_behavior(self.output, prod);
        }
    }
}

//===========================================================================//

pub const MUL_4BIT_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Four),
        AbstractConstraint::Exact(1, WireSize::Four),
        AbstractConstraint::Exact(2, WireSize::Four),
        AbstractConstraint::Exact(3, WireSize::Four),
    ],
    dependencies: &[(0, 2), (1, 2), (0, 3), (1, 3)],
};

pub struct Mul4BitChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
    carry: WireId,
}

impl Mul4BitChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), MUL_4BIT_CHIP_DATA.ports.len());
        let chip_eval = Mul4BitChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
            carry: slots[3].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for Mul4BitChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.behavior_changed(self.input1)
            || state.behavior_changed(self.input2)
        {
            let input1 = state.recv_behavior(self.input1);
            let input2 = state.recv_behavior(self.input2);
            let product = input1 * input2;
            let lo = product & 0xf;
            let hi = (product >> 4) & 0xf;
            state.send_behavior(self.output, lo);
            state.send_behavior(self.carry, hi);
        }
    }
}

//===========================================================================//

pub const NEG_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
    ],
    constraints: &[AbstractConstraint::Equal(0, 1)],
    dependencies: &[(0, 1)],
};

pub struct NegChipEval {
    size: WireSize,
    input: WireId,
    output: WireId,
}

impl NegChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), NEG_CHIP_DATA.ports.len());
        let chip_eval = NegChipEval {
            size: slots[1].1,
            input: slots[0].0,
            output: slots[1].0,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for NegChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input = state.recv_behavior(self.input);
        let output = (!input).wrapping_add(1) & self.size.mask();
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//

pub const SUB_CHIP_DATA: &ChipData = ADD_CHIP_DATA;

pub struct SubChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl SubChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), SUB_CHIP_DATA.ports.len());
        let chip_eval = SubChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for SubChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.behavior_changed(self.input1)
            || state.behavior_changed(self.input2)
        {
            let input1 = state.recv_behavior(self.input1);
            let input2 = state.recv_behavior(self.input2);
            let diff = (input1 as i64) - (input2 as i64);
            state.send_behavior(self.output, diff.abs() as u32);
        }
    }
}

//===========================================================================//
