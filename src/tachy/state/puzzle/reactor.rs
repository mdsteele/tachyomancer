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
        description: "\
            Connects to sensors in the ship's power grid that determine the \
            current and desired power outputs of the backup reactor (from 0 \
            to 9).",
        side: Direction::West,
        pos: InterfacePosition::Left(1),
        ports: &[
            InterfacePort {
                name: "Power",
                description: "Indicates the current power output.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Target",
                description: "Indicates the desired power output.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Control Rod Interface",
        description: "\
            Connects to an array of actuators that move the reactor's three \
            control rods.  Send higher values to retract a rod (increasing \
            the total power output), or lower values to extend a rod \
            (decreasing the total power).",
        side: Direction::East,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "Rod1",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
            InterfacePort {
                name: "Rod2",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
            InterfacePort {
                name: "Rod3",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
        ],
    },
];

//===========================================================================//

pub struct AutomateReactorEval {
    verification: [u64; 5],
    power_wire: usize,
    target_wire: usize,
    rod_wires: Vec<usize>,
    current_power: f64,
    current_target: u32,
    held_target_for: i32,
    num_targets_held: usize,
}

impl AutomateReactorEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>)
               -> AutomateReactorEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), NUM_RODS);
        AutomateReactorEval {
            verification: [0; 5],
            power_wire: slots[0][0].1,
            target_wire: slots[0][1].1,
            rod_wires: slots[1].iter().map(|&(_, wire)| wire).collect(),
            current_power: 0.0,
            current_target: TARGETS[0],
            held_target_for: 0,
            num_targets_held: 0,
        }
    }
}

impl PuzzleEval for AutomateReactorEval {
    fn verification_data(&self) -> &[u64] { &self.verification }

    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        let current_power = self.current_power.round() as u32;
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

        self.verification[3] = current_power as u64;
        self.verification[4] = self.current_target as u64;

        state.send_behavior(self.power_wire, current_power);
        state.send_behavior(self.target_wire, self.current_target);

        if self.num_targets_held >= TARGETS.len() {
            Some(EvalScore::Value(time_step as i32))
        } else {
            None
        }
    }

    fn end_time_step(&mut self, _time_step: u32, state: &CircuitState)
                     -> Vec<EvalError> {
        let rod_values: Vec<u32> = self.rod_wires
            .iter()
            .map(|&rod_wire| state.recv_behavior(rod_wire).0)
            .collect();
        for (index, &value) in rod_values.iter().enumerate() {
            self.verification[index] = value as u64;
        }
        let rod_total: u32 = rod_values.iter().cloned().sum();
        let rod_average: f64 = (rod_total as f64) / (rod_values.len() as f64);
        let rod_imbalance: f64 = rod_values
            .iter()
            .map(|&value| (rod_average - (value as f64)).powi(2))
            .sum();
        debug_assert!(rod_imbalance >= 0.0);
        let imbalance_factor: f64 = IMBALANCE_BASE.powf(-rod_imbalance);
        debug_assert!(imbalance_factor > 0.0 && imbalance_factor <= 1.0);
        let power_delta: f64 = (rod_total as f64) - self.current_power;
        self.current_power = self.current_power +
            DRIFT_FACTOR * imbalance_factor * power_delta;
        Vec::new()
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{INTERFACES, NUM_RODS};

    #[test]
    fn num_rods() {
        assert_eq!(NUM_RODS, INTERFACES[1].ports.len());
    }
}

//===========================================================================//
