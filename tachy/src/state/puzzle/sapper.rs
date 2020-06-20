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
use crate::geom::{AsFloat, Coords, Direction};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow};
use cgmath::{Deg, Point2};

//===========================================================================//

const NUM_SECTIONS: usize = 4;
const INIT_POSITION: Point2<i32> = Point2::new(8, 7);
const INIT_DIRECTION: Direction = Direction::North;
const TIME_PER_MOVE: u32 = 3;
const TIME_PER_TURN: u32 = 2;
const MAZE_SIZE: usize = 16;

#[cfg_attr(rustfmt, rustfmt_skip)]
const MAZE: [[i8; MAZE_SIZE]; MAZE_SIZE] = [
    [0,  4,  0,  4,  4,  4,  0,  4,  1,  1,  1,  1,  1,  1,  1,  0],
    [4, -4,  4,  0,  0,  0,  4,  1,  0,  0,  1,  0,  0,  0, -1,  1],
    [4,  0,  0,  0,  4,  0,  0,  4,  1,  0,  1,  0,  1,  1,  1,  0],
    [0,  4,  4,  4,  4,  4,  0,  4,  1,  0,  1,  0,  0,  1,  0,  1],
    [4,  0,  0,  0,  0,  0,  0,  4,  0,  0,  0,  1,  0,  1,  0,  1],
    [4,  4,  4,  0,  4,  4,  4,  1,  0,  1,  0,  0,  0,  1,  0,  1],
    [4,  0,  0,  0,  0,  0,  0,  4,  0,  1,  1,  1,  0,  0,  0,  1],
    [0,  4,  4,  4,  4,  4,  0,  0, -0,  0,  0,  2,  1,  1,  1,  0],
    [0,  3,  3,  3,  3,  3,  4,  3,  0,  2,  0,  0,  2,  2,  2,  0],
    [3,  0,  0,  0,  3,  0,  3,  0,  0,  3,  2,  0,  0,  0,  0,  2],
    [3,  0,  3,  0,  0,  0,  3,  0,  3,  2,  0,  0,  2,  2,  0,  2],
    [3,  0,  0,  3,  0,  3,  0,  0,  3,  2,  2,  2,  2,  0,  0,  2],
    [3,  0,  3,  0,  0,  3,  0,  3,  3,  2,  0,  0,  0,  0,  2,  0],
    [3,  0,  3,  0,  3,  0,  0,  0,  3,  2,  2,  0,  2,  2, -2,  2],
    [3, -3,  3,  0,  0,  0,  3,  0,  3,  2,  0,  0,  0,  0,  0,  2],
    [0,  3,  0,  3,  3,  3,  0,  3,  0,  0,  2,  2,  2,  2,  2,  0],
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Scanner Interface",
        description: "Connects to the drone's forward sensors.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Scan",
            description:
                "Indicates if an object is detected directly in front of \
                 the drone:\n    \
                 2 if a control satellite is detected.\n    \
                 1 if a mine is detected.\n    \
                 0 if nothing is detected.",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Two,
        }],
    },
    Interface {
        name: "Navigation Interface",
        description: "Connects to the drone's navigation sensors.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Face",
                description:
                    "Outputs the drone's current direction: 0 for north, 1 \
                     for east, 2 for south, or 3 for west.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
            InterfacePort {
                name: "XPos",
                description: "Outputs the drone's current X position (0-15).",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "YPos",
                description: "Outputs the drone's current Y position (0-15).",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Engine Interface",
        description: "Connects to the ship's defensive beam weapon.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Ready",
                description:
                    "Sends an event at the start of the simulation, and \
                     whenever the drone stops moving or turning.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Move",
                description:
                    "Send an event here to move the drone forward one space.  \
                     It is an error to try to move while the drone is still \
                     moving or turning.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Turn",
                description:
                    "Send 1 here to turn the drone to starboard (clockwise), \
                     or 0 to turn the drone to port (counterclockwise).  It \
                     is an error to try to turn while the drone is still \
                     moving or turning.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::One,
            },
        ],
    },
];

//===========================================================================//

#[derive(Clone, Copy, Debug)]
enum EngineMovement {
    Stationary,
    MovingForward(u32),
    TurningCw(u32),
    TurningCcw(u32),
}

//===========================================================================//

pub struct SapperEval {
    scan_wire: usize,
    face_wire: usize,
    xpos_wire: usize,
    ypos_wire: usize,
    ready_wire: usize,
    move_port: (Coords, Direction),
    move_wire: usize,
    turn_port: (Coords, Direction),
    turn_wire: usize,
    position: Point2<i32>,
    direction: Direction,
    engine_movement: EngineMovement,
    sections_armed: [bool; NUM_SECTIONS],
    ready: bool,
}

