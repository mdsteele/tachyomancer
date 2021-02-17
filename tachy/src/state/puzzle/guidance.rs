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

const MIN_X_POS: u32 = 1;
const MAX_X_POS: u32 = 9;
const INIT_TORP_X_POS: u32 = (MIN_X_POS + MAX_X_POS) / 2;
const SENSOR_RANGE: i32 = 12;
const GOAL_DIST: i32 = 240;

// Each enemy is a (dist, x_pos) pair.
const ENEMIES: &[(i32, u32)] = &[
    (12, 3),
    (17, 2),
    (17, 7),
    (19, 6),
    // TODO: more enemies
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Radar Interface",
        description: "Connects to the torpedo's radar receiver.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Enemy",
            description:
                "When an enemy figher is detected, sends an event with its \
                 X-position (1-9).",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Port Thruster Interface",
        description: "Connects to the torpedo's port-side turning thruster.",
        side: Direction::South,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Port",
            description:
                "Set this to 1 to engage the torpedo's port-side thruster, \
                 which will push the torpedo towards starboard, increasing \
                 its X-position.",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Gyro Interface",
        description: "Connects to the torpedo's dead-reckoning gyro.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "XPos",
            description: "Outputs the torpedo's current X-position (1-9).",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Starboard Thruster Interface",
        description:
            "Connects to the torpedo's starboard-side turning thruster.",
        side: Direction::South,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Stbd",
            description:
                "Set this to 1 to engage the torpedo's starboard-side \
                 thruster, which will push the torpedo towards port, \
                 decreasing its X-position.",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
];

//===========================================================================//

pub struct GuidanceEval {
    enemy_wire: WireId,
    port_wire: WireId,
    xpos_wire: WireId,
    stbd_wire: WireId,
    torp_x_pos: u32,
    dist_travelled: i32,
    num_enemies_signaled: usize,
}

impl GuidanceEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> GuidanceEval {
        debug_assert_eq!(slots.len(), 4);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        debug_assert_eq!(slots[3].len(), 1);
        GuidanceEval {
            enemy_wire: slots[0][0].1,
            port_wire: slots[1][0].1,
            xpos_wire: slots[2][0].1,
            stbd_wire: slots[3][0].1,
            torp_x_pos: INIT_TORP_X_POS,
            dist_travelled: 0,
            num_enemies_signaled: 0,
        }
    }

    pub fn enemies() -> &'static [(i32, u32)] {
        ENEMIES
    }

    pub fn init_torp_x_pos() -> u32 {
        INIT_TORP_X_POS
    }

    pub fn torp_x_pos(&self) -> u32 {
        self.torp_x_pos
    }

    pub fn distance_travelled(&self) -> i32 {
        self.dist_travelled
    }

    fn should_signal_next_enemy(&self) -> bool {
        self.num_enemies_signaled < ENEMIES.len()
            && ENEMIES[self.num_enemies_signaled].0
                <= self.dist_travelled + SENSOR_RANGE
    }

    fn signal_next_enemy_if_needed(&mut self, state: &mut CircuitState) {
        if self.should_signal_next_enemy() {
            let x_pos = ENEMIES[self.num_enemies_signaled].1;
            state.send_event(self.enemy_wire, x_pos);
            self.num_enemies_signaled += 1;
        }
    }
}

impl PuzzleEval for GuidanceEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.dist_travelled >= GOAL_DIST
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        debug_assert!(self.torp_x_pos >= MIN_X_POS);
        debug_assert!(self.torp_x_pos <= MAX_X_POS);
        state.send_behavior(self.xpos_wire, self.torp_x_pos);
        self.signal_next_enemy_if_needed(state)
    }

    fn begin_additional_cycle(&mut self, state: &mut CircuitState) {
        self.signal_next_enemy_if_needed(state)
    }

    fn needs_another_cycle(&self, _state: &CircuitState) -> bool {
        self.should_signal_next_enemy()
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        let port = state.recv_behavior(self.port_wire) != 0;
        let stbd = state.recv_behavior(self.stbd_wire) != 0;
        if port && !stbd {
            if self.torp_x_pos < MAX_X_POS {
                self.torp_x_pos += 1;
            }
        } else if stbd && !port {
            if self.torp_x_pos > MIN_X_POS {
                self.torp_x_pos -= 1;
            }
        }
        self.dist_travelled += 1;
        for &(dist, x_pos) in ENEMIES {
            if self.dist_travelled >= dist - 1
                && self.dist_travelled <= dist + 1
                && self.torp_x_pos >= x_pos - 1
                && self.torp_x_pos <= x_pos + 1
            {
                let msg = format!("Torpdeo was shot down by enemy fighter.");
                errors.push(state.fatal_error(msg));
            }
        }
        errors
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{ENEMIES, GOAL_DIST, MAX_X_POS, MIN_X_POS, SENSOR_RANGE};

    #[test]
    fn enemies_list_is_valid() {
        let mut last_dist = SENSOR_RANGE;
        for &(dist, x_pos) in ENEMIES {
            assert!(x_pos >= MIN_X_POS);
            assert!(x_pos <= MAX_X_POS);
            assert!(dist < GOAL_DIST);
            assert!(dist >= last_dist);
            last_dist = dist;
        }
    }
}

//===========================================================================//
