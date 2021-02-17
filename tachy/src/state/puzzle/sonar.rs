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
use std::collections::VecDeque;

//===========================================================================//

const NUM_WALLS: usize = 100;
const STRAIGHT_SPEED: u32 = 4;
const VEER_SPEED: u32 = 3;
const TRAVEL_SLOWDOWN: u32 = 4 * STRAIGHT_SPEED;
const DISTANCE_TRAVELLED_FOR_VICTORY: u32 =
    (NUM_WALLS as u32) * TRAVEL_SLOWDOWN;
const SONAR_SPEEDUP: u32 = 2;

#[cfg_attr(rustfmt, rustfmt_skip)]
const PORT_WALLS: &[i32; NUM_WALLS] = &[
    -9, -8, -7, -6, -6, -5, -5, -4, -4, -4,
    -3, -3, -2, -2, -1, -1,  0,  0,  1,  1,
     2,  2,  3,  3,  2,  1,  1,  0, -1, -1,
    -2, -2, -3, -4, -5, -5, -6, -6, -7, -7,
    -7, -7, -8, -8, -8, -9, -9, -9, -9, -8,
    -8, -7, -7, -6, -6, -5, -5, -4, -4, -4,
    -4, -4, -3, -3, -2, -2, -1,  0,  1,  1,
     2,  2,  3,  3,  4,  4,  3,  2,  2,  1,
     0, -1, -2, -2, -3, -4, -4, -5, -6, -7,
    -8, -8, -7, -6, -6, -7, -7, -7, -8, -9,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const STBD_WALLS: &[i32; NUM_WALLS] = &[
     9,  9,  8,  8,  7,  7,  6,  6,  6,  5,
     5,  5,  5,  6,  6,  7,  7,  8,  8,  9,
     9,  9,  9,  9,  8,  8,  7,  7,  6,  6,
     5,  5,  4,  4,  3,  3,  2,  2,  1,  1,
     0,  0, -1, -1, -1, -2, -2, -2, -1, -1,
     0,  0,  1,  2,  3,  4,  5,  5,  6,  6,
     6,  6,  7,  8,  8,  8,  9,  9,  9,  9,
     8,  8,  9,  9,  9,  9,  9,  9,  8,  8,
     8,  7,  6,  5,  4,  4,  2,  1,  0, -1,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9,
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Port Sonar Interface",
        description: "Connects to the port-side sonar.",
        side: Direction::West,
        pos: InterfacePosition::Left(1),
        ports: SONAR_PORTS,
    },
    Interface {
        name: "Starboard Sonar Interface",
        description: "Connects to the starboard-side sonar.",
        side: Direction::East,
        pos: InterfacePosition::Right(1),
        ports: SONAR_PORTS,
    },
    Interface {
        name: "Propulsion Interface",
        description: "Connects to the propellers and rudder.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Prop",
            description: "Controls the engine heading.\n    \
                          Send 3 to move straight ahead.\n    \
                          Send 2 to veer to starboard.\n    \
                          Send 1 to veer to port.\n    \
                          Send 0 to come to a stop.",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Two,
        }],
    },
    Interface {
        name: "Instruments Interface",
        description: "Connects to the dead-reckoning positioning system.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Dist",
            description:
                "Sends an event with the total distance travelled, once at \
                 the start of the simulation, and again each time the \
                 distance increments.",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Eight,
        }],
    },
];

const SONAR_PORTS: &[InterfacePort] = &[
    InterfacePort {
        name: "Ping",
        description: "Send an event here to generate a sonar ping.",
        flow: PortFlow::Recv,
        color: PortColor::Event,
        size: WireSize::Zero,
    },
    InterfacePort {
        name: "Echo",
        description:
            "Sends an event when the ping echo is detected.  The more time \
             before the echo arrives, the farther away the canyon wall is on \
             that side.",
        flow: PortFlow::Send,
        color: PortColor::Event,
        size: WireSize::Zero,
    },
];

//===========================================================================//

struct SonarUnit {
    walls: &'static [i32],
    ping_wire: WireId,
    echo_wire: WireId,
    pings: VecDeque<(u32, u32)>,
    num_pending_echoes: u32,
}

