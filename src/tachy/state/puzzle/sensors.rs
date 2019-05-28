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
use tachy::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Sensor Interface",
        description: "\
            Connects to the raw data from the fore and aft sensor arrays.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Fore",
                description: "",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Aft",
                description: "",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Data Interface",
        description: "Connects processed sensor data to the main computer.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Max",
                description: "\
                    This should be equal to the greater of the Fore and Aft \
                    values.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Avg",
                description: "\
                    This should be equal to the average of the Fore and Aft \
                    values, rounded down.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Min",
                description: "\
                    This should be equal to the lesser of the Fore and Aft \
                    values.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
];

//===========================================================================//

pub struct SensorsEval {
    table_values: Vec<u64>,
    fore_wire: usize,
    aft_wire: usize,
    max_port: (Coords, Direction),
    max_wire: usize,
    avg_port: (Coords, Direction),
    avg_wire: usize,
    min_port: (Coords, Direction),
    min_wire: usize,
}

impl SensorsEval {
    pub const TABLE_COLUMN_NAMES: &'static [&'static str] =
        &["Fore", "Aft", "Min", "Avg", "Max"];

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub const EXPECTED_TABLE_VALUES: &'static [u64] = &[
        4,  10, 4,  7,  10,
        7,  2,  2,  4,  7,
        0,  5,  0,  2,  5,
        9,  9,  9,  9,  9,
        15, 14, 14, 14, 15,
        1,  3,  1,  2,  3,
        // TODO add more table rows
    ];

    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> SensorsEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 3);
        SensorsEval {
            table_values: SensorsEval::EXPECTED_TABLE_VALUES.to_vec(),
            fore_wire: slots[0][0].1,
            aft_wire: slots[0][1].1,
            max_port: slots[1][0].0,
            max_wire: slots[1][0].1,
            avg_port: slots[1][1].0,
            avg_wire: slots[1][1].1,
            min_port: slots[1][2].0,
            min_wire: slots[1][2].1,
        }
    }

    pub fn table_values(&self) -> &[u64] { &self.table_values }
}

impl PuzzleEval for SensorsEval {
    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        let expected = SensorsEval::EXPECTED_TABLE_VALUES;
        let start = (time_step as usize) *
            SensorsEval::TABLE_COLUMN_NAMES.len();
        if start >= expected.len() {
            Some(EvalScore::WireLength)
        } else {
            let slice = &expected[start..];
            state.send_behavior(self.fore_wire, slice[0] as u32);
            state.send_behavior(self.aft_wire, slice[1] as u32);
            None
        }
    }

    fn end_time_step(&mut self, time_step: u32, state: &CircuitState)
                     -> Vec<EvalError> {
        let fore = state.recv_behavior(self.fore_wire).0;
        let aft = state.recv_behavior(self.aft_wire).0;
        let expected_max = fore.max(aft);
        let expected_avg = (fore + aft) / 2;
        let expected_min = fore.min(aft);
        let actual_max = state.recv_behavior(self.max_wire).0;
        let actual_avg = state.recv_behavior(self.avg_wire).0;
        let actual_min = state.recv_behavior(self.min_wire).0;
        self.table_values[5 * (time_step as usize) + 2] = actual_min as u64;
        self.table_values[5 * (time_step as usize) + 3] = actual_avg as u64;
        self.table_values[5 * (time_step as usize) + 4] = actual_max as u64;
        let mut errors = Vec::<EvalError>::new();
        if actual_min != expected_min {
            let error = EvalError {
                time_step,
                port: Some(self.min_port),
                message: format!("Expected Min={} for inputs {} and {}, \
                                  but got Min={}",
                                 expected_min,
                                 fore,
                                 aft,
                                 actual_min),
            };
            errors.push(error);
        }
        if actual_avg != expected_avg {
            let error = EvalError {
                time_step,
                port: Some(self.avg_port),
                message: format!("Expected Avg={} for inputs {} and {}, \
                                  but got Avg={}",
                                 expected_avg,
                                 fore,
                                 aft,
                                 actual_avg),
            };
            errors.push(error);
        }
        if actual_max != expected_max {
            let error = EvalError {
                time_step,
                port: Some(self.max_port),
                message: format!("Expected Max={} for inputs {} and {}, \
                                  but got Max={}",
                                 expected_max,
                                 fore,
                                 aft,
                                 actual_max),
            };
            errors.push(error);
        }
        errors
    }
}

//===========================================================================//
