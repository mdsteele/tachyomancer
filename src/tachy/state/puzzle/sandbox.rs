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
use super::super::eval::{CircuitState, EvalScore, PuzzleEval};
use tachy::geom::{Coords, Direction};
use tachy::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const BEHAVIOR_INTERFACES: &[Interface] = &[
    Interface {
        name: "Timer Interface",
        description: "Connected to a digital timer.",
        side: Direction::West,
        pos: InterfacePosition::Right(0),
        ports: &[
            InterfacePort {
                name: "Time",
                description: "Outputs the current time step.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
        ],
    },
];

pub const EVENT_INTERFACES: &[Interface] = &[
    Interface {
        name: "Timer Interface",
        description: "Connected to a digital timer.",
        side: Direction::West,
        pos: InterfacePosition::Right(0),
        ports: &[
            InterfacePort {
                name: "Time",
                description: "Outputs the current time step.",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Metronome",
                description: "Sends an event at the beginning of \
                              each time step.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
];

//===========================================================================//

pub struct SandboxBehaviorEval {
    timer: usize,
}

impl SandboxBehaviorEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>)
               -> SandboxBehaviorEval {
        debug_assert_eq!(slots.len(), 1);
        debug_assert_eq!(slots[0].len(), 1);
        SandboxBehaviorEval { timer: slots[0][0].1 }
    }
}

impl PuzzleEval for SandboxBehaviorEval {
    fn verification_data(&self) -> &[u64] { &[] }

    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        state.send_behavior(self.timer, time_step & 0xff);
        None
    }
}

//===========================================================================//

pub struct SandboxEventEval {
    metronome: usize,
    timer: usize,
}

impl SandboxEventEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>)
               -> SandboxEventEval {
        debug_assert_eq!(slots.len(), 1);
        debug_assert_eq!(slots[0].len(), 2);
        SandboxEventEval {
            metronome: slots[0][1].1,
            timer: slots[0][0].1,
        }
    }
}

impl PuzzleEval for SandboxEventEval {
    fn verification_data(&self) -> &[u64] { &[] }

    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore> {
        state.send_event(self.metronome, 0);
        state.send_behavior(self.timer, time_step & 0xff);
        None
    }
}

//===========================================================================//
