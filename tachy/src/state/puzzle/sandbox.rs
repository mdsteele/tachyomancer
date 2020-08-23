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

use super::super::eval::{CircuitState, PuzzleEval};
use super::super::interface::{Interface, InterfacePort, InterfacePosition};
use crate::geom::{Coords, Direction};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow, WireId};

//===========================================================================//

pub const BEHAVIOR_INTERFACES: &[Interface] = &[Interface {
    name: "Timer Interface",
    description: "Connects to a digital timer.",
    side: Direction::West,
    pos: InterfacePosition::Right(0),
    ports: &[InterfacePort {
        name: "Time",
        description: "Outputs the current time step.",
        flow: PortFlow::Send,
        color: PortColor::Behavior,
        size: WireSize::Eight,
    }],
}];

pub const EVENT_INTERFACES: &[Interface] = &[
    Interface {
        name: "Startup Interface",
        description: "Connects to the power supply.",
        side: Direction::North,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Init",
            description:
                "Sends a single event at the start of the first time step.",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Timer Interface",
        description: "Connects to a digital timer.",
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
                name: "Tick",
                description:
                    "Sends an event at the beginning of each time step.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
];

//===========================================================================//

pub struct SandboxBehaviorEval {
    time_wire: WireId,
}

impl SandboxBehaviorEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> SandboxBehaviorEval {
        debug_assert_eq!(slots.len(), 1);
        debug_assert_eq!(slots[0].len(), 1);
        SandboxBehaviorEval { time_wire: slots[0][0].1 }
    }
}

impl PuzzleEval for SandboxBehaviorEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        false
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        state.send_behavior(self.time_wire, state.time_step() & 0xff);
    }
}

//===========================================================================//

pub struct SandboxEventEval {
    init_wire: WireId,
    time_wire: WireId,
    tick_wire: WireId,
}

impl SandboxEventEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> SandboxEventEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 2);
        SandboxEventEval {
            init_wire: slots[0][0].1,
            time_wire: slots[1][0].1,
            tick_wire: slots[1][1].1,
        }
    }
}

impl PuzzleEval for SandboxEventEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        false
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        if state.time_step() == 0 {
            state.send_event(self.init_wire, 0);
        }
        state.send_behavior(self.time_wire, state.time_step() & 0xff);
        state.send_event(self.tick_wire, 0);
    }
}

//===========================================================================//
