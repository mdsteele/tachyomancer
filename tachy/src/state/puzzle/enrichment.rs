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

//      .----06----.
//     /      |     \
//   11      12      09
//  /  \    /  \    / |\
// 13  10  07  05 01 04 14
//        /  \      /  \
//       15  02    03  08

const PARENTS: &[u32] = &[0, 9, 7, 4, 9, 12, 0, 12, 4, 6, 11, 6, 6, 11, 9, 7];
const INIT_CONTENTS: &[u8] = &[0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 1, 0, 3];
const_assert_eq!(PARENTS.len(), INIT_CONTENTS.len());

const TRANSFERS: &[(u8, u32, u32)] = &[
    (1, 13, 5),
    (3, 15, 1),
    (2, 10, 15),
    (3, 1, 3),
    (1, 5, 1),
    (4, 2, 5),
    (2, 15, 2),
    (4, 5, 14),
    (2, 2, 5),
    (3, 3, 8),
    (2, 5, 3),
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Catalog Interface",
        description:
            "Connects to a database cataloging the connections between \
             storage tanks.  You can send one or more tank IDs here at any \
             time, and the parent tank ID for each requested tank will be \
             sent back during the next time step, in the same order the \
             requests were received.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Child",
                description:
                    "Send a tank ID here to request its parent tank's ID.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Parent",
                description:
                    "Indicates the ID of the requested tank's parent, or 0 if \
                     it's the root.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Control Interface",
        description:
            "Connects to the cascade control system.  When material is ready \
             to be moved from one centrifuge to another, the two ports will \
             send simultaneous events.",
        side: Direction::West,
        pos: InterfacePosition::Left(0),
        ports: &[
            InterfacePort {
                name: "Origin",
                description:
                    "Indicates the tank containing material to be moved.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Dest",
                description: "Indicates the destination tank.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Four,
            },
        ],
    },
    Interface {
        name: "Valve Interface",
        description:
            "Controls the tank valves.  Send the IDs of two connected tanks \
             to transfer the contents of one to the other.  The transfer \
             will complete after one time step.",
        side: Direction::East,
        pos: InterfacePosition::Right(0),
        ports: &[
            InterfacePort {
                name: "From",
                description: "The tank to drain (1-15), or 0 for none.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "To",
                description: "The tank to fill (1-15), or 0 for none.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Four,
            },
        ],
    },
];

//===========================================================================//

pub struct EnrichmentEval {
    child_port: (Coords, Direction),
    child_wire: WireId,
    parent_wire: WireId,
    origin_wire: WireId,
    dest_wire: WireId,
    from_wire: WireId,
    to_port: (Coords, Direction),
    to_wire: WireId,
    contents: Vec<u8>,
    num_transfers_completed: usize,
    current_goal: Option<(u8, u32)>,
    old_queries: VecDeque<usize>,
    new_queries: VecDeque<usize>,
}

impl EnrichmentEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> EnrichmentEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 2);
        debug_assert_eq!(slots[2].len(), 2);
        EnrichmentEval {
            child_port: slots[0][0].0,
            child_wire: slots[0][0].1,
            parent_wire: slots[0][1].1,
            origin_wire: slots[1][0].1,
            dest_wire: slots[1][1].1,
            from_wire: slots[2][0].1,
            to_port: slots[2][1].0,
            to_wire: slots[2][1].1,
            contents: INIT_CONTENTS.to_vec(),
            num_transfers_completed: 0,
            current_goal: None,
            old_queries: VecDeque::new(),
            new_queries: VecDeque::new(),
        }
    }

    pub fn tank_contents(&self) -> &[u8] {
        &self.contents
    }

    pub fn current_goal(&self) -> Option<(u8, u32)> {
        self.current_goal
    }

    fn send_next_query_reply(&mut self, state: &mut CircuitState) {
        if let Some(child) = self.old_queries.pop_front() {
            debug_assert!(child < PARENTS.len());
            state.send_event(self.parent_wire, PARENTS[child]);
        }
    }
}

