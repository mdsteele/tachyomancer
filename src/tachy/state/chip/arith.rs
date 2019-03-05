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
    input1: usize,
    input2: usize,
    output: usize,
}

impl AddChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
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
        let (input1, changed1) = state.recv_behavior(self.input1);
        let (input2, changed2) = state.recv_behavior(self.input2);
        if changed1 || changed2 {
            let sum = (input1 as u64) + (input2 as u64);
            let lo = (sum & (self.size.mask() as u64)) as u32;
            state.send_behavior(self.output, lo);
        }
    }
}

//===========================================================================//

pub const SUB_CHIP_DATA: &ChipData = ADD_CHIP_DATA;

pub struct SubChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl SubChipEval {
    pub fn new_evals(slots: &[(usize, WireSize)])
                     -> Vec<(usize, Box<ChipEval>)> {
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
        let (input1, changed1) = state.recv_behavior(self.input1);
        let (input2, changed2) = state.recv_behavior(self.input2);
        if changed1 || changed2 {
            let diff = (input1 as i64) - (input2 as i64);
            state.send_behavior(self.output, diff.abs() as u32);
        }
    }
}

//===========================================================================//
