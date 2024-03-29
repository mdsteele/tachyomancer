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

use super::super::eval::{CircuitState, EvalError, PuzzleEval};
use super::super::interface::{Interface, InterfacePort, InterfacePosition};
use crate::geom::{Coords, Direction};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow, WireId};

//===========================================================================//

const DRIFT_FACTOR: f64 = 0.2;
const IMBALANCE_BASE: f64 = 1.1;
const NUM_RODS: usize = 3;
const TARGETS: &[u32] =
    &[7, 9, 1, 3, 6, 4, 0, 8, 5, 2, 1, 7, 4, 2, 8, 0, 9, 6, 3, 5];
const TARGET_HOLD_TIME: i32 = 5;

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Thermostat Interface",
        description:
            "Connects to sensors in the ship's power grid that determine the \
             current and desired power outputs of the backup reactor (from 0 \
             to 9).",
        side: Direction::West,
        pos: InterfacePosition::Left(1),
        ports: &[
            InterfacePort {
                name: "Power",
                description: "Indicates the current power output.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Target",
                description: "Indicates the desired power output.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Control Rod Interface",
        description:
            "Connects to an array of actuators that move the reactor's three \
             control rods.  Send higher values to retract a rod (increasing \
             the total power output), or lower values to extend a rod \
             (decreasing the total power).",
        side: Direction::East,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "Rod1",
                description: "",
                flow: PortFlow::Sink,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
            InterfacePort {
                name: "Rod2",
                description: "",
                flow: PortFlow::Sink,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
            InterfacePort {
                name: "Rod3",
                description: "",
                flow: PortFlow::Sink,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
        ],
    },
];

const_assert_eq!(INTERFACES[1].ports.len(), NUM_RODS);

//===========================================================================//

pub struct ReactorEval {
    power_wire: WireId,
    target_wire: WireId,
    rod_wires: Vec<WireId>,
    rod_values: Vec<u32>,
    current_power: f64,
    current_target: u32,
    held_target_for: i32,
    num_targets_held: usize,
}

impl ReactorEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), WireId)>>) -> ReactorEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), NUM_RODS);
        ReactorEval {
            power_wire: slots[0][0].1,
            target_wire: slots[0][1].1,
            rod_wires: slots[1].iter().map(|&(_, wire)| wire).collect(),
            rod_values: vec![0; NUM_RODS],
            current_power: 0.0,
            current_target: TARGETS[0],
            held_target_for: 0,
            num_targets_held: 0,
        }
    }

    pub fn current_power(&self) -> u32 {
        self.current_power.round() as u32
    }

    pub fn target_power(&self) -> u32 {
        self.current_target
    }

    pub fn rod_values(&self) -> &[u32] {
        &self.rod_values
    }
}

impl PuzzleEval for ReactorEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.num_targets_held >= TARGETS.len()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        let current_power = self.current_power();
        if current_power == self.current_target {
            self.held_target_for += 1;
            if self.held_target_for >= TARGET_HOLD_TIME {
                self.held_target_for = 0;
                self.num_targets_held += 1;
                if self.num_targets_held < TARGETS.len() {
                    self.current_target = TARGETS[self.num_targets_held];
                }
            }
        } else {
            self.held_target_for = 0;
        }
        state.send_behavior(self.power_wire, current_power);
        state.send_behavior(self.target_wire, self.current_target);
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        self.rod_values = self
            .rod_wires
            .iter()
            .map(|&rod_wire| state.recv_behavior(rod_wire))
            .collect();
        let rod_total: u32 = self.rod_values.iter().copied().sum();
        let rod_average: f64 =
            (rod_total as f64) / (self.rod_values.len() as f64);
        let rod_imbalance: f64 = self
            .rod_values
            .iter()
            .map(|&value| (rod_average - (value as f64)).powi(2))
            .sum();
        debug_assert!(rod_imbalance >= 0.0);
        let imbalance_factor: f64 = IMBALANCE_BASE.powf(-rod_imbalance);
        debug_assert!(imbalance_factor > 0.0 && imbalance_factor <= 1.0);
        let power_delta: f64 = (rod_total as f64) - self.current_power;
        self.current_power =
            self.current_power + DRIFT_FACTOR * imbalance_factor * power_delta;
        Vec::new()
    }
}

//===========================================================================//
