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
use super::super::eval::{CircuitState, EvalError, EvalScore, FabricationEval,
                         PuzzleEval};
use tachy::geom::{Coords, Direction};
use tachy::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const XOR_INTERFACES: &[Interface] = &[
    Interface {
        name: "In1",
        description: "First input (0 or 1).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "In1",
                description: "",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::One,
            },
        ],
    },
    Interface {
        name: "In2",
        description: "Second input (0 or 1).",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "In2",
                description: "",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::One,
            },
        ],
    },
    Interface {
        name: "Out",
        description: "\
            Should be 1 if exactly one input is 1.\n\
            Should be 0 if the inputs are both 0 or both 1.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Out",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::One,
            },
        ],
    },
];

//===========================================================================//

pub struct FabricateXorEval {
    table_values: Vec<u64>,
    input1_wire: usize,
    input2_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
}

impl FabricateXorEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>)
               -> FabricateXorEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        FabricateXorEval {
            table_values: FabricateXorEval::expected_table_values().to_vec(),
            input1_wire: slots[0][0].1,
            input2_wire: slots[1][0].1,
            output_wire: slots[2][0].1,
            output_port: slots[2][0].0,
        }
    }
}

impl PuzzleEval for FabricateXorEval {
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
        let expected = input1 ^ input2;
        let actual = state.recv_behavior(self.output_wire).0;
        self.table_values[3 * (time_step as usize) + 2] = actual as u64;
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

impl FabricationEval for FabricateXorEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In1", "In2", "Out"]
    }

    fn expected_table_values() -> &'static [u64] {
        &[0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0]
    }

    fn table_values(&self) -> &[u64] { &self.table_values }
}

//===========================================================================//
