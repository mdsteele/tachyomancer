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

pub const EGG_TIMER_INTERFACES: &[Interface] = &[
    Interface {
        name: "Set",
        description:
            "When an event value arrives here, the timer should be set to go \
             off in that many time steps.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Set",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "Remain",
        description:
            "Should be the number of time steps before the timer goes off.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Remain",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "Alarm",
        description: "Send an event here when the timer goes off.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Alarm",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
];

pub struct FabricateEggTimerEval {
    table_values: Vec<u64>,
    set_wire: usize,
    remain_wire: usize,
    remain_port: (Coords, Direction),
    alarm_wire: usize,
    alarm_port: (Coords, Direction),
    has_received_alarm_event: bool,
}

impl FabricateEggTimerEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> FabricateEggTimerEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        FabricateEggTimerEval {
            table_values: FabricateEggTimerEval::expected_table_values()
                .to_vec(),
            set_wire: slots[0][0].1,
            remain_wire: slots[1][0].1,
            remain_port: slots[1][0].0,
            alarm_wire: slots[2][0].1,
            alarm_port: slots[2][0].0,
            has_received_alarm_event: false,
        }
    }
}

impl PuzzleEval for FabricateEggTimerEval {
    fn begin_time_step(
        &mut self,
        time_step: u32,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        self.has_received_alarm_event = false;
        let expected = FabricateEggTimerEval::expected_table_values();
        let start = (time_step as usize) * 3;
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            if slice[0] <= (u32::MAX as u64) {
                state.send_event(self.set_wire, slice[0] as u32);
            }
            None
        }
    }

    fn end_cycle(
        &mut self,
        time_step: u32,
        state: &CircuitState,
    ) -> Vec<EvalError> {
        let expected_table = FabricateEggTimerEval::expected_table_values();
        let start = (time_step as usize) * 3;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_alarm = expected_table[start + 2];

        let opt_actual_alarm = state.recv_event(self.alarm_wire);
        self.table_values[start + 2] =
            shared::opt_u32_to_u64(opt_actual_alarm);

        let mut errors = Vec::new();
        shared::end_cycle_check_event_output(
            opt_actual_alarm,
            expected_alarm,
            &mut self.has_received_alarm_event,
            self.alarm_port,
            time_step,
            &mut errors,
        );
        return errors;
    }

    fn end_time_step(
        &mut self,
        time_step: u32,
        state: &CircuitState,
    ) -> Vec<EvalError> {
        let expected_table = FabricateEggTimerEval::expected_table_values();
        let start = (time_step as usize) * 3;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_remain = expected_table[start + 1];
        let actual_remain = state.recv_behavior(self.remain_wire).0;
        let expected_alarm = expected_table[start + 2];

        let mut errors = Vec::new();
        if (actual_remain as u64) != expected_remain {
            errors.push(EvalError {
                time_step,
                port: Some(self.remain_port),
                message: format!(
                    "Expected remaining time to be {}, but was {}",
                    expected_remain, actual_remain
                ),
            });
        }
        shared::end_time_step_check_event_output(
            expected_alarm,
            self.has_received_alarm_event,
            self.alarm_port,
            time_step,
            &mut errors,
        );
        return errors;
    }
}

