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

const BEAM_COOLDOWN: u32 = 20;
const BEAM_HITS_FOR_VICTORY: u32 = 7;
const ENEMY_DAMAGE_FOR_VICTORY: u32 = BEAM_HITS_FOR_VICTORY * BEAM_COOLDOWN;
const ENEMY_DIST_LOWER_BOUND: u32 = 180;
const ENEMY_DIST_VARIATION: u32 = 45;
const ENEMY_EXPLOSION_FOR_VICTORY: u32 = 20;
const INITIAL_SHIELD_POWER: u32 = 30;
const SHIP_DAMAGE_FOR_FAILURE: u32 = 4;

// (time_step, speed)
const TORPEDOES: &[(u32, u32)] = &[
    (2, 9),
    (15, 9),
    (28, 9),
    // gap
    (60, 12),
    (70, 12),
    // gap
    (100, 9),
    (115, 8),
    (130, 14),
    // gap
    (160, 7),
    (170, 15),
    // gap
    (205, 10),
    (215, 10),
    (225, 10),
    (235, 10),
    // gap
    (275, 15),
    (280, 15),
    // gap
    (315, 10),
    (328, 11),
    (341, 12),
    // gap
    (375, 9),
    (385, 9),
    // gap
    (420, 16),
    (425, 7),
    (430, 13),
    // gap
    (460, 7),
    (470, 15),
    // gap
    (500, 10),
    (510, 10),
    (520, 10),
    (530, 10),
    // gap
    (565, 17),
    (570, 17),
    // gap
    (605, 10),
    (617, 11),
    (629, 12),
];

fn enemy_dist_for_time_step(time_step: u32) -> u32 {
    let modulus = 2 * (ENEMY_DIST_VARIATION - 1);
    let remainder = time_step % modulus;
    let variation =
        if 2 * remainder <= modulus { remainder } else { modulus - remainder };
    ENEMY_DIST_LOWER_BOUND + variation
}

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Sensors Interface",
        description: "Connects to the ship's tactical sensors.",
        side: Direction::North,
        pos: InterfacePosition::Right(0),
        ports: &[
            InterfacePort {
                name: "Dist",
                description:
                    "Indicates the current distance to the enemy ship, in \
                     kilometers.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Torp",
                description:
                    "When the enemy ship fires a torpedo, this sends an event \
                     with the speed of that torpedo, in kilometers per time \
                     step.",
                flow: PortFlow::Source,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
        ],
    },
    Interface {
        name: "Beam Interface",
        description: "Connects to the ship's defensive beam weapon.",
        side: Direction::North,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Fire",
            description:
                "Fires a continuous beam at the enemy ship for 20 time \
                 steps.  While the beam is firing, shields cannot be \
                 raised.  It is an error to try to fire the beam again \
                 while it is still firing.",
            flow: PortFlow::Sink,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Shields Interface",
        description: "Connects to the ship's deflector shields.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Power",
                description:
                    "Indicates the remaining shield power.  This decreases by \
                     1 each time step that the shields are up.  When this \
                     reaches zero, the shields will no longer function.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Raise",
                description:
                    "Controls whether the shields are up (1) or down (0).  \
                     When the shields are up, enemy torpedoes will be \
                     neutralized.  It is an error for the shields to be up \
                     while the ship's beam weapon is firing.",
                flow: PortFlow::Sink,
                color: PortColor::Behavior,
                size: WireSize::One,
            },
        ],
    },
];

//===========================================================================//

pub struct ShieldsEval {
    dist_wire: WireId,
    torp_wire: WireId,
    fire_port: (Coords, Direction),
    fire_wire: WireId,
    power_wire: WireId,
    raise_port: (Coords, Direction),
    raise_wire: WireId,
    torpedoes: Vec<(u32, u32)>, // (dist, speed)
    num_torpedoes_fired: usize,
    beam_cooldown: u32,
    shield_power: u32,
    shield_is_up: bool,
    ship_damage: u32,
    enemy_damage: u32,
    enemy_dist: u32,
    enemy_explosion: u32,
}

