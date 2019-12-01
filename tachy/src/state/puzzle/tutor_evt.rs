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
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Ctrl",
        description:
            "Indicates which output (0-3) the event should be sent to.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Ctrl",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Two,
        }],
    },
    Interface {
        name: "Out",
        description:
            "Input events should be sent to $*Out0$* when $*Ctrl$* is 0, to \
             $*Out1$* when $*Ctrl$* is 1, and so on.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Out0",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Out1",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Out2",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Out3",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
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
        "A single $*Demux$* chip can direct an event to one of two places.  \
         A chain of multiple $*Demux$* chips can bifurcate even further.",
    ),
];

pub struct TutorialDemuxEval {
    table_values: Vec<u64>,
    input_wire: usize,
    control_wire: usize,
    output_ports: [(Coords, Direction); 4],
    output_wires: [usize; 4],
    has_received_output_event: [bool; 4],
}

impl TutorialDemuxEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TutorialDemuxEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 4);
        TutorialDemuxEval {
            table_values: TutorialDemuxEval::expected_table_values().to_vec(),
            input_wire: slots[0][0].1,
            control_wire: slots[1][0].1,
            output_ports: [
                slots[2][0].0,
                slots[2][1].0,
                slots[2][2].0,
                slots[2][3].0,
            ],
            output_wires: [
                slots[2][0].1,
                slots[2][1].1,
                slots[2][2].1,
                slots[2][3].1,
            ],
            has_received_output_event: [false; 4],
        }
    }
}

impl PuzzleEval for TutorialDemuxEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        self.has_received_output_event = [false; 4];
        let expected = TutorialDemuxEval::expected_table_values();
        let start = (state.time_step() as usize) * 6;
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
        let expected_table = TutorialDemuxEval::expected_table_values();
        let start = (state.time_step() as usize) * 6;
        if start >= expected_table.len() {
            return vec![];
        }
        let mut errors = Vec::new();
        for index in 0..4 {
            let expected_output = expected_table[start + 2 + index];
            let opt_output = state.recv_event(self.output_wires[index]);
            self.table_values[start + 2 + index] =
                shared::opt_u32_to_u64(opt_output);
            shared::end_cycle_check_event_output(
                state,
                opt_output,
                expected_output,
                &mut self.has_received_output_event[index],
                self.output_ports[index],
                &mut errors,
            );
        }
        return errors;
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let expected_table = TutorialDemuxEval::expected_table_values();
        let start = (state.time_step() as usize) * 6;
        if start >= expected_table.len() {
            return vec![];
        }
        let mut errors = Vec::new();
        for index in 0..4 {
            let expected_output = expected_table[start + 2 + index];
            shared::end_time_step_check_event_output(
                state,
                expected_output,
                self.has_received_output_event[index],
                self.output_ports[index],
                &mut errors,
            );
        }
        return errors;
    }
}

impl FabricationEval for TutorialDemuxEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In", "Ctrl", "Out0", "Out1", "Out2", "Out3"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            0, 0, 0, u64::MAX, u64::MAX, u64::MAX,
            u64::MAX, 0, u64::MAX, u64::MAX, u64::MAX, u64::MAX,
            0, 1, u64::MAX, 0, u64::MAX, u64::MAX,
            u64::MAX, 1, u64::MAX, u64::MAX, u64::MAX, u64::MAX,
            0, 2, u64::MAX, u64::MAX, 0, u64::MAX,
            u64::MAX, 2, u64::MAX, u64::MAX, u64::MAX, u64::MAX,
            0, 3, u64::MAX, u64::MAX, u64::MAX, 0,
            u64::MAX, 3, u64::MAX, u64::MAX, u64::MAX, u64::MAX,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//

pub const AMP_INTERFACES: &[Interface] = &[
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
        name: "Out",
        description:
            "Output events should be sent here.  The output value should be \
             twice the input value, unless that would be more than 10, in \
             which case no output value should be sent.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
];

pub const AMP_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "Splitting an event wire sends the same event to multiple \
         receiver ports.  Then each copy of the event can be used \
         separately.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::South),
        "A $*Demux$* chip can be used to filter out unwanted events by \
         ignoring one of its two output ports.",
    ),
];

pub struct TutorialAmpEval {
    table_values: Vec<u64>,
    input_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
    has_received_output_event: bool,
}

impl TutorialAmpEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TutorialAmpEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        TutorialAmpEval {
            table_values: TutorialAmpEval::expected_table_values().to_vec(),
            input_wire: slots[0][0].1,
            output_wire: slots[1][0].1,
            output_port: slots[1][0].0,
            has_received_output_event: false,
        }
    }
}

impl PuzzleEval for TutorialAmpEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        self.has_received_output_event = false;
        let expected = TutorialAmpEval::expected_table_values();
        let start = (state.time_step() as usize) * 2;
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            if slice[0] < (u32::MAX as u64) {
                state.send_event(self.input_wire, slice[0] as u32);
            }
            None
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let expected_table = TutorialAmpEval::expected_table_values();
        let start = (state.time_step() as usize) * 2;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_output = expected_table[start + 1];

        let opt_output = state.recv_event(self.output_wire);
        self.table_values[start + 1] = shared::opt_u32_to_u64(opt_output);

        let mut errors = Vec::new();
        shared::end_cycle_check_event_output(
            state,
            opt_output,
            expected_output,
            &mut self.has_received_output_event,
            self.output_port,
            &mut errors,
        );
        return errors;
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let expected_table = TutorialAmpEval::expected_table_values();
        let start = (state.time_step() as usize) * 2;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_output = expected_table[start + 1];

        let mut errors = Vec::new();
        shared::end_time_step_check_event_output(
            state,
            expected_output,
            self.has_received_output_event,
            self.output_port,
            &mut errors,
        );
        return errors;
    }
}

impl FabricationEval for TutorialAmpEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In", "Out"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            3, 6,
            u64::MAX, u64::MAX,
            1, 2,
            5, 10,
            6, u64::MAX,
            4, 8,
            7, u64::MAX,
            0, 0,
            u64::MAX, u64::MAX,
            2, 4,
            1, 2,
            6, u64::MAX,
            3, 6,
            5, 10,
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

pub const SUM_BUBBLES: &[(TutorialBubblePosition, &str)] = &[(
    TutorialBubblePosition::Bounds(Direction::South),
    "A $*Latest$* chip can act as a memory cell.  Use it in a loop with a \
     $*Delay$* chip to set a new value based on the old one.",
)];

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
            let message = format!(
                "Expected output of {}, but output was {}",
                expected, actual
            );
            vec![state.port_error(self.output_port, message)]
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
