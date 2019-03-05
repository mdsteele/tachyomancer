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
    input1: usize,
    input2: usize,
    output: usize,
}

impl CmpChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
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
        let input1 = state.recv_behavior(self.input1).0;
        let input2 = state.recv_behavior(self.input2).0;
        let output = if input1 < input2 { 1 } else { 0 };
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//

pub const CMPEQ_CHIP_DATA: &ChipData = CMP_CHIP_DATA;

pub struct CmpEqChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl CmpEqChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
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
        let input1 = state.recv_behavior(self.input1).0;
        let input2 = state.recv_behavior(self.input2).0;
        let output = if input1 <= input2 { 1 } else { 0 };
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//

pub const EQ_CHIP_DATA: &ChipData = CMP_CHIP_DATA;

pub struct EqChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl EqChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
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
        let input1 = state.recv_behavior(self.input1).0;
        let input2 = state.recv_behavior(self.input2).0;
        let output = if input1 == input2 { 1 } else { 0 };
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//