impl ShieldsEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), WireId)>>) -> ShieldsEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 2);
        ShieldsEval {
            dist_wire: slots[0][0].1,
            torp_wire: slots[0][1].1,
            fire_port: slots[1][0].0,
            fire_wire: slots[1][0].1,
            power_wire: slots[2][0].1,
            raise_port: slots[2][1].0,
            raise_wire: slots[2][1].1,
            torpedoes: Vec::new(),
            num_torpedoes_fired: 0,
            beam_cooldown: 0,
            shield_power: INITIAL_SHIELD_POWER,
            shield_is_up: false,
            ship_damage: 0,
            enemy_damage: 0,
            enemy_dist: enemy_dist_for_time_step(0),
            enemy_explosion: 0,
        }
    }

    pub fn enemy_max_health() -> f32 {
        BEAM_HITS_FOR_VICTORY as f32
    }

    pub fn enemy_health(&self) -> f32 {
        (BEAM_HITS_FOR_VICTORY as f32)
            - (self.enemy_damage as f32) / (BEAM_COOLDOWN as f32)
    }

    pub fn initial_enemy_distance() -> u32 {
        enemy_dist_for_time_step(0)
    }

    pub fn enemy_distance(&self) -> u32 {
        self.enemy_dist
    }

    pub fn enemy_explosion(&self) -> f32 {
        (self.enemy_explosion.min(ENEMY_EXPLOSION_FOR_VICTORY) as f32)
            / (ENEMY_EXPLOSION_FOR_VICTORY as f32)
    }

    pub fn ship_max_health() -> f32 {
        SHIP_DAMAGE_FOR_FAILURE as f32
    }

    pub fn ship_health(&self) -> f32 {
        (SHIP_DAMAGE_FOR_FAILURE - self.ship_damage) as f32
    }

    pub fn initial_shield_power() -> u32 {
        INITIAL_SHIELD_POWER
    }

    pub fn shield_power(&self) -> u32 {
        self.shield_power
    }

    pub fn shield_is_up(&self) -> bool {
        self.shield_is_up
    }

    pub fn beam_cooldown(&self) -> u32 {
        self.beam_cooldown
    }

    pub fn torpedoes(&self) -> &[(u32, u32)] {
        &self.torpedoes
    }
}

impl PuzzleEval for ShieldsEval {
    fn seconds_per_time_step(&self) -> f64 {
        0.05
    }

    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.enemy_explosion >= ENEMY_EXPLOSION_FOR_VICTORY
            && self.torpedoes.is_empty()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        self.enemy_dist = enemy_dist_for_time_step(state.time_step());
        state.send_behavior(self.dist_wire, self.enemy_dist);
        state.send_behavior(self.power_wire, self.shield_power);
        if self.enemy_damage < ENEMY_DAMAGE_FOR_VICTORY {
            while self.num_torpedoes_fired < TORPEDOES.len()
                && TORPEDOES[self.num_torpedoes_fired].0 <= state.time_step()
            {
                let (_, speed) = TORPEDOES[self.num_torpedoes_fired];
                self.torpedoes.push((self.enemy_dist, speed));
                state.send_event(self.torp_wire, speed);
                self.num_torpedoes_fired += 1;
            }
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if state.has_event(self.fire_wire) {
            if self.beam_cooldown > 0 {
                let message =
                    format!("Cannot fire beam while it is still firing");
                errors.push(state.fatal_port_error(self.fire_port, message));
            } else if self.enemy_damage < ENEMY_DAMAGE_FOR_VICTORY {
                self.beam_cooldown = BEAM_COOLDOWN;
            }
        }
        errors
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if self.enemy_damage >= ENEMY_DAMAGE_FOR_VICTORY {
            self.enemy_explosion += 1;
        }
        // Shields:
        let mut shields_up = false;
        if state.recv_behavior(self.raise_wire) != 0 {
            if self.beam_cooldown > 0 {
                let message =
                    format!("Cannot raise shields while beam is firing");
                errors.push(state.fatal_port_error(self.raise_port, message));
            } else if self.shield_power > 0 {
                self.shield_power -= 1;
                shields_up = true;
            }
        }
        self.shield_is_up = shields_up;
        // Torpedoes:
        for &mut (ref mut dist, speed) in self.torpedoes.iter_mut() {
            *dist = dist.saturating_sub(speed);
        }
        let mut damage = 0;
        self.torpedoes.retain(|&(dist, _)| {
            if dist == 0 && !shields_up {
                damage += 1;
            }
            dist != 0
        });
        self.ship_damage += damage;
        // Ship:
        if self.ship_damage >= SHIP_DAMAGE_FOR_FAILURE {
            let message = format!("Ship has taken too much damage");
            errors.push(state.fatal_error(message));
        }
        // Beam:
        if self.beam_cooldown > 0 {
            self.enemy_damage += 1;
            self.beam_cooldown -= 1;
        }
        errors
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{
        BEAM_COOLDOWN, INITIAL_SHIELD_POWER, INTERFACES,
        SHIP_DAMAGE_FOR_FAILURE, TORPEDOES,
    };

    #[test]
    fn beam_cooldown_description() {
        // Test that the description for the beam fire port gives the correct
        // cooldown time.
        let interface = &INTERFACES[1];
        assert_eq!(interface.name, "Beam Interface");
        let port = &interface.ports[0];
        assert_eq!(port.name, "Fire");
        let expected = format!("for {} time steps", BEAM_COOLDOWN);
        assert!(port.description.contains(&expected));
    }

    #[test]
    fn torpedoes_list_is_valid() {
        let mut prev_time_step = 0;
        for &(time_step, speed) in TORPEDOES.iter() {
            assert!(time_step >= prev_time_step);
            assert!(speed > 0);
            prev_time_step = time_step;
        }
        let num_torpedoes_needed = (INITIAL_SHIELD_POWER as usize)
            + (SHIP_DAMAGE_FOR_FAILURE as usize);
        assert!(
            TORPEDOES.len() >= num_torpedoes_needed,
            "num_torpedoes={}, num_torpedoes_needed={}",
            TORPEDOES.len(),
            num_torpedoes_needed
        );
    }
}

//===========================================================================//