impl PuzzleEval for EnrichmentEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.num_transfers_completed >= TRANSFERS.len()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        if self.current_goal.is_none()
            && self.num_transfers_completed < TRANSFERS.len()
        {
            let (item, origin, dest) = TRANSFERS[self.num_transfers_completed];
            self.current_goal = Some((item, dest));
            state.send_event(self.origin_wire, origin);
            state.send_event(self.dest_wire, dest);
        }
        self.send_next_query_reply(state);
    }

    fn begin_additional_cycle(&mut self, state: &mut CircuitState) {
        self.send_next_query_reply(state);
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if let Some(child) = state.recv_event(self.child_wire) {
            if child == 0 || (child as usize) >= PARENTS.len() {
                let msg = format!("{} is not a valid tank number.", child);
                errors.push(state.fatal_port_error(self.child_port, msg));
            } else {
                self.new_queries.push_back(child as usize);
            }
        }
        errors
    }

    fn needs_another_cycle(&self, _state: &CircuitState) -> bool {
        !self.old_queries.is_empty()
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        debug_assert!(self.old_queries.is_empty());
        std::mem::swap(&mut self.old_queries, &mut self.new_queries);

        let mut errors = Vec::<EvalError>::new();
        let from = state.recv_behavior(self.from_wire) as usize;
        let to = state.recv_behavior(self.to_wire) as usize;
        if from != 0
            && to != 0
            && from < PARENTS.len()
            && to < PARENTS.len()
            && ((PARENTS[from] as usize) == to
                || (PARENTS[to] as usize) == from)
            && self.contents[from] != 0
        {
            if self.contents[to] != 0 {
                let msg = format!(
                    "Cannot transfer material to occupied tank #{}",
                    to
                );
                errors.push(state.fatal_port_error(self.to_port, msg));
            } else {
                self.contents[to] = self.contents[from];
                self.contents[from] = 0;
            }
        }

        if let Some((item, dest)) = self.current_goal {
            if self.contents[dest as usize] == item {
                self.current_goal = None;
                self.num_transfers_completed += 1;
            }
        }

        errors
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{INIT_CONTENTS, PARENTS, TRANSFERS};
    use std::collections::HashSet;

    fn is_leaf(node: usize) -> bool {
        assert!(node > 0);
        assert!(node < PARENTS.len());
        !PARENTS.contains(&(node as u32))
    }

    #[test]
    fn tree_is_valid() {
        // Test that parents are valid and that there is exactly one root.
        let mut root: usize = 0;
        for (node, &parent) in PARENTS.iter().enumerate() {
            if node == 0 {
                continue;
            }
            let parent = parent as usize;
            assert!(parent < PARENTS.len());
            if parent == 0 {
                assert_eq!(
                    root, 0,
                    "Tree has more than one root ({} and {})",
                    root, node
                );
                root = node;
            }
        }
        assert_ne!(root, 0, "Tree has no root");

        // Test that all nodes reach up to the root (with no cycles).
        for start in 1..PARENTS.len() {
            let mut chain = Vec::<usize>::new();
            let mut visited = HashSet::<usize>::new();
            let mut node = start;
            while node != root {
                chain.push(node);
                assert!(
                    visited.insert(node),
                    "Cycle starting at {} ({:?})",
                    start,
                    chain
                );
                node = PARENTS[node] as usize;
            }
        }
    }

    #[test]
    fn non_leaves_start_empty() {
        for (index, &item) in INIT_CONTENTS.iter().enumerate() {
            if item != 0u8 {
                assert!(
                    is_leaf(index),
                    "Non-leaf node #{} starts with item {}",
                    index,
                    item
                );
            }
        }
    }

    #[test]
    fn transfers_are_valid() {
        let mut contents = INIT_CONTENTS.to_vec();
        for &(item, origin, dest) in TRANSFERS.iter() {
            assert_ne!(item, 0);
            let origin = origin as usize;
            let dest = dest as usize;
            assert!(is_leaf(origin));
            assert!(is_leaf(dest));
            assert_eq!(contents[origin], item);
            assert_eq!(contents[dest], 0);
            contents[origin] = 0;
            contents[dest] = item;
        }
    }
}

//===========================================================================//
