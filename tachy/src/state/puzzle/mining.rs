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

#[cfg_attr(rustfmt, rustfmt_skip)]
const INITIAL_ORE_DEPOSITS: &[u32] = &[
    0, 1, 3, 1, 3, 3, 1, 2, 3, 2, 1, 2, 2, 3, 3, 2, 2, 1, 2, 3, 3, 1, 2, 2, 2,
    3, 1, 1, 3, 1, 1,
];
const ORE_NEEDED_AT_BASE: u32 = 60;
const MAX_ORE_CARRIED: u32 = 15;

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Nav Interface",
        description: "Connects to the robot's automatic navigation system.",
        side: Direction::East,
        pos: InterfacePosition::Right(1),
        ports: &[
            InterfacePort {
                name: "Dist",
                description:
                    "\
                     Indicates how far away the robot is from the base.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Back",
                description:
                    "\
                     Signal here when the robot should return to the base.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
    Interface {
        name: "Digger Interface",
        description: "Connects to the robot's ore digging/hauling equipment.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Carry",
                description:
                    "\
                     Indicates how much ore the robot is currently carrying, \
                     in kilograms.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Found",
                description:
                    "\
                     Sends an event with the size of an ore deposit (from \
                     1-3kg) when the robot passes over it.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Two,
            },
            InterfacePort {
                name: "Dig",
                description:
                    "\
                     Signal here to dig up the ore deposit under the robot.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
];

//===========================================================================//

pub struct MiningRobotEval {
    dist_wire: usize,
    back_wire: usize,
    carry_wire: usize,
    found_wire: usize,
    dig_port: (Coords, Direction),
    dig_wire: usize,
    ore_carried: u32,
    ore_at_base: u32,
    current_position: usize,
    returning_to_base: bool,
    ore_deposits: Vec<u32>,
}

impl MiningRobotEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> MiningRobotEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 3);
        MiningRobotEval {
            dist_wire: slots[0][0].1,
            back_wire: slots[0][1].1,
            carry_wire: slots[1][0].1,
            found_wire: slots[1][1].1,
            dig_port: slots[1][2].0,
            dig_wire: slots[1][2].1,
            ore_carried: 0,
            ore_at_base: 0,
            current_position: 0,
            returning_to_base: false,
            ore_deposits: INITIAL_ORE_DEPOSITS.to_vec(),
        }
    }

    pub fn current_position(&self) -> usize {
        self.current_position
    }

    pub fn ore_carried(&self) -> u32 {
        self.ore_carried
    }

    pub fn ore_at_base(&self) -> u32 {
        self.ore_at_base
    }
}

impl PuzzleEval for MiningRobotEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        state.send_behavior(self.dist_wire, self.current_position as u32);
        state.send_behavior(self.carry_wire, self.ore_carried);
        let ore = self.ore_deposits[self.current_position];
        if ore > 0 {
            state.send_event(self.found_wire, ore);
        }
        if self.ore_at_base >= ORE_NEEDED_AT_BASE {
            Some(EvalScore::Value(state.time_step()))
        } else {
            None
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if state.recv_event(self.back_wire).is_some() {
            self.returning_to_base = true;
        }
        if state.recv_event(self.dig_wire).is_some() {
            let ore = self.ore_deposits[self.current_position];
            if ore + self.ore_carried <= MAX_ORE_CARRIED {
                self.ore_carried += ore;
                self.ore_deposits[self.current_position] = 0;
            } else {
                let message = format!(
                    "Tried to dig up a {}kg ore deposit \
                     while already carrying {}kg (max load \
                     is {}kg).",
                    ore, self.ore_carried, MAX_ORE_CARRIED
                );
                errors.push(state.fatal_port_error(self.dig_port, message));
            }
        }
        errors
    }

    fn end_time_step(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        let last_position = self.ore_deposits.len() - 1;
        if self.returning_to_base {
            if self.current_position > 0 {
                self.current_position -= 1;
            }
        } else {
            if self.current_position < last_position {
                self.current_position += 1;
            }
        }
        if self.current_position == 0 {
            self.ore_at_base += self.ore_carried;
            self.ore_carried = 0;
            self.returning_to_base = false;
        } else if self.current_position == last_position {
            self.returning_to_base = true;
        }
        Vec::new()
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{INITIAL_ORE_DEPOSITS, ORE_NEEDED_AT_BASE};

    #[test]
    fn base_needs_all_ore_deposits() {
        assert_eq!(ORE_NEEDED_AT_BASE, INITIAL_ORE_DEPOSITS.iter().sum());
    }
}

//===========================================================================//
