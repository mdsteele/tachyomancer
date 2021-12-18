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

const GRAVITY: f64 = 0.04;
const ACCEL_PER_THRUST: f64 = GRAVITY / 15.0;

const INIT_ALTITUDE: f64 = 250.0;
const INIT_VELOCITY: f64 = -1.0;
const INIT_ANGLE: i32 = 90;
const INIT_FUEL: u32 = 3000;
const FUEL_FACTOR: u32 = 15;
const_assert!(INIT_FUEL <= FUEL_FACTOR * 255);

const MAX_LANDING_SPEED: f64 = 2.0;
const MIN_LANDING_ANGLE: i32 = 85;
const MAX_LANDING_ANGLE: i32 = 95;

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Instruments Interface",
        description: "Connects to the lander's instruments panel.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Alt",
                description: "Indicates the current altitude.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Angle",
                description:
                    "Indicates the current descent angle, in degrees \
                     (0-180).  90 is vertical.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Fuel",
                description:
                    "Indicates how much fuel is remaining.  When this reaches \
                     zero, the thrusters will stop working.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
        ],
    },
    Interface {
        name: "Thruster Interface",
        description:
            "Connects to the lander's descent thrusters.  Increase the value \
             for each side to increase thrust, at the cost of more fuel.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Port",
                description: "Controls the port thrusters.",
                flow: PortFlow::Sink,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Stbd",
                description: "Controls the starboard thrusters.",
                flow: PortFlow::Sink,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
];

//===========================================================================//

pub struct LanderEval {
    alt_wire: WireId,
    angle_wire: WireId,
    fuel_wire: WireId,
    port_wire: WireId,
    stbd_wire: WireId,
    current_altitude: f64,
    current_velocity: f64,
    current_angle: i32,
    current_fuel: u32,
    port_thrust: u32,
    stbd_thrust: u32,
}

impl LanderEval {
    pub const INITIAL_ALTITUDE: f64 = INIT_ALTITUDE;

    pub fn new(slots: Vec<Vec<((Coords, Direction), WireId)>>) -> LanderEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 3);
        debug_assert_eq!(slots[1].len(), 2);
        LanderEval {
            alt_wire: slots[0][0].1,
            angle_wire: slots[0][1].1,
            fuel_wire: slots[0][2].1,
            port_wire: slots[1][0].1,
            stbd_wire: slots[1][1].1,
            current_altitude: INIT_ALTITUDE,
            current_velocity: INIT_VELOCITY,
            current_angle: INIT_ANGLE,
            current_fuel: INIT_FUEL,
            port_thrust: 0,
            stbd_thrust: 0,
        }
    }

    pub fn lander_altitude(&self) -> f64 {
        self.current_altitude
    }

    pub fn lander_angle_from_vertical(&self) -> i32 {
        self.current_angle - 90
    }

    pub fn fuel(&self) -> f32 {
        (self.current_fuel as f32) / (INIT_FUEL as f32)
    }

    pub fn port_thrust(&self) -> f32 {
        (self.port_thrust as f32) / 15.0
    }

    pub fn stbd_thrust(&self) -> f32 {
        (self.stbd_thrust as f32) / 15.0
    }
}

impl PuzzleEval for LanderEval {
    fn seconds_per_time_step(&self) -> f64 {
        0.05
    }

    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.current_altitude <= 0.0
            && self.port_thrust == 0
            && self.stbd_thrust == 0
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        debug_assert!(self.current_altitude >= 0.0);
        let altitude = self.current_altitude.ceil() as u32;
        debug_assert!(altitude <= 255);
        state.send_behavior(self.alt_wire, altitude);

        debug_assert!(self.current_angle >= 0);
        debug_assert!(self.current_angle <= 180);
        state.send_behavior(self.angle_wire, self.current_angle as u32);

        let fuel = (self.current_fuel + FUEL_FACTOR - 1) / FUEL_FACTOR;
        debug_assert!(fuel <= 255);
        state.send_behavior(self.fuel_wire, fuel);
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let port_thrust = state.recv_behavior(self.port_wire);
        let stbd_thrust = state.recv_behavior(self.stbd_wire);
        let (port_thrust, stbd_thrust) =
            limit_thrust(port_thrust, stbd_thrust, self.current_fuel);
        debug_assert!(port_thrust + stbd_thrust <= self.current_fuel);
        self.current_fuel -= port_thrust + stbd_thrust;
        self.port_thrust = port_thrust;
        self.stbd_thrust = stbd_thrust;

        let current_wind = wind_at_altitude(self.current_altitude);

        self.current_altitude += self.current_velocity;
        self.current_altitude =
            self.current_altitude.max(0.0).min(INIT_ALTITUDE);

