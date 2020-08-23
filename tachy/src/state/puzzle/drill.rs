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

const DEPTH_FOR_VICTORY: u32 = 200;
const NUM_JOLTS_TO_FAIL: usize = 3;
const JOLT_TIME_WINDOW: usize = 5;

#[cfg_attr(rustfmt, rustfmt_skip)]
const JOLTS: &[u8; DEPTH_FOR_VICTORY as usize] = &[
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0,
    0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0,
    1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[Interface {
    name: "Drill Interface",
    description: "Connects to the drill's motor and sensors.",
    side: Direction::South,
    pos: InterfacePosition::Center,
    ports: &[
        InterfacePort {
            name: "Depth",
            description: "Indicates the current depth of the drill head, in \
                          decameters.",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        },
        InterfacePort {
            name: "Speed",
            description: "Controls the drilling speed.",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Two,
        },
        InterfacePort {
            name: "Jolt",
            description:
                "Signals whenever the drill head jolts while drilling.  \
                 If three jolts occur within five time steps, the drill \
                 head will break.",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        },
    ],
}];

//===========================================================================//

pub struct DrillingRigEval {
    depth_wire: WireId,
    speed_wire: WireId,
    jolt_wire: WireId,
    current_depth: u32,
    recent_jolts: VecDeque<bool>,
    hit_jolt: bool,
}

impl DrillingRigEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> DrillingRigEval {
        debug_assert_eq!(slots.len(), 1);
        debug_assert_eq!(slots[0].len(), 3);
        DrillingRigEval {
            depth_wire: slots[0][0].1,
            speed_wire: slots[0][1].1,
            jolt_wire: slots[0][2].1,
            current_depth: 0,
            recent_jolts: vec![false; JOLT_TIME_WINDOW].into_iter().collect(),
            hit_jolt: false,
        }
    }
}

impl PuzzleEval for DrillingRigEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.current_depth >= DEPTH_FOR_VICTORY
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        state.send_behavior(self.depth_wire, self.current_depth);
        if self.hit_jolt {
            state.send_event(self.jolt_wire, 0);
            self.hit_jolt = false;
        }
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        let speed = state.recv_behavior(self.speed_wire);
        let new_depth = (self.current_depth + speed).min(DEPTH_FOR_VICTORY);
        let start = self.current_depth as usize;
        let end = new_depth as usize;
        self.current_depth = new_depth;
        for &jolt in JOLTS[start..end].iter() {
            if jolt != 0 {
                self.hit_jolt = true;
            }
        }
        self.recent_jolts.pop_front();
        self.recent_jolts.push_back(self.hit_jolt);
        if self.recent_jolts.iter().filter(|&&j| j).count()
            >= NUM_JOLTS_TO_FAIL
        {
            errors.push(state.fatal_error(format!(
                "Encountered {} jolts within {} time steps",
                NUM_JOLTS_TO_FAIL, JOLT_TIME_WINDOW
            )));
        }
        errors
    }
}

//===========================================================================//
