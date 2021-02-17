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

//===========================================================================//

const NUM_POSITIONS: usize = 7;

#[cfg_attr(rustfmt, rustfmt_skip)]
const ARRAYS: &[[u32; NUM_POSITIONS]] = &[
    [2, 3, 2, 4, 5, 5, 6],
    [5, 6, 7, 5, 2, 0, 3],
    [1, 4, 1, 6, 2, 7, 4],
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Injection Interface",
        description: "Connects to the plasma injection head.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Dir",
                description:
                    "Set this to 0 to move the injection head to the left; \
                     set this to 1 to move the injection head to the right.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::One,
            },
            InterfacePort {
                name: "Pos",
                description:
                    "Sends an event with the current position (0-6) at the \
                     start of each time step.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Four,
            },
            InterfacePort {
                name: "Inj",
                description:
                    "Send an event here to inject at the current position.  \
                     It is an error to inject more than once per time step.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
    Interface {
        name: "Sensor Interface",
        description: "Connects to the feedback sensor array.",
        side: Direction::West,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Seq",
            description:
                "Whenever a new round of injections is needed, sends a \
                 sequence of 7 events indicating the number of injections \
                 needed at each position (from position 0 to position 6).",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Eight,
        }],
    },
];

//===========================================================================//

pub struct InjectorEval {
    dir_wire: WireId,
    pos_wire: WireId,
    inj_port: (Coords, Direction),
    inj_wire: WireId,
    sensor_wire: WireId,
    position: usize,
    pending: Vec<u32>,
    sensor_queue: &'static [u32],
    num_arrays_sent: usize,
    injected: bool,
}

impl InjectorEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> InjectorEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 3);
        debug_assert_eq!(slots[1].len(), 1);
        InjectorEval {
            dir_wire: slots[0][0].1,
            pos_wire: slots[0][1].1,
            inj_port: slots[0][2].0,
            inj_wire: slots[0][2].1,
            sensor_wire: slots[1][0].1,
            position: NUM_POSITIONS - 1,
            pending: vec![0; NUM_POSITIONS],
            sensor_queue: &[],
            num_arrays_sent: 0,
            injected: false,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn pending(&self) -> &[u32] {
        &self.pending
    }
}

impl PuzzleEval for InjectorEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.num_arrays_sent >= ARRAYS.len()
            && self.pending.iter().all(|&p| p == 0)
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        state.send_event(self.pos_wire, self.position as u32);
        if self.num_arrays_sent < ARRAYS.len()
            && self.pending.iter().all(|&p| p == 0)
        {
            self.sensor_queue = &ARRAYS[self.num_arrays_sent];
            self.pending = self.sensor_queue.to_vec();
            self.num_arrays_sent += 1;
            state.send_event(self.sensor_wire, self.sensor_queue[0]);
            self.sensor_queue = &self.sensor_queue[1..];
        }
    }

    fn begin_additional_cycle(&mut self, state: &mut CircuitState) {
        if !self.sensor_queue.is_empty() {
            state.send_event(self.sensor_wire, self.sensor_queue[0]);
            self.sensor_queue = &self.sensor_queue[1..];
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if state.has_event(self.inj_wire) {
            if self.injected {
                let msg =
                    format!("Can't inject more than once per time step.");
                errors.push(state.fatal_port_error(self.inj_port, msg));
            } else if self.pending[self.position] == 0 {
                let msg = format!(
                    "No more injections needed at position {}.",
                    self.position
                );
                errors.push(state.fatal_port_error(self.inj_port, msg));
            } else {
                self.pending[self.position] -= 1;
                self.injected = true;
            }
        }
        errors
    }

    fn needs_another_cycle(&self, _state: &CircuitState) -> bool {
        !self.sensor_queue.is_empty()
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        self.injected = false;
        let dir = state.recv_behavior(self.dir_wire);
        if dir == 0 && self.position > 0 {
            self.position -= 1;
        } else if dir != 0 && self.position < NUM_POSITIONS - 1 {
            self.position += 1;
        }
        vec![]
    }
}

//===========================================================================/
