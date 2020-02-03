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
use std::collections::HashSet;

//===========================================================================//

const GRID_COLS: u32 = 9;
const GRID_ROWS: u32 = 8;
const TIME_BETWEEN_TRANSMISSIONS: u32 = 4;

#[cfg_attr(rustfmt, rustfmt_skip)]
const POSITIONS: &[(u32, u32)] = &[
    (2, 7), (3, 4), (7, 4), (4, 5), (7, 0),
    (1, 1), (2, 3), (5, 4), (8, 1), (0, 6),
    (7, 5), (7, 1), (8, 0), (3, 6), (5, 3),
    (6, 3), (3, 3), (5, 2), (4, 2), (3, 5),
    (6, 5), (5, 6), (6, 6), (0, 4), (4, 4),
    (1, 7), (4, 1), (5, 1), (8, 7), (8, 2),
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Radio Interface",
        description: "Connects to a radio antenna.",
        side: Direction::West,
        pos: InterfacePosition::Left(1),
        ports: &[InterfacePort {
            name: "RX",
            description:
                "Connects to the radio receiver.  Sends an event when a \
                 radio command arrives.  Each command encodes an (X, Y) \
                 position that should be collected; the lower 4 bits contain \
                 X, and the upper 4 bits contain Y.",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "X-Movement Interface",
        description: "Connects to the X-axis actuator.",
        side: Direction::East,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "XPos",
                description: "Outputs the current X position.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "XMove",
                description:
                    "Send 1 here to move right; send 0 here to move left.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::One,
            },
            InterfacePort {
                name: "Done",
                description: "Signals when X movement has finished.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
    Interface {
        name: "Y-Movement Interface",
        description: "Connects to the Y-axis actuator.",
        side: Direction::North,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "YPos",
                description: "Outputs the current Y position.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "YMove",
                description:
                    "Send 1 here to move up; send 0 here to move down.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::One,
            },
            InterfacePort {
                name: "Done",
                description: "Signals when Y movement has finished.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
    Interface {
        name: "Collection Interface",
        description: "Connects to the collector arm.",
        side: Direction::South,
        pos: InterfacePosition::Left(1),
        ports: &[
            InterfacePort {
                name: "Coll",
                description: "Signal here to collect at the current position.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Done",
                description: "Signals when collection has finished.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
];

//===========================================================================//

pub struct CollectorEval {
    rx_wire: usize,
    xpos_wire: usize,
    xmove_port: (Coords, Direction),
    xmove_wire: usize,
    xdone_wire: usize,
    ypos_wire: usize,
    ymove_port: (Coords, Direction),
    ymove_wire: usize,
    ydone_wire: usize,
    cmove_port: (Coords, Direction),
    cmove_wire: usize,
    cdone_wire: usize,
    position: (u32, u32),
    x_moving: Option<u32>,
    y_moving: Option<u32>,
    c_moving: bool,
    pending: HashSet<(u32, u32)>,
    num_transmissions: usize,
}

impl CollectorEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), usize)>>,
    ) -> CollectorEval {
        debug_assert_eq!(slots.len(), 4);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 3);
        debug_assert_eq!(slots[2].len(), 3);
        debug_assert_eq!(slots[3].len(), 2);
        CollectorEval {
            rx_wire: slots[0][0].1,
            xpos_wire: slots[1][0].1,
            xmove_port: slots[1][1].0,
            xmove_wire: slots[1][1].1,
            xdone_wire: slots[1][2].1,
            ypos_wire: slots[2][0].1,
            ymove_port: slots[2][1].0,
            ymove_wire: slots[2][1].1,
            ydone_wire: slots[2][2].1,
            cmove_port: slots[3][0].0,
            cmove_wire: slots[3][0].1,
            cdone_wire: slots[3][1].1,
            position: (GRID_COLS / 2, GRID_ROWS / 2),
            x_moving: None,
            y_moving: None,
            c_moving: false,
            pending: HashSet::new(),
            num_transmissions: 0,
        }
    }
}

impl PuzzleEval for CollectorEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.num_transmissions == POSITIONS.len() && self.pending.is_empty()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        if self.num_transmissions < POSITIONS.len()
            && ((state.time_step() / TIME_BETWEEN_TRANSMISSIONS) as usize)
                >= self.num_transmissions
        {
            let (x, y) = POSITIONS[self.num_transmissions];
            state.send_event(self.rx_wire, (x << 4) | y);
            self.num_transmissions += 1;
            self.pending.insert((x, y));
        }
        if self.x_moving.is_some() {
            state.send_event(self.xdone_wire, 0);
            self.x_moving = None;
        }
        if self.y_moving.is_some() {
            state.send_event(self.ydone_wire, 0);
            self.y_moving = None;
        }
        if self.c_moving {
            state.send_event(self.cdone_wire, 0);
            self.c_moving = false;
        }
        state.send_behavior(self.xpos_wire, self.position.0);
        state.send_behavior(self.ypos_wire, self.position.1);
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if let Some(dir) = state.recv_event(self.xmove_wire) {
            if self.x_moving.is_some() {
                let msg = format!("Already moving along X-axis.");
                errors.push(state.fatal_port_error(self.xmove_port, msg));
            } else if dir == 0 && self.position.0 == 0 {
                let msg = format!("Can't move any farther left.");
                errors.push(state.fatal_port_error(self.xmove_port, msg));
            } else if dir != 0 && self.position.0 + 1 >= GRID_COLS {
                let msg = format!("Can't move any farther right.");
                errors.push(state.fatal_port_error(self.xmove_port, msg));
            } else {
                self.x_moving = Some(dir);
            }
        }
        if let Some(dir) = state.recv_event(self.ymove_wire) {
            if self.y_moving.is_some() {
                let msg = format!("Already moving along Y-axis.");
                errors.push(state.fatal_port_error(self.ymove_port, msg));
            } else if dir == 0 && self.position.1 == 0 {
                let msg = format!("Can't move any farther down.");
                errors.push(state.fatal_port_error(self.ymove_port, msg));
            } else if dir != 0 && self.position.1 + 1 >= GRID_ROWS {
                let msg = format!("Can't move any farther up.");
                errors.push(state.fatal_port_error(self.ymove_port, msg));
            } else {
                self.y_moving = Some(dir);
            }
        }
        if state.has_event(self.cmove_wire) {
            if self.c_moving {
                let msg = format!("Still collecting.");
                errors.push(state.fatal_port_error(self.cmove_port, msg));
            } else if !self.pending.contains(&self.position) {
                let msg = format!(
                    "There is nothing to collect at {:?}.",
                    self.position
                );
                errors.push(state.fatal_port_error(self.cmove_port, msg));
            } else {
                self.pending.remove(&self.position);
                self.c_moving = true;
            }
        }
        errors
    }

    fn end_time_step(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        if let Some(dir) = self.x_moving {
            if dir == 0 {
                debug_assert!(self.position.0 > 0);
                self.position.0 -= 1;
            } else {
                self.position.0 += 1;
                debug_assert!(self.position.0 < GRID_COLS);
            }
        }
        if let Some(dir) = self.y_moving {
            if dir == 0 {
                debug_assert!(self.position.1 > 0);
                self.position.1 -= 1;
            } else {
                self.position.1 += 1;
                debug_assert!(self.position.1 < GRID_ROWS);
            }
        }
        vec![]
    }
}

//===========================================================================/

#[cfg(test)]
mod tests {
    use super::{GRID_COLS, GRID_ROWS, POSITIONS};
    use std::collections::HashSet;

    #[test]
    fn positions_are_within_grid() {
        for &(x, y) in POSITIONS {
            assert!(x < GRID_COLS);
            assert!(y < GRID_ROWS);
        }
    }

    #[test]
    fn positions_are_unique() {
        let mut positions = HashSet::new();
        for &pos in POSITIONS {
            assert!(!positions.contains(&pos));
            positions.insert(pos);
        }
    }
}

//===========================================================================//
