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

pub const ACMP_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Analog, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Analog, (0, 0), Direction::East),
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::ANALOG),
        AbstractConstraint::Exact(1, WireSize::ANALOG),
        AbstractConstraint::Exact(2, WireSize::Zero),
        AbstractConstraint::Exact(3, WireSize::One),
    ],
    dependencies: &[(0, 3), (1, 3), (2, 3)],
};

pub struct ACmpChipEval {
    input1: WireId,
    input2: WireId,
    test: WireId,
    output: WireId,
}

impl ACmpChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), ACMP_CHIP_DATA.ports.len());
        let chip_eval = ACmpChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            test: slots[2].0,
            output: slots[3].0,
        };
        vec![(3, Box::new(chip_eval))]
    }
}

impl ChipEval for ACmpChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.test) {
            let input1 = state.recv_analog(self.input1);
            let input2 = state.recv_analog(self.input2);
            let output = if input1 < input2 { 1 } else { 0 };
            state.send_event(self.output, output);
        }
    }
}

//===========================================================================//

pub const CMP_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::East),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Exact(2, WireSize::One),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct CmpChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl CmpChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), CMP_CHIP_DATA.ports.len());
        let chip_eval = CmpChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for CmpChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        let output = if input1 < input2 { 1 } else { 0 };
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//

pub const CMPEQ_CHIP_DATA: &ChipData = CMP_CHIP_DATA;

pub struct CmpEqChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl CmpEqChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), CMPEQ_CHIP_DATA.ports.len());
        let chip_eval = CmpEqChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for CmpEqChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        let output = if input1 <= input2 { 1 } else { 0 };
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//

pub const EQ_CHIP_DATA: &ChipData = CMP_CHIP_DATA;

pub struct EqChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl EqChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), EQ_CHIP_DATA.ports.len());
        let chip_eval = EqChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for EqChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        let output = if input1 == input2 { 1 } else { 0 };
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//
