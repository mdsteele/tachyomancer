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
use super::rng::SimpleRng;
use crate::geom::{Coords, Direction};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow, WireId};
use cgmath::Point2;
use num_integer::Roots;

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Sensor Interface",
        description:
            "Connects to a subspace sensor array that determines the ideal \
             position for the beacon dish.  Use the motor interface to move \
             the dish to this position.",
        side: Direction::South,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "XGoal",
                description: "Outputs ideal X position.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "YGoal",
                description: "Outputs ideal Y position.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Motor Interface",
        description:
            "Connects to a stepper motor that controls the position of the \
             beacon mirror.",
        side: Direction::South,
        pos: InterfacePosition::Right(0),
        ports: &[
            InterfacePort {
                name: "XPos",
                description: "Outputs current X position.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "YPos",
                description: "Outputs current Y position.",
                flow: PortFlow::Source,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Motor",
                description: "Receives motor commands.\n    \
                              Send 8 to move up.\n    \
                              Send 4 to move down.\n    \
                              Send 2 to move left.\n    \
                              Send 1 to move right.\n  \
                              Send any other value to not move.",
                flow: PortFlow::Sink,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
];

//===========================================================================//

pub struct BeaconEval {
    opt_x_wire: WireId,
    opt_y_wire: WireId,
    pos_x_wire: WireId,
    pos_y_wire: WireId,
    motor_wire: WireId,
    current_opt: Point2<u32>,
    current_pos: Point2<u32>,
    energy: u32,
    rng: SimpleRng,
}

impl BeaconEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), WireId)>>) -> BeaconEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 3);
        BeaconEval {
            opt_x_wire: slots[0][0].1,
            opt_y_wire: slots[0][1].1,
            pos_x_wire: slots[1][0].1,
            pos_y_wire: slots[1][1].1,
            motor_wire: slots[1][2].1,
            current_opt: Point2::new(3, 7),
            current_pos: Point2::new(15, 15),
            energy: 1000,
            rng: SimpleRng::new(0x4f3173b1f817227f),
        }
    }

    pub fn current_energy(&self) -> u32 {
        self.energy
    }

    pub fn current_optimum(&self) -> Point2<u32> {
        self.current_opt
    }

    pub fn current_position(&self) -> Point2<u32> {
        self.current_pos
    }
}

impl PuzzleEval for BeaconEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.energy >= 5000
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        if (state.time_step() % 20) == 0 {
            let x = self.rng.rand_u4();
            let y = self.rng.rand_u4();
            self.current_opt = Point2::new(x, y);
        }
        state.send_behavior(self.opt_x_wire, self.current_opt.x);
        state.send_behavior(self.opt_y_wire, self.current_opt.y);
        state.send_behavior(self.pos_x_wire, self.current_pos.x);
        state.send_behavior(self.pos_y_wire, self.current_pos.y);
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let delta = 10
            * Point2::new(
                self.current_pos.x as i32 - self.current_opt.x as i32,
                self.current_pos.y as i32 - self.current_opt.y as i32,
            );
        let dist = (delta.x * delta.x + delta.y * delta.y).sqrt() as u32;
        self.energy += 85;
        self.energy = self.energy.saturating_sub(dist);
        match state.recv_behavior(self.motor_wire) {
            0x8 if self.current_pos.y < 0xf => self.current_pos.y += 1,
            0x4 if self.current_pos.y > 0x0 => self.current_pos.y -= 1,
            0x2 if self.current_pos.x > 0x0 => self.current_pos.x -= 1,
            0x1 if self.current_pos.x < 0xf => self.current_pos.x += 1,
            _ => {}
        }
        Vec::new()
    }
}

//===========================================================================//