        self.current_velocity -= GRAVITY;
        self.current_velocity += ((port_thrust + stbd_thrust) as f64)
            * ACCEL_PER_THRUST
            * (self.current_angle as f64).to_radians().sin();

        self.current_angle += (port_thrust as i32) - (stbd_thrust as i32);
        self.current_angle += current_wind;
        self.current_angle = self.current_angle.max(0).min(180);

        let mut errors = Vec::new();
        if self.current_altitude <= 0.0 {
            if -self.current_velocity > MAX_LANDING_SPEED {
                errors.push(state.fatal_error(format!(
                    "Landed at too high a speed ({} d/t is \
                     above the safe limit of {} d/t).",
                    self.current_velocity.abs().ceil(),
                    MAX_LANDING_SPEED.floor()
                )));
            }
            if self.current_angle < MIN_LANDING_ANGLE
                || self.current_angle > MAX_LANDING_ANGLE
            {
                errors.push(state.fatal_error(format!(
                    "Landed at too shallow an angle ({}° is \
                     not in the safe range of {}° to {}°).",
                    self.current_angle, MIN_LANDING_ANGLE, MAX_LANDING_ANGLE
                )));
            }
            self.current_velocity = self.current_velocity.max(0.0);
        }
        errors
    }
}

fn limit_thrust(
    mut port_thrust: u32,
    mut stbd_thrust: u32,
    fuel: u32,
) -> (u32, u32) {
    if port_thrust + stbd_thrust > fuel {
        let mut shortfall = port_thrust + stbd_thrust - fuel;
        let common = port_thrust.min(stbd_thrust).min(shortfall / 2);
        port_thrust -= common;
        stbd_thrust -= common;
        shortfall -= 2 * common;
        if shortfall > 0 {
            if port_thrust == 0 {
                stbd_thrust -= shortfall;
            } else if stbd_thrust == 0 {
                port_thrust -= shortfall;
            } else {
                debug_assert_eq!(shortfall, 1);
                if port_thrust > stbd_thrust {
                    port_thrust -= 1;
                } else {
                    stbd_thrust -= 1;
                }
            }
        }
    }
    (port_thrust, stbd_thrust)
}

fn wind_at_altitude(altitude: f64) -> i32 {
    debug_assert!(altitude >= 0.0);
    ((0.5 * altitude).sqrt().sin() * (0.1 * altitude).cbrt()).round() as i32
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{limit_thrust, wind_at_altitude};

    #[test]
    fn thrust() {
        assert_eq!(limit_thrust(0, 0, 0), (0, 0));
        assert_eq!(limit_thrust(0, 0, 100), (0, 0));
        assert_eq!(limit_thrust(9, 7, 100), (9, 7));

        assert_eq!(limit_thrust(3, 5, 8), (3, 5));
        assert_eq!(limit_thrust(3, 5, 7), (3, 4));
        assert_eq!(limit_thrust(3, 5, 6), (2, 4));
        assert_eq!(limit_thrust(3, 5, 5), (2, 3));
        assert_eq!(limit_thrust(3, 5, 4), (1, 3));
        assert_eq!(limit_thrust(3, 5, 3), (1, 2));
        assert_eq!(limit_thrust(3, 5, 2), (0, 2));
        assert_eq!(limit_thrust(3, 5, 1), (0, 1));
        assert_eq!(limit_thrust(3, 5, 0), (0, 0));

        assert_eq!(limit_thrust(3, 2, 5), (3, 2));
        assert_eq!(limit_thrust(3, 2, 4), (2, 2));
        assert_eq!(limit_thrust(3, 2, 3), (2, 1));
        assert_eq!(limit_thrust(3, 2, 2), (1, 1));
        assert_eq!(limit_thrust(3, 2, 1), (1, 0));
        assert_eq!(limit_thrust(3, 2, 0), (0, 0));
    }

    #[test]
    fn wind() {
        assert_eq!(wind_at_altitude(0.0), 0);
        assert_eq!(wind_at_altitude(10.0), 1);
        assert_eq!(wind_at_altitude(20.0), 0);
        assert_eq!(wind_at_altitude(30.0), -1);
        assert_eq!(wind_at_altitude(50.0), -2);
        assert_eq!(wind_at_altitude(70.0), -1);
        assert_eq!(wind_at_altitude(80.0), 0);
        assert_eq!(wind_at_altitude(90.0), 1);
        assert_eq!(wind_at_altitude(120.0), 2);
        assert_eq!(wind_at_altitude(160.0), 1);
        assert_eq!(wind_at_altitude(180.0), 0);
        assert_eq!(wind_at_altitude(190.0), -1);
        assert_eq!(wind_at_altitude(210.0), -2);
        assert_eq!(wind_at_altitude(240.0), -3);
    }
}

//===========================================================================//
