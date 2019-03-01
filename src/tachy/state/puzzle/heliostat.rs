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

use super::iface::{Interface, InterfacePort, InterfacePosition};
use super::rng::SimpleRng;
use super::super::eval::{CircuitState, EvalError, EvalScore, PuzzleEval};
use cgmath::Point2;
use num_integer::Roots;
use tachy::geom::{Coords, Direction};
use tachy::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Sensor Interface",
        description: "\
            Connects to a photosensor array that determines the \
            ideal position for the heliostat mirror.  Use the \
            motor interface to move the mirror to this position.",
        side: Direction::South,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "XGoal",
                description: "Outputs ideal X position.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "YGoal",
                description: "Outputs ideal Y position.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Motor Interface",
        description: "\
            Connects to a stepper motor that controls the \
            position of the heliostat mirror.",
        side: Direction::South,
        pos: InterfacePosition::Right(0),
        ports: &[
            InterfacePort {
                name: "XPos",
                description: "Outputs current X position.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "YPos",
                description: "Outputs current Y position.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Motor",
                description: "\
                    Receives motor commands.\n    \
                    Send 8 to move up.\n    \
                    Send 4 to move down.\n    \
                    Send 2 to move left.\n    \
                    Send 1 to move right.\n  \
                    Send any other value to not move.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
];

//===========================================================================//

pub struct AutomateHeliostatEval {
    verification: [u64; 5],
    opt_x_wire: usize,
    opt_y_wire: usize,
    pos_x_wire: usize,
    pos_y_wire: usize,
    motor_wire: usize,
    current_opt: Point2<u32>,
    current_pos: Point2<u32>,
    energy: u32,
    rng: SimpleRng,
}

impl AutomateHeliostatEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>)
               -> AutomateHeliostatEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 3);
        AutomateHeliostatEval {
            verification: [0; 5],
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

    fn update_verification(&mut self) {
        self.verification[0] = self.current_opt.x as u64;
        self.verification[1] = self.current_opt.y as u64;
        self.verification[2] = self.current_pos.x as u64;
        self.verification[3] = self.current_pos.y as u64;
        self.verification[4] = self.energy as u64;
    }
}

impl PuzzleEval for AutomateHeliostatEval {
    fn verification_data(&self) -> &[u64] { &self.verification }

    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        if (time_step % 20) == 0 {
            let x = self.rng.rand_u4();
            let y = self.rng.rand_u4();
            self.current_opt = Point2::new(x, y);
        }
        self.update_verification();
        state.send_behavior(self.opt_x_wire, self.current_opt.x);
        state.send_behavior(self.opt_y_wire, self.current_opt.y);
        state.send_behavior(self.pos_x_wire, self.current_pos.x);
        state.send_behavior(self.pos_y_wire, self.current_pos.y);
        if self.energy >= 5000 {
            Some(EvalScore::Value(time_step as i32))
        } else {
            None
        }
    }

    fn end_time_step(&mut self, _time_step: u32, state: &CircuitState)
                     -> Vec<EvalError> {
        let delta = 10 *
            Point2::new(self.current_pos.x as i32 - self.current_opt.x as i32,
                        self.current_pos.y as i32 - self.current_opt.y as i32);
        let dist = (delta.x * delta.x + delta.y * delta.y).sqrt() as u32;
        self.energy += 85;
        self.energy = self.energy.saturating_sub(dist);
        match state.recv_behavior(self.motor_wire).0 {
            0x8 if self.current_pos.y < 0xf => self.current_pos.y += 1,
            0x4 if self.current_pos.y > 0x0 => self.current_pos.y -= 1,
            0x2 if self.current_pos.x > 0x0 => self.current_pos.x -= 1,
            0x1 if self.current_pos.x < 0xf => self.current_pos.x += 1,
            _ => {}
        }
        self.update_verification();
        Vec::new()
    }
}

//===========================================================================//
