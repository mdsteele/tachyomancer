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

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
const CLICKS: &[u8] = &[
    1, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0,
    0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1,
    1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0,
    0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0,
    1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0,
    0, 1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0,
    1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1,
    1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1,
    0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0,
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Detector Interface",
        description: "Connects to the alpha particle detector.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Click",
            description:
                "Sends an event when a particle is detected (at most once \
                 per time step).",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Display Interface",
        description: "Connects to the device display.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Count",
            description:
                "This should be equal to the number of particles detected \
                 within the last ten time steps (including the current \
                 one).",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
];

//===========================================================================//

pub struct GeigerEval {
    click_wire: usize,
    count_port: (Coords, Direction),
    count_wire: usize,
}

impl GeigerEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> GeigerEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        GeigerEval {
            click_wire: slots[0][0].1,
            count_port: slots[1][0].0,
            count_wire: slots[1][0].1,
        }
    }
}

impl PuzzleEval for GeigerEval {
    fn task_is_completed(&self, state: &CircuitState) -> bool {
        (state.time_step() as usize) >= CLICKS.len()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        let step = state.time_step() as usize;
        if step < CLICKS.len() && CLICKS[step] != 0 {
            state.send_event(self.click_wire, 0);
        }
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let step = state.time_step() as usize;
        let start = step.saturating_sub(9);
        let end = CLICKS.len().min(step + 1);
        let expected = CLICKS[start..end].iter().map(|&i| i as u32).sum();
        let actual = state.recv_behavior(self.count_wire);
        let mut errors = Vec::<EvalError>::new();
        if actual != expected {
            let msg = format!(
                "Expected count of {} at time step {}, but was {}.",
                expected, step, actual
            );
            errors.push(state.port_error(self.count_port, msg));
        }
        errors
    }
}

//===========================================================================//