impl SapperEval {
    pub const NUM_SECTIONS: usize = NUM_SECTIONS;

    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> SapperEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 3);
        debug_assert_eq!(slots[2].len(), 3);
        SapperEval {
            scan_wire: slots[0][0].1,
            face_wire: slots[1][0].1,
            xpos_wire: slots[1][1].1,
            ypos_wire: slots[1][2].1,
            ready_wire: slots[2][0].1,
            move_port: slots[2][1].0,
            move_wire: slots[2][1].1,
            turn_port: slots[2][2].0,
            turn_wire: slots[2][2].1,
            position: INIT_POSITION,
            direction: INIT_DIRECTION,
            engine_movement: EngineMovement::Stationary,
            sections_armed: [false; NUM_SECTIONS],
            ready: true,
        }
    }

    pub fn maze_cell(position: Point2<i32>) -> i8 {
        if (position.x < 0 || position.x >= (MAZE_SIZE as i32))
            || (position.y < 0 || position.y >= (MAZE_SIZE as i32))
        {
            return 0;
        }
        MAZE[position.y as usize][position.x as usize]
    }

    pub fn initial_sapper_position() -> Point2<f32> {
        INIT_POSITION.as_f32()
    }

    pub fn sapper_position(&self) -> Point2<f32> {
        let offset = match self.engine_movement {
            EngineMovement::MovingForward(t) => {
                ((TIME_PER_MOVE - t) as f32) / (TIME_PER_MOVE as f32)
            }
            _ => 0.0,
        };
        self.position.as_f32() + self.direction.delta().as_f32() * offset
    }

    pub fn initial_sapper_angle() -> Deg<f32> {
        INIT_DIRECTION.angle_from_east()
    }

    pub fn sapper_angle(&self) -> Deg<f32> {
        let offset = match self.engine_movement {
            EngineMovement::TurningCcw(t) => {
                let frac =
                    ((TIME_PER_TURN - t) as f32) / (TIME_PER_TURN as f32);
                Deg(frac * -90.0)
            }
            EngineMovement::TurningCw(t) => {
                let frac =
                    ((TIME_PER_TURN - t) as f32) / (TIME_PER_TURN as f32);
                Deg(frac * 90.0)
            }
            _ => Deg(0.0),
        };
        self.direction.angle_from_east() + offset
    }

    pub fn sections_armed(&self) -> [bool; NUM_SECTIONS] {
        self.sections_armed
    }

    fn scan_value(&self, position: Point2<i32>) -> u32 {
        let cell = SapperEval::maze_cell(position);
        return if cell > 0 {
            let section = (cell - 1) as usize;
            debug_assert!(section < NUM_SECTIONS);
            if self.sections_armed[section] {
                0
            } else {
                1
            }
        } else if cell < 0 {
            let section = (-cell - 1) as usize;
            debug_assert!(section < NUM_SECTIONS);
            if self.sections_armed[section] {
                0
            } else {
                2
            }
        } else {
            0
        };
    }
}

impl PuzzleEval for SapperEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.sections_armed == [true; NUM_SECTIONS]
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        let ahead = self.position + self.direction;
        state.send_behavior(self.scan_wire, self.scan_value(ahead));

        let face = match self.direction {
            Direction::North => 0,
            Direction::East => 1,
            Direction::South => 2,
            Direction::West => 3,
        };
        state.send_behavior(self.face_wire, face);

        state.send_behavior(self.xpos_wire, self.position.x as u32);
        state.send_behavior(
            self.ypos_wire,
            ((MAZE_SIZE as i32) - self.position.y - 1) as u32,
        );

        if self.ready {
            state.send_event(self.ready_wire, 0);
            self.ready = false;
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if state.has_event(self.move_wire) {
            if let EngineMovement::Stationary = self.engine_movement {
                self.engine_movement =
                    EngineMovement::MovingForward(TIME_PER_MOVE);
            } else {
                let message =
                    format!("Cannot move drone while it is still moving.");
                errors.push(state.fatal_port_error(self.move_port, message));
            }
        }
        if let Some(dir) = state.recv_event(self.turn_wire) {
            if let EngineMovement::Stationary = self.engine_movement {
                self.engine_movement = if dir == 0 {
                    EngineMovement::TurningCcw(TIME_PER_TURN)
                } else {
                    EngineMovement::TurningCw(TIME_PER_TURN)
                };
            } else {
                let message =
                    format!("Cannot turn drone while it is still moving.");
                errors.push(state.fatal_port_error(self.turn_port, message));
            }
        }
        errors
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        match self.engine_movement {
            EngineMovement::Stationary => {}
            EngineMovement::MovingForward(time) => {
                self.engine_movement = if time > 1 {
                    EngineMovement::MovingForward(time - 1)
                } else {
                    self.position = self.position + self.direction;
                    let cell = SapperEval::maze_cell(self.position);
                    if cell > 0 {
                        let message = format!("Hit a mine!");
                        errors.push(state.fatal_error(message));
                    } else if cell < 0 {
                        let section = (-cell - 1) as usize;
                        debug_assert!(section < NUM_SECTIONS);
                        self.sections_armed[section] = true;
                    }
                    self.ready = true;
                    EngineMovement::Stationary
                };
            }
            EngineMovement::TurningCw(time) => {
                self.engine_movement = if time > 1 {
                    EngineMovement::TurningCw(time - 1)
                } else {
                    self.direction = self.direction.rotate_cw();
                    self.ready = true;
                    EngineMovement::Stationary
                };
            }
            EngineMovement::TurningCcw(time) => {
                self.engine_movement = if time > 1 {
                    EngineMovement::TurningCcw(time - 1)
                } else {
                    self.direction = self.direction.rotate_ccw();
                    self.ready = true;
                    EngineMovement::Stationary
                };
            }
        }
        errors
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{SapperEval, INIT_POSITION, MAZE, NUM_SECTIONS};

    #[test]
    fn maze_entries_are_valid() {
        let mut found_controller = [false; NUM_SECTIONS];
        for row in MAZE.iter() {
            for &cell in row.iter() {
                assert!(cell <= (NUM_SECTIONS as i8));
                assert!(cell >= -(NUM_SECTIONS as i8));
                if cell < 0 {
                    let section = (-cell - 1) as usize;
                    assert!(!found_controller[section]);
                    found_controller[section] = true;
                }
            }
        }
        assert_eq!(found_controller, [true; NUM_SECTIONS]);
    }

    #[test]
    fn initial_position_is_empty() {
        assert_eq!(SapperEval::maze_cell(INIT_POSITION), 0);
    }
}

//===========================================================================//
