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

pub const AND_CHIP_DATA: &ChipData = &ChipData {
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

pub struct AndChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl AndChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), AND_CHIP_DATA.ports.len());
        let chip_eval = AndChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for AndChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        state.send_behavior(self.output, input1 & input2);
    }
}

//===========================================================================//

pub const MUX_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
        AbstractConstraint::Exact(3, WireSize::One),
    ],
    dependencies: &[(0, 2), (1, 2), (3, 2)],
};

pub struct MuxChipEval {
    input1: usize,
    input2: usize,
    output: usize,
    control: usize,
}

impl MuxChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), MUX_CHIP_DATA.ports.len());
        let chip_eval = MuxChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
            control: slots[3].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for MuxChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let output = if state.recv_behavior(self.control) == 0 {
            state.recv_behavior(self.input1)
        } else {
            state.recv_behavior(self.input2)
        };
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//

pub const NOT_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
    ],
    constraints: &[AbstractConstraint::Equal(0, 1)],
    dependencies: &[(0, 1)],
};

pub struct NotChipEval {
    size: WireSize,
    input: usize,
    output: usize,
}

impl NotChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), NOT_CHIP_DATA.ports.len());
        let chip_eval = NotChipEval {
            size: slots[1].1,
            input: slots[0].0,
            output: slots[1].0,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for NotChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input = state.recv_behavior(self.input);
        state.send_behavior(self.output, (!input) & self.size.mask());
    }
}

//===========================================================================//

pub const OR_CHIP_DATA: &ChipData = AND_CHIP_DATA;

pub struct OrChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl OrChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), OR_CHIP_DATA.ports.len());
        let chip_eval = OrChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for OrChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        state.send_behavior(self.output, input1 | input2);
    }
}

//===========================================================================//

pub const XOR_CHIP_DATA: &ChipData = AND_CHIP_DATA;

pub struct XorChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl XorChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), XOR_CHIP_DATA.ports.len());
        let chip_eval = XorChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for XorChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        state.send_behavior(self.output, input1 ^ input2);
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::super::super::eval::{
        ChipEval, CircuitEval, CircuitInteraction, NullPuzzleEval,
    };
    use super::{AndChipEval, NotChipEval};
    use crate::state::WireSize;
    use std::collections::HashSet;

    #[test]
    fn evaluate_boolean_or_circuit() {
        let chips: Vec<Vec<Box<dyn ChipEval>>> = vec![
            vec![
                Box::new(NotChipEval {
                    size: WireSize::One,
                    input: 0,
                    output: 2,
                }),
                Box::new(NotChipEval {
                    size: WireSize::One,
                    input: 1,
                    output: 3,
                }),
            ],
            vec![Box::new(AndChipEval { input1: 2, input2: 3, output: 4 })],
            vec![Box::new(NotChipEval {
                size: WireSize::One,
                input: 4,
                output: 5,
            })],
        ];
        let mut eval = CircuitEval::new(
            6,
            HashSet::new(),
            chips,
            Box::new(NullPuzzleEval()),
            CircuitInteraction::new(),
        );
        for &inputs in &[(0, 0), (0, 1), (1, 0), (1, 1)] {
            eval.circuit_state_mut().send_behavior(0, inputs.0);
            eval.circuit_state_mut().send_behavior(1, inputs.1);
            let _ = eval.step_time();
            let output = eval.circuit_state_mut().recv_behavior(5);
            assert_eq!(output, inputs.0 | inputs.1, "inputs: {:?}", inputs);
        }
    }
}

//===========================================================================//
