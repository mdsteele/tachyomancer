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

use super::super::eval::{
    CircuitState, EvalError, EvalScore, FabricationEval, PuzzleEval,
};
use super::iface::{Interface, InterfacePort, InterfacePosition};
use super::shared::TutorialBubblePosition;
use crate::geom::{Coords, Direction};
use crate::state::{PortColor, PortFlow, WireSize};
use std::u32;
use std::u64;

//===========================================================================//

pub const OR_INTERFACES: &[Interface] = &[
    Interface {
        name: "In1",
        description: "First input (0 or 1).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In1",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "In2",
        description: "Second input (0 or 1).",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In2",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Out",
        description: "\
                      Should be 1 if either input is 1.\n\
                      Should be 0 if both inputs are 0.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
];

pub const OR_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::PartsTray,
        "Drag chips from the parts\ntray onto the board.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "Drag between board squares to\ncreate wires between chip ports.",
    ),
    (
        TutorialBubblePosition::ControlsTray,
        "When you're ready, press\nthe play button to test\nyour design.",
    ),
];

pub struct TutorialOrEval {
    table_values: Vec<u64>,
    input1_wire: usize,
    input2_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
}

impl TutorialOrEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TutorialOrEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        TutorialOrEval {
            table_values: TutorialOrEval::expected_table_values().to_vec(),
            input1_wire: slots[0][0].1,
            input2_wire: slots[1][0].1,
            output_wire: slots[2][0].1,
            output_port: slots[2][0].0,
        }
    }
}

impl PuzzleEval for TutorialOrEval {
    fn seconds_per_time_step(&self) -> f64 {
        0.2
    }

    fn begin_time_step(
        &mut self,
        time_step: u32,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        if time_step >= 4 {
            Some(EvalScore::WireLength)
        } else {
            state.send_behavior(self.input1_wire, time_step & 0x1);
            state.send_behavior(self.input2_wire, (time_step & 0x2) >> 1);
            None
        }
    }

    fn end_time_step(
        &mut self,
        time_step: u32,
        state: &CircuitState,
    ) -> Vec<EvalError> {
        let input1 = state.recv_behavior(self.input1_wire).0;
        let input2 = state.recv_behavior(self.input2_wire).0;
        let expected = input1 | input2;
        let actual = state.recv_behavior(self.output_wire).0;
        self.table_values[3 * (time_step as usize) + 2] = actual as u64;
        if actual != expected {
            let error = EvalError {
                time_step,
                port: Some(self.output_port),
                message: format!(
                    "Expected output {} for inputs {} and {}, \
                     but output was {}",
                    expected, input1, input2, actual
                ),
            };
            vec![error]
        } else {
            vec![]
        }
    }
}

impl FabricationEval for TutorialOrEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In1", "In2", "Out"]
    }

    fn expected_table_values() -> &'static [u64] {
        &[0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//

pub const MUX_INTERFACES: &[Interface] = &[
    Interface {
        name: "In0",
        description: "The input to use when the control value is 0.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In0",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "In1",
        description: "The input to use when the control value is 1.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In1",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Ctrl",
        description: "Indicates which input should be sent to the output.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Ctrl",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Out",
        description: "Should be the value of $*In0$* if $*Ctrl$* is 0, or \
                      of $*In1$* if $*Ctrl$* is 1.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
];

pub const MUX_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "Drag sideways from a wire to split it.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::South),
        "Perpendicular wires can cross over each other.  Click on a \
         crossing to toggle whether the wires are connected.",
    ),
];

pub struct TutorialMuxEval {
    table_values: Vec<u64>,
    input0_wire: usize,
    input1_wire: usize,
    control_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
}

impl TutorialMuxEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TutorialMuxEval {
        debug_assert_eq!(slots.len(), 4);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        debug_assert_eq!(slots[3].len(), 1);
        TutorialMuxEval {
            table_values: TutorialMuxEval::expected_table_values().to_vec(),
            input0_wire: slots[0][0].1,
            input1_wire: slots[1][0].1,
            control_wire: slots[2][0].1,
            output_wire: slots[3][0].1,
            output_port: slots[3][0].0,
        }
    }
}

