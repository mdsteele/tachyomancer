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

use super::super::eval::{CircuitState, EvalError, EvalScore, PuzzleEval};
use super::super::interface::{Interface, InterfacePort, InterfacePosition};
use crate::geom::{Coords, Direction};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow};

//===========================================================================//

const NUM_POSITIONS: u32 = 8;
const DEGREES_PER_POSITION: u32 = 360 / NUM_POSITIONS;
const RADAR_DEGREES_PER_TIME: u32 = 45;
const CANNON_COOLDOWN_TIME: u32 = 30;
const TIME_PER_TURRET_TURN: u32 = 3;
const TURRET_DEGREES_PER_TIME: u32 =
    360 / (NUM_POSITIONS * TIME_PER_TURRET_TURN);
const ENEMY_START_DIST: u32 = 255;
const BASE_DAMAGE_FOR_FAILURE: u32 = 5;

// (time_step, position, speed)
const ENEMIES: &[(u32, u32, u32)] = &[
    (0, 5, 2),
    (10, 3, 3),
    (60, 6, 2),
    (80, 2, 3),
    (90, 7, 3),
    (100, 0, 4),
    // TODO: add more enemies for CommandTurret puzzle
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Radar Interface",
        description: "Connects to the base's radar dish.",
        side: Direction::West,
        pos: InterfacePosition::Left(1),
        ports: &[
            InterfacePort {
                name: "Dir",
                description:
                    "Indicates which direction the radar dish is currently \
                     pointing (0-7).",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Dist",
                description:
                    "Sends an event each time step with the distance to the \
                     nearest enemy in the direction of the radar dish, if \
                     any.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
        ],
    },
    Interface {
        name: "Cannon Interface",
        description: "Connects to the pulse cannon mounted on the turret.",
        side: Direction::East,
        pos: InterfacePosition::Right(1),
        ports: &[
            InterfacePort {
                name: "Fire",
                description:
                    "Fires the cannon in the direction the turret is \
                     currently facing.  It is an error to try to fire when \
                     the cannon is not ready.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Loaded",
                description: "Signals when the cannon is ready to fire.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
    Interface {
        name: "Turret Interface",
        description: "Connects to the motor on the turret base.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Face",
                description:
                    "Indicates the direction the turret is currently facing \
                     (0-7).",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Rotate",
                description:
                    "Send 1 to rotate the turret clockwise, 0 to rotate \
                     counterclockwise.  It is an error to try to rotate while \
                     the turret is still moving.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::One,
            },
            InterfacePort {
                name: "Done",
                description: "Signals when the turret has finished moving.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
];

//===========================================================================//

#[derive(Clone, Copy, Debug)]
enum MotorMovement {
    Stationary,
    TurningCw(u32),
    TurningCcw(u32),
}

//===========================================================================//

pub struct TurretEval {
    dir_wire: usize,
    dist_wire: usize,
    fire_port: (Coords, Direction),
    fire_wire: usize,
    loaded_wire: usize,
    face_wire: usize,
    rotate_port: (Coords, Direction),
    rotate_wire: usize,
    done_wire: usize,
    radar_position: u32,
    radar_position_degrees: u32,
    cannon_cooldown: u32,
    cannon_is_loaded: bool,
    motor_movement: MotorMovement,
    turret_position: u32,
    turret_position_degrees: u32,
    movement_is_done: bool,
    enemies: Vec<(u32, u32, u32)>,
    num_enemies_appeared: usize,
    base_damage: u32,
}

impl TurretEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> TurretEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 2);
        debug_assert_eq!(slots[2].len(), 3);
        TurretEval {
            dir_wire: slots[0][0].1,
            dist_wire: slots[0][1].1,
            fire_port: slots[1][0].0,
            fire_wire: slots[1][0].1,
            loaded_wire: slots[1][1].1,
            face_wire: slots[2][0].1,
            rotate_port: slots[2][1].0,
            rotate_wire: slots[2][1].1,
            done_wire: slots[2][2].1,
            radar_position: 0,
            radar_position_degrees: 0,
            cannon_cooldown: 0,
            cannon_is_loaded: true,
            motor_movement: MotorMovement::Stationary,
            turret_position: 0,
            turret_position_degrees: 0,
            movement_is_done: false,
            enemies: Vec::new(),
            num_enemies_appeared: 0,
            base_damage: 0,
        }
    }

    pub fn turret_position(&self) -> u32 {
        self.turret_position
    }

    pub fn turret_angle(&self) -> u32 {
        self.turret_position_degrees
    }

    pub fn cannon_cooldown(&self) -> u32 {
        self.cannon_cooldown
    }

    pub fn base_damage(&self) -> u32 {
        self.base_damage
    }

    /// Returns list of (position, distance, speed) tuples for enemies.
    pub fn enemies(&self) -> &[(u32, u32, u32)] {
        &self.enemies
    }

    fn closest_enemy(&self, position: u32) -> Option<(usize, u32)> {
        let mut closest_dist = ENEMY_START_DIST;
        let mut closest_index: Option<usize> = None;
        for (index, &(pos, dist, _)) in self.enemies.iter().enumerate() {
            if pos == position && dist <= closest_dist {
                closest_index = Some(index);
                closest_dist = dist;
            }
        }
        closest_index.map(|index| (index, closest_dist))
    }
}

impl PuzzleEval for TurretEval {
    fn seconds_per_time_step(&self) -> f64 {
        0.05
    }

    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        while self.num_enemies_appeared < ENEMIES.len()
            && ENEMIES[self.num_enemies_appeared].0 <= state.time_step()
        {
            let (_, position, speed) = ENEMIES[self.num_enemies_appeared];
            self.enemies.push((position, ENEMY_START_DIST, speed));
            self.num_enemies_appeared += 1;
        }
        state.send_behavior(self.dir_wire, self.radar_position);
        if let Some((_, dist)) = self.closest_enemy(self.radar_position) {
            state.send_event(self.dist_wire, dist);
        }
        state.send_behavior(self.face_wire, self.turret_position);
        if self.cannon_is_loaded {
            state.send_event(self.loaded_wire, 0);
            self.cannon_is_loaded = false;
        }
        if self.movement_is_done {
            state.send_event(self.done_wire, 0);
            self.movement_is_done = false;
        }
        if self.num_enemies_appeared == ENEMIES.len()
            && self.enemies.is_empty()
        {
            Some(EvalScore::Value(state.time_step()))
        } else {
            None
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if let Some(dir) = state.recv_event(self.rotate_wire) {
            if let MotorMovement::Stationary = self.motor_movement {
                self.motor_movement = if dir == 0 {
                    MotorMovement::TurningCcw(TIME_PER_TURRET_TURN)
                } else {
                    MotorMovement::TurningCw(TIME_PER_TURRET_TURN)
                };
            } else {
                let message =
                    format!("Cannot rotate turret while it is still moving.");
                errors.push(state.fatal_port_error(self.rotate_port, message));
            }
        }
        if state.has_event(self.fire_wire) {
            if self.cannon_cooldown == 0 {
                self.cannon_cooldown = CANNON_COOLDOWN_TIME;
                if let Some((index, _)) =
                    self.closest_enemy(self.turret_position)
                {
                    self.enemies.remove(index);
                }
            } else {
                let message = format!(
                    "Cannot fire cannon while it is still \
                     cooling down."
                );
                errors.push(state.fatal_port_error(self.fire_port, message));
            }
        }
        errors
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        // Turret:
        match self.motor_movement {
            MotorMovement::Stationary => {}
            MotorMovement::TurningCw(time) => {
                self.turret_position_degrees = (self.turret_position_degrees
                    + TURRET_DEGREES_PER_TIME)
                    % 360;
                self.motor_movement = if time > 1 {
                    MotorMovement::TurningCw(time - 1)
                } else {
                    self.movement_is_done = true;
                    MotorMovement::Stationary
                };
            }
            MotorMovement::TurningCcw(time) => {
                self.turret_position_degrees = (self.turret_position_degrees
                    + (360 - TURRET_DEGREES_PER_TIME))
                    % 360;
                self.motor_movement = if time > 1 {
                    MotorMovement::TurningCcw(time - 1)
                } else {
                    self.movement_is_done = true;
                    MotorMovement::Stationary
                };
            }
        }
        self.turret_position =
            div_round(self.turret_position_degrees, DEGREES_PER_POSITION)
                % NUM_POSITIONS;
        // Cannon:
        if self.cannon_cooldown > 0 {
            self.cannon_cooldown -= 1;
            if self.cannon_cooldown == 0 {
                self.cannon_is_loaded = true;
            }
        }
        // Radar:
        self.radar_position_degrees =
            (self.radar_position_degrees + RADAR_DEGREES_PER_TIME) % 360;
        self.radar_position =
            div_round(self.radar_position_degrees, DEGREES_PER_POSITION)
                % NUM_POSITIONS;
        // Enemies:
        for &mut (_, ref mut dist, speed) in self.enemies.iter_mut() {
            *dist = dist.saturating_sub(speed);
        }
        let mut damage = 0;
        self.enemies.retain(|&(_, dist, _)| {
            if dist == 0 {
                damage += 1;
            }
            dist != 0
        });
        self.base_damage += damage;
        // Base:
        if self.base_damage >= BASE_DAMAGE_FOR_FAILURE {
            let message = format!("Base has taken too much damage");
            errors.push(state.fatal_error(message));
        }
        errors
    }
}

//===========================================================================//

fn div_round(a: u32, b: u32) -> u32 {
    ((a as f64) / (b as f64)).round() as u32
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{BASE_DAMAGE_FOR_FAILURE, ENEMIES, NUM_POSITIONS};

    #[test]
    fn enemies_list_is_valid() {
        assert!(ENEMIES.len() >= (BASE_DAMAGE_FOR_FAILURE as usize));
        let mut prev_time_step = 0;
        for &(time_step, pos, speed) in ENEMIES.iter() {
            assert!(time_step >= prev_time_step);
            assert!(pos < NUM_POSITIONS);
            assert!(speed > 0);
            prev_time_step = time_step;
        }
    }
}

//===========================================================================//