impl FabricationEval for FabricateEggTimerEval {
    fn table_column_names() -> &'static [&'static str] {
        &["Set", "Remain", "Alarm"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            u64::MAX, 0, u64::MAX,
            3,        3, u64::MAX,
            u64::MAX, 2, u64::MAX,
            u64::MAX, 1, u64::MAX,
            u64::MAX, 0, 0,
            u64::MAX, 0, u64::MAX,
            5,        5, u64::MAX,
            u64::MAX, 4, u64::MAX,
            1,        1, u64::MAX,
            u64::MAX, 0, 0,
            3,        3, u64::MAX,
            u64::MAX, 2, u64::MAX,
            u64::MAX, 1, u64::MAX,
            9,        9, u64::MAX,
            u64::MAX, 8, u64::MAX,
            u64::MAX, 7, u64::MAX,
            0,        0, 0,
            u64::MAX, 0, u64::MAX,
            0,        0, 0,
            u64::MAX, 0, u64::MAX,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//

pub const STOPWATCH_INTERFACES: &[Interface] = &[
    Interface {
        name: "Start",
        description:
            "When an event value arrives here, the timer should start \
             counting up from its current value.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Start",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Stop",
        description:
            "When an event value arrives here, the timer should pause.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Stop",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Reset",
        description:
            "When an event value arrives here, the timer value should be \
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
        name: "Time",
        description: "Should be the current timer value, starting at zero.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Time",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
];

pub struct FabricateStopwatchEval {
    table_values: Vec<u64>,
    start_wire: usize,
    stop_wire: usize,
    reset_wire: usize,
    time_wire: usize,
    time_port: (Coords, Direction),
}

impl FabricateStopwatchEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> FabricateStopwatchEval {
        debug_assert_eq!(slots.len(), 4);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        debug_assert_eq!(slots[3].len(), 1);
        FabricateStopwatchEval {
            table_values: FabricateStopwatchEval::expected_table_values()
                .to_vec(),
            start_wire: slots[0][0].1,
            stop_wire: slots[1][0].1,
            reset_wire: slots[2][0].1,
            time_wire: slots[3][0].1,
            time_port: slots[3][0].0,
        }
    }
}

impl PuzzleEval for FabricateStopwatchEval {
    fn begin_time_step(
        &mut self,
        time_step: u32,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        let expected = FabricateStopwatchEval::expected_table_values();
        let start = (time_step as usize) * 4;
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            if slice[0] <= (u32::MAX as u64) {
                state.send_event(self.start_wire, slice[0] as u32);
            }
            if slice[1] <= (u32::MAX as u64) {
                state.send_event(self.stop_wire, slice[1] as u32);
            }
            if slice[2] <= (u32::MAX as u64) {
                state.send_event(self.reset_wire, slice[2] as u32);
            }
            None
        }
    }

    fn end_time_step(
        &mut self,
        time_step: u32,
        state: &CircuitState,
    ) -> Vec<EvalError> {
        let expected_table = FabricateStopwatchEval::expected_table_values();
        let start = (time_step as usize) * 4;
        if start >= expected_table.len() {
            return vec![];
        }
        let expected_time = expected_table[start + 3];
        let actual_time = state.recv_behavior(self.time_wire).0;

        let mut errors = Vec::new();
        if (actual_time as u64) != expected_time {
            errors.push(EvalError {
                time_step,
                port: Some(self.time_port),
                message: format!(
                    "Expected output time to be {}, but was {}",
                    expected_time, actual_time
                ),
            });
        }
        return errors;
    }
}

impl FabricationEval for FabricateStopwatchEval {
    fn table_column_names() -> &'static [&'static str] {
        &["Start", "Stop", "Reset", "Time"]
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn expected_table_values() -> &'static [u64] {
        &[
            u64::MAX, u64::MAX, u64::MAX, 0,
            0,        u64::MAX, u64::MAX, 0,
            u64::MAX, u64::MAX, u64::MAX, 1,
            u64::MAX, u64::MAX, u64::MAX, 2,
            u64::MAX, 0,        u64::MAX, 3,
            u64::MAX, u64::MAX, u64::MAX, 3,
            u64::MAX, u64::MAX, 0,        0,
            u64::MAX, u64::MAX, u64::MAX, 0,
            0,        u64::MAX, u64::MAX, 0,
            u64::MAX, u64::MAX, u64::MAX, 1,
            u64::MAX, u64::MAX, u64::MAX, 2,
            u64::MAX, u64::MAX, 0,        0,
            u64::MAX, u64::MAX, u64::MAX, 1,
            u64::MAX, u64::MAX, u64::MAX, 2,
            u64::MAX, 0,        u64::MAX, 3,
            u64::MAX, u64::MAX, u64::MAX, 3,
            0,        u64::MAX, 0,        0,
            u64::MAX, u64::MAX, u64::MAX, 1,
            u64::MAX, u64::MAX, u64::MAX, 2,
            u64::MAX, 0,        0,        0,
            u64::MAX, u64::MAX, u64::MAX, 0,
        ]
    }

    fn table_values(&self) -> &[u64] {
        &self.table_values
    }
}

//===========================================================================//
