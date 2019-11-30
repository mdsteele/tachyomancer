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
use super::shared;
use crate::geom::{Coords, Direction};
use crate::state::{PortColor, PortFlow, WireSize};
use std::u32;
use std::u64;

//===========================================================================//

pub const COUNTER_INTERFACES: &[Interface] = &[
    Interface {
        name: "Set",
        description:
            "When an event is sent from here, the counter should be set to \
             that value.",
        side: Direction::South,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Set",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Inc",
        description:
            "When an event is sent from here, the counter value should be \
             incremented by one.",
        side: Direction::North,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Inc",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Dec",
        description:
            "When an event is sent from here, the counter value should be \
             decremented by one.",
        side: Direction::South,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Dec",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Out",
        description:
            "Should be equal to the current counter value (initially zero).",
        side: Direction::North,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
];

pub struct FabricateCounterEval {
    table_values: Vec<u64>,
    set_wire: usize,
    inc_wire: usize,
    dec_wire: usize,
    out_wire: usize,
    out_port: (Coords, Direction),
}

impl FabricateCounterEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> FabricateCounterEval {
        debug_assert_eq!(slots.len(), 4);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        debug_assert_eq!(slots[3].len(), 1);
        FabricateCounterEval {
            table_values: FabricateCounterEval::expected_table_values()
                .to_vec(),
            set_wire: slots[0][0].1,
            inc_wire: slots[1][0].1,
            dec_wire: slots[2][0].1,
            out_wire: slots[3][0].1,
            out_port: slots[3][0].0,
        }
    }
}

impl PuzzleEval for FabricateCounterEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        let expected = FabricateCounterEval::expected_table_values();
        let start = (state.time_step() as usize) * 4;
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            if slice[0] <= (u32::MAX as u64) {
                state.send_event(self.set_wire, slice[0] as u32);
            }
            if slice[1] <= (u32::MAX as u64) {
                state.send_event(self.inc_wire, slice[1] as u32);
            }
            if slice[2] <= (u32::MAX as u64) {
                state.send_event(self.dec_wire, slice[2] as u32);
            }
            None
        }
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let expected_table = FabricateCounterEval::expected_table_values();
        let start = (state.time_step() as usize) * 4;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_output = expected_table[start + 3] as u32;
        let actual_output = state.recv_behavior(self.out_wire);
        if actual_output != expected_output {
            let message = format!(
                "Expected output {} at time step {}, but output was {}",
                expected_output,
                state.time_step(),
                actual_output
            );
            vec![state.port_error(self.out_port, message)]
        } else {
            vec![]
        }
    }
}

impl FabricationEval for FabricateCounterEval {
    fn table_column_names() -> &'static [&'static str] {
        &["Set", "Inc", "Dec", "Out"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            u64::MAX, u64::MAX, u64::MAX,  0,
            u64::MAX,        0, u64::MAX,  1,
            u64::MAX,        0, u64::MAX,  2,
            u64::MAX,        0, u64::MAX,  3,
            u64::MAX, u64::MAX,        0,  2,
            u64::MAX, u64::MAX,        0,  1,
            u64::MAX, u64::MAX,        0,  0,
            u64::MAX, u64::MAX,        0, 15,
            u64::MAX, u64::MAX,        0, 14,
            u64::MAX,        0, u64::MAX, 15,
            u64::MAX,        0, u64::MAX,  0,
            u64::MAX,        0, u64::MAX,  1,
                   7, u64::MAX, u64::MAX,  7,
            u64::MAX,        0, u64::MAX,  8,
                  11, u64::MAX, u64::MAX, 11,
            u64::MAX, u64::MAX,        0, 10,
                   3,        0, u64::MAX,  4,
                  13, u64::MAX,        0, 12,
            u64::MAX,        0,        0, 12,
                   5,        0,        0,  5,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//

pub const INC_INTERFACES: &[Interface] = &[
    Interface {
        name: "InE",
        description: "Input events arrive here.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "InE",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "InB",
        description: "Provides the value that should be added to each event.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "InB",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Out",
        description:
            "Whenever an input event arrives, send an event here with the sum \
             of the two input values.",
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

pub struct FabricateIncEval {
    table_values: Vec<u64>,
    input_e_wire: usize,
    input_b_wire: usize,
    output_wire: usize,
    output_port: (Coords, Direction),
    has_received_output_event: bool,
}

impl FabricateIncEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> FabricateIncEval {
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
            has_received_output_event: false,
        }
    }
}

impl PuzzleEval for FabricateIncEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        self.has_received_output_event = false;
        let expected = FabricateIncEval::expected_table_values();
        let start = (state.time_step() as usize) * 3;
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

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let time_step = state.time_step();
        let expected_table = FabricateIncEval::expected_table_values();
        let start = (time_step as usize) * 3;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_output = expected_table[start + 2];

        let opt_actual_output = state.recv_event(self.output_wire);
        self.table_values[start + 2] =
            shared::opt_u32_to_u64(opt_actual_output);

        let mut errors = Vec::new();
        shared::end_cycle_check_event_output(
            state,
            opt_actual_output,
            expected_output,
            &mut self.has_received_output_event,
            self.output_port,
            &mut errors,
        );
        return errors;
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let expected_table = FabricateIncEval::expected_table_values();
        let start = (state.time_step() as usize) * 3;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_output = expected_table[start + 2];

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

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//
