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
const DEGREES_PER_TIME: u32 = 360 / (NUM_POSITIONS * TIME_PER_TURN);

const TIME_PER_TURN: u32 = 3;
const TIME_PER_MANIPULATE: u32 = 4;
const TIME_BETWEEN_COMMANDS: u32 = 9;

const COMMANDS: &[u32] = &[3, 7, 5, 2, 0, 4, 1, 6, 3, 7, 5, 1, 6, 2];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Radio Interface",
        description: "Connects to a radio antenna.",
        side: Direction::West,
        pos: InterfacePosition::Right(1),
        ports: &[
            InterfacePort {
                name: "Recv",
                description:
                    "Connects to the radio receiver.  Sends an event when a \
                     radio command arrives.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Xmit",
                description:
                    "Connects to the radio transmitter.  Signal here when \
                     the command is completed.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
    Interface {
        name: "Arm Interface",
        description:
            "\
             Connects to the sensors and servo motors of the robot arm.",
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
                name: "Rotate",
                description: "Send 1 to rotate clockwise, 0 to rotate \
                              counterclockwise.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::One,
            },
            InterfacePort {
                name: "Manip",
                description:
                    "Signal here to manipulate at the current position.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Done",
                description:
                    "Signals when the the robot arm has finished moving/\
                     manipulating.",
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
    Manipulating(u32),
}

//===========================================================================//

pub struct RobotArmEval {
    recv_wire: usize,
    xmit_port: (Coords, Direction),
    xmit_wire: usize,
    pos_wire: usize,
    turn_wire: usize,
    manip_port: (Coords, Direction),
    manip_wire: usize,
    done_wire: usize,
    motor_movement: MotorMovement,
    movement_is_done: bool,
    current_position: u32,
    current_position_degrees: u32,
    time_to_next_command: Option<u32>,
    last_command: u32,
    num_commands_sent: usize,
    has_completed_command: bool,
    has_sent_radio_reply: bool,
}

impl RobotArmEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> RobotArmEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 4);
        RobotArmEval {
            recv_wire: slots[0][0].1,
            xmit_port: slots[0][1].0,
            xmit_wire: slots[0][1].1,
            pos_wire: slots[1][0].1,
            turn_wire: slots[1][1].1,
            manip_port: slots[1][2].0,
            manip_wire: slots[1][2].1,
            done_wire: slots[1][3].1,
            motor_movement: MotorMovement::Stationary,
            movement_is_done: false,
            current_position: 0,
            current_position_degrees: 0,
            time_to_next_command: Some(0),
            last_command: 0,
            num_commands_sent: 0,
            has_completed_command: false,
            has_sent_radio_reply: false,
        }
    }

    pub fn arm_angle(&self) -> u32 {
        self.current_position_degrees
    }

    pub fn arm_extension(&self) -> f32 {
        match self.motor_movement {
            MotorMovement::Manipulating(3) => 0.5,
            MotorMovement::Manipulating(2) => 1.0,
            MotorMovement::Manipulating(1) => 0.5,
            _ => 0.0,
        }
    }

    pub fn station_manipulation(&self) -> Option<(u32, f32)> {
        match self.motor_movement {
            MotorMovement::Manipulating(2) => {
                Some((self.current_position, 0.333))
            }
            MotorMovement::Manipulating(1) => {
                Some((self.current_position, 0.667))
            }
            _ => None,
        }
    }

    pub fn current_command(&self) -> Option<u32> {
        if self.has_completed_command || self.time_to_next_command.is_some() {
            None
        } else {
            Some(self.last_command)
        }
    }
}

impl PuzzleEval for RobotArmEval {
    fn seconds_per_time_step(&self) -> f64 {
        0.075
    }

    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        if self.time_to_next_command == Some(0) {
            self.time_to_next_command = None;
            if self.num_commands_sent >= COMMANDS.len() {
                return Some(EvalScore::Value(state.time_step()));
            }
            self.last_command = COMMANDS[self.num_commands_sent];
            self.num_commands_sent += 1;
            self.has_completed_command = false;
            self.has_sent_radio_reply = false;
            state.send_event(self.recv_wire, self.last_command);
        }

        state.send_behavior(self.pos_wire, self.current_position);
        if self.movement_is_done {
            state.send_event(self.done_wire, 0);
            self.movement_is_done = false;
        }

        return None;
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if state.recv_event(self.xmit_wire).is_some() {
            if !self.has_completed_command {
                let message = format!(
                    "Sent radio reply without first completing the instructed \
                     manipulation."
                );
                errors.push(state.fatal_port_error(self.xmit_port, message));
            } else if self.has_sent_radio_reply {
                let message = format!(
                    "Sent more than one radio reply for the same command."
                );
                errors.push(state.fatal_port_error(self.xmit_port, message));
            } else {
                self.has_sent_radio_reply = true;
            }
        }
        if let MotorMovement::Stationary = self.motor_movement {
            if let Some(dir) = state.recv_event(self.turn_wire) {
                self.motor_movement = if dir == 0 {
                    MotorMovement::TurningCcw(TIME_PER_TURN)
                } else {
                    MotorMovement::TurningCw(TIME_PER_TURN)
                };
            } else if state.recv_event(self.manip_wire).is_some() {
                if self.current_position != self.last_command {
                    let message = format!(
                        "Manipulated position {}, but last \
                         command was for position {}.",
                        self.current_position, self.last_command
                    );
                    errors.push(
                        state.fatal_port_error(self.manip_port, message),
                    );
                } else if self.has_completed_command {
                    errors.push(state.fatal_port_error(
                        self.manip_port,
                        format!("Already performed manipulation."),
                    ));
                }
                self.motor_movement =
                    MotorMovement::Manipulating(TIME_PER_MANIPULATE);
            }
        }
        errors
    }

    fn end_time_step(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        if self.has_sent_radio_reply && self.time_to_next_command.is_none() {
            self.time_to_next_command = Some(TIME_BETWEEN_COMMANDS);
        }
        match self.motor_movement {
            MotorMovement::Stationary => {}
            MotorMovement::TurningCw(time) => {
                self.current_position_degrees =
                    (self.current_position_degrees + DEGREES_PER_TIME) % 360;
                self.motor_movement = if time > 1 {
                    MotorMovement::TurningCw(time - 1)
                } else {
                    self.movement_is_done = true;
                    MotorMovement::Stationary
                };
            }
            MotorMovement::TurningCcw(time) => {
                self.current_position_degrees =
                    (self.current_position_degrees + (360 - DEGREES_PER_TIME))
                        % 360;
                self.motor_movement = if time > 1 {
                    MotorMovement::TurningCcw(time - 1)
                } else {
                    self.movement_is_done = true;
                    MotorMovement::Stationary
                };
            }
            MotorMovement::Manipulating(time) => {
                self.motor_movement = if time > 1 {
                    MotorMovement::Manipulating(time - 1)
                } else {
                    if self.current_position == self.last_command {
                        self.has_completed_command = true;
                    }
                    self.movement_is_done = true;
                    MotorMovement::Stationary
                };
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
