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
use std::u32;
use std::u64;
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

pub const INC_INTERFACES: &[Interface] = &[
    Interface {
        name: "InE",
        description: "Input events arrive here.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "InE",
                description: "",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "InB",
        description: "Provides the value that should be added to each event.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "InB",
                description: "",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Out",
        description: "\
            Whenever an input event arrives, send an event here with the sum \
            of the two input values.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Out",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Four,
            },
        ],
    },
];

//===========================================================================//

pub struct FabricateIncEval {
    table_values: Vec<u64>,
    input_e_wire: usize,
    input_b_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
}

impl FabricateIncEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>)
               -> FabricateIncEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        FabricateIncEval {
            table_values: FabricateIncEval::expected_table_values().to_vec(),
            input_e_wire: slots[0][0].1,
            input_b_wire: slots[1][0].1,
            output_wire: slots[2][0].1,
            output_port: slots[2][0].0,
        }
    }
}

impl PuzzleEval for FabricateIncEval {
    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        let expected = FabricateIncEval::expected_table_values();
        let start = (time_step as usize) * 3;
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            if slice[0] <= (u32::MAX as u64) {
                state.send_event(self.input_e_wire, slice[0] as u32);
            }
            state.send_behavior(self.input_b_wire, slice[1] as u32);
            None
        }
    }

    fn end_time_step(&mut self, time_step: u32, state: &CircuitState)
                     -> Vec<EvalError> {
        let expected = FabricateIncEval::expected_table_values();
        let start = (time_step as usize) * 3;
        if start >= expected.len() {
            return vec![];
        }
        let expected_output = expected[start + 2];
        if let Some(actual_output) = state.recv_event(self.output_wire) {
            self.table_values[3 * (time_step as usize) + 2] = actual_output as
                u64;
            if expected_output > (u32::MAX as u64) {
                let error = EvalError {
                    time_step,
                    port: Some(self.output_port),
                    message: format!("No output event expected, but got \
                                      output event value of {}",
                                     actual_output),
                };
                return vec![error];
            } else if actual_output != (expected_output as u32) {
                let error = EvalError {
                    time_step,
                    port: Some(self.output_port),
                    message: format!("Expected output event value of {}, \
                                      but output event value of {}",
                                     expected_output,
                                     actual_output),
                };
                return vec![error];
            }
        } else {
            self.table_values[3 * (time_step as usize) + 2] = u64::MAX;
            if expected_output <= (u32::MAX as u64) {
                let error = EvalError {
                    time_step,
                    port: Some(self.output_port),
                    message: format!("Expected output event value of {}, but \
                                      got no output event",
                                     expected_output),
                };
                return vec![error];
            }
        }
        return vec![];
    }
}

impl FabricationEval for FabricateIncEval {
    fn table_column_names() -> &'static [&'static str] {
        &["InE", "InB", "Out"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            4, 7, 11,
            u64::MAX, 12, u64::MAX,
            6, 0, 6,
            9, 1, 10,
            u64::MAX, 8, u64::MAX,
            0, 14, 14,
            1, 2, 3,
            5, 10, 15,
        ]
    }

    fn table_values(&self) -> &[u64] { &self.table_values }
}

//===========================================================================//
