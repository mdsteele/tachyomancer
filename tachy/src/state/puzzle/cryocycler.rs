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
use std::collections::HashSet;

//===========================================================================//

const NUM_PODS: u32 = 96;

#[cfg_attr(rustfmt, rustfmt_skip)]
const CYCLES: &[(u32, u32)] = &[
    ( 1,  2), (22, 30), (37, 38), (75, 76), ( 3,  4), (49, 50),
    (19, 27), (23, 31), ( 7,  8), (36, 44), (71, 79), (93, 94),
    (25, 26), (65, 73), (83, 91), (52, 60), (89, 90), (40, 48),
    (56, 64), (87, 88), (13, 14), (70, 78), (61, 62), (33, 41),
    ( 2, 10), (21, 22), (37, 45), (67, 68), ( 3, 11), (57, 58),
    (27, 28), (23, 24), ( 8, 16), (35, 36), (71, 72), (85, 86),
    (18, 26), (66, 74), (84, 92), (51, 59), (82, 90), (39, 47),
    (63, 64), (95, 96), ( 5, 13), (77, 78), (53, 61), (41, 42),
    ( 9, 10), (29, 30), (38, 46), (68, 76), (11, 12), (50, 58),
    (19, 20), (24, 32), ( 7, 15), (35, 43), (79, 80), (85, 93),
    (17, 25), (73, 74), (83, 84), (59, 60), (81, 82), (39, 40),
    (55, 63), (87, 95), ( 6, 14), (69, 70), (53, 54), (34, 42),
    ( 1,  9), (21, 29), (45, 46), (67, 75), ( 4, 12), (49, 57),
    (20, 28), (31, 32), (15, 16), (43, 44), (72, 80), (86, 94),
    (17, 18), (65, 66), (91, 92), (51, 52), (81, 89), (47, 48),
    (55, 56), (88, 96), ( 5,  6), (69, 77), (54, 62), (33, 34),
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Scheduler Interface",
        description:
            "Connects to the cryopod power cycle scheduler.  Whenever a pair \
             of cryopods is ready, each port sends an event with the number \
             of one of those pods (1-96).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Pod1",
                description: "",
                flow: PortFlow::Source,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Pod2",
                description: "",
                flow: PortFlow::Source,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
        ],
    },
    Interface {
        name: "Thermal Control Interface",
        description: "Connects to the cryopod thermal controller.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Thaw",
            description:
                "Set this to the number of a ready cryopod (1-96) to start \
                 thawing.  Set to 0 to not thaw anything this time step.  It \
                 is an error to try to thaw a cryopod that wasn't signaled \
                 by the scheduler this time step, or that has already been \
                 thawed.",
            flow: PortFlow::Sink,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
];

//===========================================================================//

pub struct CryocyclerEval {
    pod1_wire: WireId,
    pod2_wire: WireId,
    thaw_port: (Coords, Direction),
    thaw_wire: WireId,
    thawed: HashSet<u32>,
    cycle_index: usize,
}

impl CryocyclerEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> CryocyclerEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 1);
        CryocyclerEval {
            pod1_wire: slots[0][0].1,
            pod2_wire: slots[0][1].1,
            thaw_port: slots[1][0].0,
            thaw_wire: slots[1][0].1,
            thawed: HashSet::new(),
            cycle_index: 0,
        }
    }
}

impl PuzzleEval for CryocyclerEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.thawed.len() == (NUM_PODS as usize)
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        let (pod1, pod2) = CYCLES[self.cycle_index];
        state.send_event(self.pod1_wire, pod1);
        state.send_event(self.pod2_wire, pod2);
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        let (pod1, pod2) = CYCLES[self.cycle_index];
        let thaw = state.recv_behavior(self.thaw_wire);
        if thaw > NUM_PODS {
            let msg = format!(
                "Invalid pod number: {} is not in the range 1-{}.",
                thaw, NUM_PODS
            );
            errors.push(state.fatal_port_error(self.thaw_port, msg));
        } else if thaw > 0 {
            if thaw != pod1 && thaw != pod2 {
                let msg = format!(
                    "Pod number {} is not ready this time step (the ready \
                     pods are {} and {}).",
                    thaw, pod1, pod2
                );
                errors.push(state.fatal_port_error(self.thaw_port, msg));
            } else if self.thawed.contains(&thaw) {
                let msg =
                    format!("Pod number {} has already been thawed.", thaw);
                errors.push(state.fatal_port_error(self.thaw_port, msg));
            } else {
                self.thawed.insert(thaw);
            }
        }
        self.cycle_index = (self.cycle_index + 1) % CYCLES.len();
        errors
    }
}

//===========================================================================/

#[cfg(test)]
mod tests {
    use super::{CYCLES, NUM_PODS};

    #[test]
    fn cycles_hit_each_pod_twice() {
        let mut counts = vec![0i32; NUM_PODS as usize];
        for &(pod1, pod2) in CYCLES {
            assert!(pod1 >= 1 && pod1 <= NUM_PODS);
            counts[(pod1 - 1) as usize] += 1;
            assert!(pod2 >= 1 && pod2 <= NUM_PODS);
            counts[(pod2 - 1) as usize] += 1;
        }
        assert!(counts.iter().all(|&count| count == 2));
    }
}

//===========================================================================/