impl PuzzleEval for TutorialMuxEval {
    fn begin_time_step(
        &mut self,
        time_step: u32,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        let expected = TutorialMuxEval::expected_table_values();
        let start = (time_step as usize) * 4;
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            state.send_behavior(self.input0_wire, slice[0] as u32);
            state.send_behavior(self.input1_wire, slice[1] as u32);
            state.send_behavior(self.control_wire, slice[2] as u32);
            None
        }
    }

    fn end_time_step(
        &mut self,
        time_step: u32,
        state: &CircuitState,
    ) -> Vec<EvalError> {
        let input0 = state.recv_behavior(self.input0_wire).0;
        let input1 = state.recv_behavior(self.input1_wire).0;
        let control = state.recv_behavior(self.control_wire).0;
        let expected = if control == 0 { input0 } else { input1 };
        let actual = state.recv_behavior(self.output_wire).0;
        self.table_values[4 * (time_step as usize) + 3] = actual as u64;
        if actual != expected {
            let error = EvalError {
                time_step,
                port: Some(self.output_port),
                message: format!(
                    "Expected output {} for inputs {} and {} \
                     with control {}, but output was {}",
                    expected, input0, input1, control, actual
                ),
            };
            vec![error]
        } else {
            vec![]
        }
    }
}

impl FabricationEval for TutorialMuxEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In0", "In1", "Ctrl", "Out"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            0, 0, 0, 0,
            0, 0, 1, 0,
            0, 1, 0, 0,
            0, 1, 1, 1,
            1, 0, 0, 1,
            1, 0, 1, 0,
            1, 1, 0, 1,
            1, 1, 1, 1,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//

pub const ADD_INTERFACES: &[Interface] = &[
    Interface {
        name: "In1",
        description: "First input (from 0 to 15).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In1",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "In2",
        description: "Second input (from 0 to 15).",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In2",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Out",
        description:
            "\
             Should be the sum of the two inputs (which will never be more \
             than 15 for this task).",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
];

pub const ADD_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "Drag the edges of the board to resize it.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::South),
        "Drag from any grid cell corner in the board to select part of the \
         circuit and cut/copy/paste.",
    ),
];

pub struct TutorialAddEval {
    table_values: Vec<u64>,
    input1_wire: usize,
    input2_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
}

impl TutorialAddEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TutorialAddEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        TutorialAddEval {
            table_values: TutorialAddEval::expected_table_values().to_vec(),
            input1_wire: slots[0][0].1,
            input2_wire: slots[1][0].1,
            output_wire: slots[2][0].1,
            output_port: slots[2][0].0,
        }
    }
}

impl PuzzleEval for TutorialAddEval {
    fn begin_time_step(
        &mut self,
        time_step: u32,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        let expected = TutorialAddEval::expected_table_values();
        let start = (time_step as usize) * 3;
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            state.send_behavior(self.input1_wire, slice[0] as u32);
            state.send_behavior(self.input2_wire, slice[1] as u32);
            None
        }
    }

    fn end_time_step(
        &mut self,
        time_step: u32,
        state: &CircuitState,
    ) -> Vec<EvalError> {
        let input1 = state.recv_behavior(self.input1_wire).0;
        let input2 = state.recv_behavior(self.input2_wire).0;
        let expected = input1 + input2;
        let actual = state.recv_behavior(self.output_wire).0;
        self.table_values[3 * (time_step as usize) + 2] = actual as u64;
        if actual != expected {
            let error = EvalError {
                time_step,
                port: Some(self.output_port),
                message: format!(
                    "Expected output {} for inputs {} and {}, \
                     but output was {}",
                    expected, input1, input2, actual
                ),
            };
            vec![error]
        } else {
            vec![]
        }
    }
}

impl FabricationEval for TutorialAddEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In1", "In2", "Out"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            2, 1, 3,
            5, 7, 12,
            8, 2, 10,
            4, 4, 8,
            7, 7, 14,
            3, 6, 9,
            6, 5, 11,
            1, 3, 4,
            0, 15, 15,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//
