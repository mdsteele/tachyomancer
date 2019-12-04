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
use crate::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Upper",
        description:
            "Indicates the current upper bound of the scan range (inclusive).",
        side: Direction::West,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Upper",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Lower",
        description:
            "Indicates the current lower bound of the scan range (inclusive).",
        side: Direction::West,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Lower",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Out",
        description: "Controls where the scan range will be subdivided.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
];

//===========================================================================//

const GOALS: &[u32] = &[5, 9, 15, 7, 11, 1, 3, 13];

pub struct SensorsEval {
    upper_wire: usize,
    lower_wire: usize,
    out_wire: usize,
    current_upper: u32,
    current_lower: u32,
    current_goal: u32,
    num_goals_found: usize,
}

impl SensorsEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> SensorsEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        SensorsEval {
            upper_wire: slots[0][0].1,
            lower_wire: slots[1][0].1,
            out_wire: slots[2][0].1,
            current_upper: 15,
            current_lower: 0,
            current_goal: GOALS[0],
            num_goals_found: 0,
        }
    }

    pub fn lower_bound(&self) -> u32 {
        self.current_lower
    }

    pub fn upper_bound(&self) -> u32 {
        self.current_upper
    }

    pub fn num_goals_found(&self) -> usize {
        self.num_goals_found
    }
}

impl PuzzleEval for SensorsEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        if self.num_goals_found >= GOALS.len() {
            Some(EvalScore::Value(state.time_step()))
        } else {
            state.send_behavior(self.upper_wire, self.current_upper);
            state.send_behavior(self.lower_wire, self.current_lower);
            None
        }
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let out = state.recv_behavior(self.out_wire);
        if self.current_lower == self.current_upper {
            if out == self.current_lower {
                self.num_goals_found += 1;
                self.current_lower = 0;
                self.current_upper = 15;
                if self.num_goals_found < GOALS.len() {
                    self.current_goal = GOALS[self.num_goals_found];
                }
            }
        } else if out >= self.current_lower && out <= self.current_upper {
            if out == self.current_goal {
                if out - self.current_lower <= self.current_upper - out {
                    self.current_upper = out;
                } else {
                    self.current_lower = out;
                }
            } else if out > self.current_goal {
                self.current_upper = out;
            } else {
                self.current_lower = out;
            }
        }
        vec![]
    }
}

//===========================================================================//
