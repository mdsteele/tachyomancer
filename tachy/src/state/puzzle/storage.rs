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
use crate::state::{PortColor, PortFlow};
use std::mem;

//===========================================================================//

const LOADING_DOCK_POSITION: usize = 0;
const NUM_POSITIONS: u32 = 8;
const DEGREES_PER_POSITION: u32 = 360 / NUM_POSITIONS;
const DEGREES_PER_TIME: u32 = 360 / (NUM_POSITIONS * TIME_PER_TURN);

const TIME_PER_TURN: u32 = 3;
const TIME_PER_EXTEND: u32 = 2;

//===========================================================================//

#[derive(Clone, Copy)]
enum Command {
    Store(u32),
    Retrieve(u32),
}

const DELAYS_AND_COMMANDS: &[(u32, Command)] = &[
    (0, Command::Store(53)),
    (4, Command::Store(29)),
    (9, Command::Retrieve(53)),
    (4, Command::Store(82)),
    (8, Command::Store(7)),
    (8, Command::Store(38)),
    (7, Command::Retrieve(82)),
    (6, Command::Store(14)),
    (7, Command::Retrieve(14)),
    (7, Command::Retrieve(38)),
    (9, Command::Store(46)),
    (7, Command::Store(1)),
    (6, Command::Store(60)),
    (4, Command::Store(32)),
    (7, Command::Store(71)),
    (8, Command::Retrieve(60)),
    (3, Command::Retrieve(1)),
    (3, Command::Retrieve(32)),
    (8, Command::Retrieve(46)),
    (9, Command::Store(99)),
    (7, Command::Retrieve(71)),
    (9, Command::Retrieve(7)),
    (6, Command::Retrieve(99)),
    (5, Command::Retrieve(29)),
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Radio Interface",
        description: "Connects to a radio antenna.",
        side: Direction::West,
        pos: InterfacePosition::Left(1),
        ports: &[
            InterfacePort {
                name: "Recv",
                description:
                    "Connects to the radio receiver.  Sends 0 when a crate is \
                     at the loading dock (position 0) waiting to be stored.  \
                     Sends 1-99 when the crate with that number should be \
                     returned to the loading dock.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Xmit",
                description:
                    "Connects to the radio transmitter.  Signal here when \
                     ready for the next command to be received.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
    Interface {
        name: "Sensor Interface",
        description: "Connects to the sensors on the robot arm.",
        side: Direction::East,
        pos: InterfacePosition::Right(0),
        ports: &[
            InterfacePort {
                name: "Pos",
                description:
                    "Indicates the current position of the arm (0-7).",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Held",
                description:
                    "Indicates the number of the currently-held crate \
                     (1-99), or 0 if the arm isn't holding a crate.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
        ],
    },
    Interface {
        name: "Motor Interface",
        description: "Connects to the servo motors of the robot arm.",
        side: Direction::East,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "Rotate",
                description: "Send 1 to rotate clockwise, or 0 to rotate \
                              counterclockwise.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::One,
            },
            InterfacePort {
                name: "Grab",
                description:
                    "Send 1 to grab a crate from the current position, or 0 \
                     to drop the crate at the current position.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::One,
            },
            InterfacePort {
                name: "Done",
                description:
                    "Signals when the the robot arm has finished moving/\
                     grabbing/dropping.",
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
    Extending(u32),
    Retracting(u32),
}

//===========================================================================//

pub struct StorageDepotEval {
    recv_wire: usize,
    xmit_wire: usize,
    pos_wire: usize,
    held_wire: usize,
    turn_wire: usize,
    grab_port: (Coords, Direction),
    grab_wire: usize,
    done_wire: usize,
    motor_movement: MotorMovement,
    movement_is_done: bool,
    current_position: u32,
    current_position_degrees: u32,
    currently_holding: u32,
    station_crates: [u32; NUM_POSITIONS as usize],
    time_to_next_command: Option<u32>,
    num_commands_sent: usize,
    num_commands_completed: usize,
    has_sent_radio_reply: bool,
}

impl StorageDepotEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> StorageDepotEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 2);
        debug_assert_eq!(slots[2].len(), 3);
        StorageDepotEval {
            recv_wire: slots[0][0].1,
            xmit_wire: slots[0][1].1,
            pos_wire: slots[1][0].1,
            held_wire: slots[1][1].1,
            turn_wire: slots[2][0].1,
            grab_port: slots[2][1].0,
            grab_wire: slots[2][1].1,
            done_wire: slots[2][2].1,
            motor_movement: MotorMovement::Stationary,
            movement_is_done: false,
            current_position: 0,
            current_position_degrees: 0,
            currently_holding: 0,
            station_crates: [0; NUM_POSITIONS as usize],
            time_to_next_command: Some(DELAYS_AND_COMMANDS[0].0),
            num_commands_sent: 0,
            num_commands_completed: 0,
            has_sent_radio_reply: false,
        }
    }

    pub fn arm_angle(&self) -> u32 {
        self.current_position_degrees
    }

    pub fn arm_extension(&self) -> f32 {
        match self.motor_movement {
            MotorMovement::Extending(1) => 0.5,
            MotorMovement::Retracting(2) => 1.0,
            MotorMovement::Retracting(1) => 0.5,
            _ => 0.0,
        }
    }

    pub fn currently_holding(&self) -> u32 {
        self.currently_holding
    }

    pub fn station_crates(&self) -> &[u32] {
        &self.station_crates
    }

    pub fn desired_crate(&self) -> Option<u32> {
        if self.num_commands_completed < DELAYS_AND_COMMANDS.len() {
            match DELAYS_AND_COMMANDS[self.num_commands_completed].1 {
                Command::Store(_) => None,
                Command::Retrieve(crate_id) => {
                    if self.num_commands_sent > self.num_commands_completed {
                        Some(crate_id)
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }
}

impl PuzzleEval for StorageDepotEval {
    fn seconds_per_time_step(&self) -> f64 {
        0.075
    }

    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.num_commands_completed >= DELAYS_AND_COMMANDS.len()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        if self.time_to_next_command == Some(0) {
            match DELAYS_AND_COMMANDS[self.num_commands_sent].1 {
                Command::Store(crate_id) => {
                    if self.station_crates[LOADING_DOCK_POSITION] == 0 {
                        self.station_crates[LOADING_DOCK_POSITION] = crate_id;
                        state.send_event(self.recv_wire, 0);
                        self.num_commands_sent += 1;
                        self.time_to_next_command = None;
                    }
                }
                Command::Retrieve(crate_id) => {
                    state.send_event(self.recv_wire, crate_id);
                    self.num_commands_sent += 1;
                    self.time_to_next_command = None;
                }
            }
        }
        state.send_behavior(self.pos_wire, self.current_position);
        state.send_behavior(self.held_wire, self.currently_holding);
        if self.movement_is_done {
            state.send_event(self.done_wire, 0);
            self.movement_is_done = false;
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if state.recv_event(self.xmit_wire).is_some() {
            self.has_sent_radio_reply = true;
        }
        if let MotorMovement::Stationary = self.motor_movement {
            if let Some(dir) = state.recv_event(self.turn_wire) {
                self.motor_movement = if dir == 0 {
                    MotorMovement::TurningCcw(TIME_PER_TURN)
                } else {
                    MotorMovement::TurningCw(TIME_PER_TURN)
                };
            } else if let Some(dir) = state.recv_event(self.grab_wire) {
                if dir == 0 {
                    if self.currently_holding == 0 {
                        errors.push(state.fatal_port_error(
                            self.grab_port,
                            "Cannot drop; not holding a crate".to_string(),
                        ));
                    } else if self.station_crates
                        [self.current_position as usize]
                        != 0
                    {
                        errors.push(state.fatal_port_error(
                            self.grab_port,
                            "Cannot drop; station is occupied".to_string(),
                        ));
                    } else {
                        self.motor_movement =
                            MotorMovement::Extending(TIME_PER_EXTEND)
                    }
                } else {
                    if self.currently_holding != 0 {
                        errors.push(state.fatal_port_error(
                            self.grab_port,
                            "Cannot grab; already holding a crate".to_string(),
                        ));
                    } else {
                        self.motor_movement =
                            MotorMovement::Extending(TIME_PER_EXTEND)
                    }
                }
            }
        }
        errors
    }

    fn end_time_step(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        if self.has_sent_radio_reply
            && self.time_to_next_command.is_none()
            && self.num_commands_sent < DELAYS_AND_COMMANDS.len()
        {
            self.time_to_next_command =
                Some(DELAYS_AND_COMMANDS[self.num_commands_sent].0);
            self.has_sent_radio_reply = false;
        }
        while self.num_commands_completed < self.num_commands_sent {
            match DELAYS_AND_COMMANDS[self.num_commands_completed].1 {
                Command::Store(_) => {
                    self.num_commands_completed += 1;
                }
                Command::Retrieve(crate_id) => {
                    if self.station_crates[LOADING_DOCK_POSITION] == crate_id {
                        self.station_crates[LOADING_DOCK_POSITION] = 0;
                        self.num_commands_completed += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        match self.motor_movement {
            MotorMovement::Stationary => {}
            MotorMovement::TurningCw(ref mut time) => {
                self.current_position_degrees =
                    (self.current_position_degrees + DEGREES_PER_TIME) % 360;
                *time -= 1;
                if *time == 0 {
                    self.movement_is_done = true;
                    self.motor_movement = MotorMovement::Stationary;
                }
            }
            MotorMovement::TurningCcw(ref mut time) => {
                self.current_position_degrees =
                    (self.current_position_degrees + (360 - DEGREES_PER_TIME))
                        % 360;
                *time -= 1;
                if *time == 0 {
                    self.movement_is_done = true;
                    self.motor_movement = MotorMovement::Stationary;
                }
            }
            MotorMovement::Extending(ref mut time) => {
                *time -= 1;
                if *time == 0 {
                    let index = self.current_position as usize;
                    mem::swap(
                        &mut self.currently_holding,
                        &mut self.station_crates[index],
                    );
                    self.motor_movement =
                        MotorMovement::Retracting(TIME_PER_EXTEND);
                }
            }
            MotorMovement::Retracting(ref mut time) => {
                *time -= 1;
                if *time == 0 {
                    self.movement_is_done = true;
                    self.motor_movement = MotorMovement::Stationary;
                }
            }
        }
        self.current_position =
            div_round(self.current_position_degrees, DEGREES_PER_POSITION)
                % NUM_POSITIONS;
        if let Some(ref mut time) = self.time_to_next_command {
            if *time > 0 {
                *time -= 1;
            }
        }
        Vec::new()
    }
}

//===========================================================================//

fn div_round(a: u32, b: u32) -> u32 {
    ((a as f64) / (b as f64)).round() as u32
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Command, DELAYS_AND_COMMANDS, NUM_POSITIONS};
    use std::collections::HashSet;

    #[test]
    fn command_sequence_is_valid() {
        let mut crates_in_storage = HashSet::<u32>::new();
        let mut all_crates = HashSet::<u32>::new();
        for &(_, command) in DELAYS_AND_COMMANDS.iter() {
            match command {
                Command::Store(crate_id) => {
                    assert!(crate_id >= 1);
                    assert!(crate_id <= 100);
                    assert!(!crates_in_storage.contains(&crate_id));
                    crates_in_storage.insert(crate_id);
                    assert!(!all_crates.contains(&crate_id));
                    all_crates.insert(crate_id);
                }
                Command::Retrieve(crate_id) => {
                    assert!(crates_in_storage.contains(&crate_id));
                    crates_in_storage.remove(&crate_id);
                }
            }
            assert!(crates_in_storage.len() < (NUM_POSITIONS as usize));
        }
        assert!(crates_in_storage.is_empty());
    }
}

//===========================================================================//
