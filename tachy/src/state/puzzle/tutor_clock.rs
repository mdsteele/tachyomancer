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

pub const CLOCK_INTERFACES: &[Interface] = &[
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
        description: "Output events should be sent here.",
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

pub const CLOCK_BUBBLES: &[(TutorialBubblePosition, &str)] = &[(
    TutorialBubblePosition::Bounds(Direction::North),
    "A $*Clock$* chip can be used together with a $*Latest$* chip to \
     delay a value by one time step.",
)];

pub struct TutorialClockEval {
    table_values: Vec<u64>,
    input_wire: usize,
    output_port: (Coords, Direction),
    output_wire: usize,
    has_received_output_event: bool,
}

impl TutorialClockEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> TutorialClockEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        TutorialClockEval {
            table_values: TutorialClockEval::expected_table_values().to_vec(),
            input_wire: slots[0][0].1,
            output_port: slots[1][0].0,
            output_wire: slots[1][0].1,
            has_received_output_event: false,
        }
    }
}

impl PuzzleEval for TutorialClockEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        self.has_received_output_event = false;
        let expected = TutorialClockEval::expected_table_values();
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
        let expected_table = TutorialClockEval::expected_table_values();
        let start = (state.time_step() as usize)
            * TutorialClockEval::table_column_names().len();
        if start >= expected_table.len() {
            return vec![];
        }
        let mut errors = Vec::new();
        let expected_output = expected_table[start + 1];
        let opt_output = state.recv_event(self.output_wire);
        if !self.has_received_output_event {
            self.table_values[start + 1] = shared::opt_u32_to_u64(opt_output);
        }
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
        let expected_table = TutorialClockEval::expected_table_values();
        let start = (state.time_step() as usize)
            * TutorialClockEval::table_column_names().len();
        if start >= expected_table.len() {
            return vec![];
        }
        let mut errors = Vec::new();
        let expected_output = expected_table[start + 1];
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

impl FabricationEval for TutorialClockEval {
    fn table_column_names() -> &'static [&'static str] {
        &["In", "Out"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            u64::MAX, u64::MAX,
            5,        u64::MAX,
            3,        u64::MAX,
            u64::MAX, 5,
            4,        u64::MAX,
            u64::MAX, u64::MAX,
            7,        4,
            14,       u64::MAX,
            u64::MAX, 7,
            u64::MAX, u64::MAX,
            9,        u64::MAX,
            8,        u64::MAX,
            2,        9,
            12,       u64::MAX,
            15,       2,
            0,        u64::MAX,
            6,        15,
            u64::MAX, u64::MAX,
            u64::MAX, 6,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//
