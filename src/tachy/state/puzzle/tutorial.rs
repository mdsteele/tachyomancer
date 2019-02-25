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

use super::iface::{Interface, InterfacePort, InterfacePosition};
use super::super::eval::{CircuitState, EvalError, EvalScore, PuzzleEval};
use tachy::geom::{Coords, Direction};
use tachy::save::Puzzle;
use tachy::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const OR_INTERFACES: &[Interface] = &[
    Interface {
        name: "Input1",
        description: "First input (0 or 1).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "",
                description: "",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::One,
            },
        ],
    },
    Interface {
        name: "Input2",
        description: "Second input (0 or 1).",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "",
                description: "",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::One,
            },
        ],
    },
    Interface {
        name: "Output",
        description: "\
                        Should be 1 if either input is 1.\n\
                        Should be 0 if both inputs are 0.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::One,
            },
        ],
    },
];

//===========================================================================//

pub struct TutorialOrEval {
    verification: Vec<u64>,
    input1_wire: usize,
    input2_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
}

impl TutorialOrEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>)
               -> TutorialOrEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        TutorialOrEval {
            verification: Puzzle::TutorialOr
                .static_verification_data()
                .to_vec(),
            input1_wire: slots[0][0].1,
            input2_wire: slots[1][0].1,
            output_wire: slots[2][0].1,
            output_port: slots[2][0].0,
        }
    }
}

impl PuzzleEval for TutorialOrEval {
    fn verification_data(&self) -> &[u64] { &self.verification }

    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        if time_step >= 4 {
            Some(EvalScore::WireLength)
        } else {
            state.send_behavior(self.input1_wire, time_step & 0x1);
            state.send_behavior(self.input2_wire, (time_step & 0x2) >> 1);
            None
        }
    }

    fn end_time_step(&mut self, time_step: u32, state: &CircuitState)
                     -> Vec<EvalError> {
        let input1 = state.recv_behavior(self.input1_wire).0;
        let input2 = state.recv_behavior(self.input2_wire).0;
        let expected = input1 | input2;
        let actual = state.recv_behavior(self.output_wire).0;
        self.verification[3 * (time_step as usize) + 2] = actual as u64;
        if actual != expected {
            let error = EvalError {
                time_step,
                port: Some(self.output_port),
                message: format!("Expected output {} for inputs {} and {}, \
                                  but output was {}",
                                 expected,
                                 input1,
                                 input2,
                                 actual),
            };
            vec![error]
        } else {
            vec![]
        }
    }
}

//===========================================================================//