impl SonarUnit {
    fn new(
        walls: &'static [i32],
        slots: &[((Coords, Direction), WireId)],
    ) -> SonarUnit {
        debug_assert_eq!(slots.len(), 2);
        SonarUnit {
            walls,
            ping_wire: slots[0].1,
            echo_wire: slots[1].1,
            pings: VecDeque::new(),
            num_pending_echoes: 0,
        }
    }

    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
        horz_position: i32,
        wall_index: usize,
    ) {
        debug_assert!(self.num_pending_echoes == 0);
        if wall_index >= self.walls.len() {
            self.pings.clear();
        } else if let Some(&(time, count)) = self.pings.front() {
            let wall_pos = self.walls[wall_index];
            let wall_dist = (wall_pos - horz_position).abs() as u32;
            if state.time_step() >= time + wall_dist / SONAR_SPEEDUP {
                self.num_pending_echoes = count;
                self.pings.pop_front();
                self.begin_additional_cycle(state);
            }
        }
    }

    fn begin_additional_cycle(&mut self, state: &mut CircuitState) {
        if self.num_pending_echoes > 0 {
            state.send_event(self.echo_wire, 0);
            self.num_pending_echoes -= 1;
        }
    }

    fn end_cycle(
        &mut self,
        state: &CircuitState,
        _errors: &mut Vec<EvalError>,
    ) {
        if state.has_event(self.ping_wire) {
            let time_step = state.time_step();
            match self.pings.back_mut() {
                Some(&mut (time, ref mut count)) if time == time_step => {
                    *count += 1;
                }
                _ => self.pings.push_back((time_step, 1)),
            }
        }
    }

    fn needs_another_cycle(&self) -> bool {
        self.num_pending_echoes > 0
    }
}

//===========================================================================//

pub struct SonarEval {
    port_sonar: SonarUnit,
    stbd_sonar: SonarUnit,
    prop_wire: WireId,
    dist_wire: WireId,
    should_send_dist: bool,
    heading: i32,
    horz_position: i32,
    distance_travelled: u32,
}

impl SonarEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), WireId)>>) -> SonarEval {
        debug_assert_eq!(slots.len(), 4);
        debug_assert_eq!(slots[2].len(), 1);
        debug_assert_eq!(slots[3].len(), 1);
        SonarEval {
            port_sonar: SonarUnit::new(PORT_WALLS, &slots[0]),
            stbd_sonar: SonarUnit::new(STBD_WALLS, &slots[1]),
            prop_wire: slots[2][0].1,
            dist_wire: slots[3][0].1,
            should_send_dist: true,
            heading: 0,
            horz_position: 0,
            distance_travelled: 0,
        }
    }

    pub fn port_walls() -> &'static [i32] {
        PORT_WALLS
    }

    pub fn starboard_walls() -> &'static [i32] {
        STBD_WALLS
    }

    pub fn heading(&self) -> i32 {
        self.heading
    }

    pub fn horz_position(&self) -> i32 {
        self.horz_position
    }

    pub fn distance_travelled(&self) -> f32 {
        (self.distance_travelled as f32) / (TRAVEL_SLOWDOWN as f32)
    }

    fn wall_index(&self) -> usize {
        (self.distance_travelled / TRAVEL_SLOWDOWN) as usize
    }
}

impl PuzzleEval for SonarEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.distance_travelled >= DISTANCE_TRAVELLED_FOR_VICTORY
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        let wall_index = self.wall_index();
        self.port_sonar.begin_time_step(state, self.horz_position, wall_index);
        self.stbd_sonar.begin_time_step(state, self.horz_position, wall_index);
        if self.should_send_dist {
            state.send_event(self.dist_wire, wall_index as u32);
            self.should_send_dist = false;
        }
    }

    fn begin_additional_cycle(&mut self, state: &mut CircuitState) {
        self.port_sonar.begin_additional_cycle(state);
        self.stbd_sonar.begin_additional_cycle(state);
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::new();
        self.port_sonar.end_cycle(state, &mut errors);
        self.stbd_sonar.end_cycle(state, &mut errors);
        errors
    }

    fn needs_another_cycle(&self, _state: &CircuitState) -> bool {
        self.port_sonar.needs_another_cycle()
            || self.stbd_sonar.needs_another_cycle()
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let old_wall_index = self.wall_index();
        let prop = state.recv_behavior(self.prop_wire);
        self.heading = match prop {
            1 => -1,
            2 => 1,
            _ => 0,
        };
        self.horz_position += self.heading;
        if prop != 0 {
            self.distance_travelled +=
                if self.heading == 0 { STRAIGHT_SPEED } else { VEER_SPEED };
        }
        let new_wall_index = self.wall_index();
        if new_wall_index != old_wall_index {
            self.should_send_dist = true;
        }
        let mut errors = Vec::new();
        if new_wall_index < PORT_WALLS.len()
            && PORT_WALLS[new_wall_index] >= self.horz_position
        {
            let msg = format!("Crashed into the port-side canyon wall.");
            errors.push(state.fatal_error(msg));
        }
        if new_wall_index < STBD_WALLS.len()
            && STBD_WALLS[new_wall_index] <= self.horz_position
        {
            let msg = format!("Crashed into the starboard-side canyon wall.");
            errors.push(state.fatal_error(msg));
        }
        errors
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{PORT_WALLS, STBD_WALLS};

    #[test]
    fn walls_do_not_get_too_close() {
        for (index, (&port, &stbd)) in
            PORT_WALLS.iter().zip(STBD_WALLS.iter()).enumerate()
        {
            assert!(
                port + 4 < stbd,
                "At index {}, port wall is at {} and stbd wall is at {}",
                index,
                port,
                stbd
            );
        }
    }
}

//===========================================================================//
