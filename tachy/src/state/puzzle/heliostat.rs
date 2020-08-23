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
use num_integer::mod_floor;

//===========================================================================//

const ENERGY_INITIAL_VALUE: u32 = 1000;
const ENERGY_NEEDED_FOR_VICTORY: u32 = 5000;
const ENERGY_DRAIN_PER_TIME_STEP: u32 = 30;
const ENERGY_MAX_GEN_PER_TIME_STEP: u32 = 100;
const ORBIT_DEGREES_PER_TIME_STEP: i32 = 5;

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Sensor Interface",
        description:
            "Connects to a subspace sensor array that determines the ideal \
             position for the heliostat mirror.  Use the motor interface to \
             move the mirror to this position.",
        side: Direction::West,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "Goal",
                description: "Outputs the ideal mirror position.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Power",
                description:
                    "Outputs the current power generation efficiency, from 0 \
                     to 100 percent.  (You can ignore this port if you don't \
                     need it.)",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
        ],
    },
    Interface {
        name: "Motor Interface",
        description:
            "Connects to a stepper motor that controls the position of the \
             heliostat mirror.",
        side: Direction::East,
        pos: InterfacePosition::Right(0),
        ports: &[
            InterfacePort {
                name: "Pos",
                description: "Outputs the current mirror position.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Motor",
                description: "Receives motor commands.\n    \
                              Send 1 to move clockwise.\n    \
                              Send 2 to move counterclockwise.\n  \
                              Send any other value to not move.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
        ],
    },
];

//===========================================================================//

pub struct HeliostatEval {
    goal_wire: WireId,
    efficiency_wire: WireId,
    pos_wire: WireId,
    motor_wire: WireId,
    current_goal: u32,
    current_pos: u32,
    current_efficiency: u32,
    current_orbit_degrees: i32,
    energy: u32,
}

impl HeliostatEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> HeliostatEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 2);
        HeliostatEval {
            goal_wire: slots[0][0].1,
            efficiency_wire: slots[0][1].1,
            pos_wire: slots[1][0].1,
            motor_wire: slots[1][1].1,
            current_goal: 0,
            current_pos: 3,
            current_efficiency: 0,
            current_orbit_degrees: 0,
            energy: ENERGY_INITIAL_VALUE,
        }
    }

    pub fn current_energy(&self) -> u32 {
        self.energy
    }

    pub fn current_goal(&self) -> u32 {
        self.current_goal
    }

    pub fn current_position(&self) -> u32 {
        self.current_pos
    }

    pub fn current_efficiency(&self) -> u32 {
        self.current_efficiency
    }

    pub fn current_orbit_degrees(&self) -> i32 {
        self.current_orbit_degrees
    }
}

impl PuzzleEval for HeliostatEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.energy >= ENERGY_NEEDED_FOR_VICTORY
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        let is_in_shadow = self.current_orbit_degrees >= 135
            && self.current_orbit_degrees <= 225;
        self.current_goal = if is_in_shadow {
            self.current_pos
        } else {
            let turns = (-self.current_orbit_degrees as f64) / 360.0;
            let signed_pos = (16.0 * turns).round() as i32;
            mod_floor(signed_pos, 16) as u32
        };
        self.current_efficiency = if is_in_shadow {
            0
        } else {
            let delta =
                ((self.current_pos as i32) - (self.current_goal as i32)).abs();
            let theta = (delta as f64) * std::f64::consts::FRAC_PI_8;
            let efficiency = (4.0 + 96.0 * 0.5 * (theta.cos() + 1.0)).round();
            debug_assert!(efficiency >= 0.0);
            debug_assert!(efficiency <= 100.0);
            efficiency as u32
        };

        state.send_behavior(self.goal_wire, self.current_goal);
        state.send_behavior(self.efficiency_wire, self.current_efficiency);
        state.send_behavior(self.pos_wire, self.current_pos);
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        self.energy +=
            (ENERGY_MAX_GEN_PER_TIME_STEP * self.current_efficiency) / 100;
        self.energy = self.energy.saturating_sub(ENERGY_DRAIN_PER_TIME_STEP);
        match state.recv_behavior(self.motor_wire) {
            0x1 => self.current_pos = (self.current_pos + 1) % 16,
            0x2 => self.current_pos = (self.current_pos + 15) % 16,
            _ => {}
        }
        self.current_orbit_degrees =
            (self.current_orbit_degrees + ORBIT_DEGREES_PER_TIME_STEP) % 360;
        Vec::new()
    }
}

//===========================================================================//
