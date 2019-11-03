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
use super::shared::{self, TutorialBubblePosition};
use crate::geom::{Coords, Direction};
use crate::state::{PortColor, PortFlow, WireSize};
use std::u32;
use std::u64;

//===========================================================================//

pub const DEMUX_INTERFACES: &[Interface] = &[
    Interface {
        name: "In",
        description: "Input events arrive here.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Ctrl",
        description: "Indicates which output the event should be sent to.",
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
        name: "Out0",
        description: "Input events should be sent here when $*Ctrl$* is 0.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out0",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Out1",
        description: "Input events should be sent here when $*Ctrl$* is 1.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out1",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::One,
        }],
    },
];

pub const DEMUX_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "A wire can carry either $Obehaviors$D or $Cevents$D, depending on \
         which color of ports it is connected to.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::East),
        "Splitting an event wire allows sending the same event to multiple \
         receiver ports.  Then each copy of the event can be filtered \
         separately.",
    ),
];

pub struct TutorialDemuxEval {
    table_values: Vec<u64>,
    input_wire: usize,
    control_wire: usize,
    output0_wire: usize,
    output0_port: (Coords, Direction),
    output1_wire: usize,
    output1_port: (Coords, Direction),
    has_received_output0_event: bool,
    has_received_output1_event: bool,
}

impl TutorialDemuxEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TutorialDemuxEval {
        debug_assert_eq!(slots.len(), 4);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        debug_assert_eq!(slots[3].len(), 1);
        TutorialDemuxEval {
            table_values: TutorialDemuxEval::expected_table_values().to_vec(),
            input_wire: slots[0][0].1,
            control_wire: slots[1][0].1,
            output0_wire: slots[2][0].1,
            output0_port: slots[2][0].0,
            output1_wire: slots[3][0].1,
            output1_port: slots[3][0].0,
            has_received_output0_event: false,
            has_received_output1_event: false,
        }
    }
}

impl PuzzleEval for TutorialDemuxEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        self.has_received_output0_event = false;
        self.has_received_output1_event = false;
        let expected = TutorialDemuxEval::expected_table_values();
        let start = (state.time_step() as usize) * 4;
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            if slice[0] < (u32::MAX as u64) {
                state.send_event(self.input_wire, slice[0] as u32);
            }
            state.send_behavior(self.control_wire, slice[1] as u32);
            None
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let time_step = state.time_step();
        let expected_table = TutorialDemuxEval::expected_table_values();
        let start = (time_step as usize) * 4;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_output0 = expected_table[start + 2];
        let expected_output1 = expected_table[start + 3];

        let opt_output0 = state.recv_event(self.output0_wire);
        self.table_values[start + 2] = shared::opt_u32_to_u64(opt_output0);
        let opt_output1 = state.recv_event(self.output1_wire);
        self.table_values[start + 3] = shared::opt_u32_to_u64(opt_output1);

        let mut errors = Vec::new();
        shared::end_cycle_check_event_output(
            opt_output0,
            expected_output0,
            &mut self.has_received_output0_event,
            self.output0_port,
            time_step,
            &mut errors,
        );
        shared::end_cycle_check_event_output(
            opt_output1,
            expected_output1,
            &mut self.has_received_output1_event,
            self.output1_port,
            time_step,
            &mut errors,
        );
        return errors;
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let time_step = state.time_step();
        let expected_table = TutorialDemuxEval::expected_table_values();
        let start = (time_step as usize) * 4;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_output0 = expected_table[start + 2];
        let expected_output1 = expected_table[start + 3];

        let mut errors = Vec::new();
        shared::end_time_step_check_event_output(
            expected_output0,
            self.has_received_output0_event,
            self.output0_port,
            time_step,
            &mut errors,
        );
        shared::end_time_step_check_event_output(
            expected_output1,
            self.has_received_output1_event,
            self.output1_port,
            time_step,
            &mut errors,
        );
        return errors;
    }
}

impl FabricationEval for TutorialDemuxEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In", "Ctrl", "Out0", "Out1"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            0, 0, 0, u64::MAX,
            0, 1, u64::MAX, 0,
            u64::MAX, 0, u64::MAX, u64::MAX,
            1, 1, u64::MAX, 1,
            1, 0, 1, u64::MAX,
            u64::MAX, 1, u64::MAX, u64::MAX,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//

pub const SUM_INTERFACES: &[Interface] = &[
    Interface {
        name: "In",
        description: "Input events arrive here.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Reset",
        description: "When an event arrives here, the output sum should be \
                      reset to zero.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Reset",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Out",
        description:
            "Should equal the sum of all input events since the last \
             reset.",
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

pub const SUM_BUBBLES: &[(TutorialBubblePosition, &str)] = &[];

pub struct TutorialSumEval {
    table_values: Vec<u64>,
    input_wire: usize,
    reset_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
}

impl TutorialSumEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TutorialSumEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        TutorialSumEval {
            table_values: TutorialSumEval::expected_table_values().to_vec(),
            input_wire: slots[0][0].1,
            reset_wire: slots[1][0].1,
            output_wire: slots[2][0].1,
            output_port: slots[2][0].0,
        }
    }
}

impl PuzzleEval for TutorialSumEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        let expected = TutorialSumEval::expected_table_values();
        let start = (state.time_step() as usize)
            * TutorialSumEval::table_column_names().len();
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            if slice[0] < (u32::MAX as u64) {
                state.send_event(self.input_wire, slice[0] as u32);
            }
            if slice[1] < (u32::MAX as u64) {
                state.send_event(self.reset_wire, slice[1] as u32);
            }
            None
        }
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let time_step = state.time_step();
        let expected_table = TutorialSumEval::expected_table_values();
        let start =
            (time_step as usize) * TutorialSumEval::table_column_names().len();
        let expected = expected_table[start + 2] as u32;
        let actual = state.recv_behavior(self.output_wire);
        self.table_values[start + 2] = actual as u64;
        if actual != expected {
            let error = EvalError {
                time_step,
                port: Some(self.output_port),
                message: format!(
                    "Expected output of {}, but output was {}",
                    expected, actual
                ),
            };
            vec![error]
        } else {
            vec![]
        }
    }
}

impl FabricationEval for TutorialSumEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In", "Reset", "Out"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            u64::MAX, u64::MAX, 0,
            5, u64::MAX, 5,
            7, u64::MAX, 12,
            u64::MAX, u64::MAX, 12,
            1, u64::MAX, 13,
            u64::MAX, 0, 0,
            6, u64::MAX, 6,
            2, 0, 2,
            3, u64::MAX, 5,
            9, u64::MAX, 14,
            u64::MAX, 0, 0,
            u64::MAX, 0, 0,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//
