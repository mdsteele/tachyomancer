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

const EGG_INCUBATION_TIME: u32 = 20;
const EGG_LOAD_TIME: u32 = 2;
const EGG_UNLOAD_TIME: u32 = 2;

const LEFT_EGG_DELAYS: &[u32] = &[0, 9, 3, 4, 8];
const RIGHT_EGG_DELAYS: &[u32] = &[5, 3, 5, 9, 2];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Heater Interface",
        description: "Connects to the incubator heat lamp.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Heat",
            description:
                "Controls whether the heater is on (1) or off (0).  It is \
                 an error for the heater to be on while an egg is being \
                 loaded or unloaded.",
            flow: PortFlow::Sink,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Left Slot Interface",
        description:
            "Connects to the loading slot for the left-hand side of the \
             incubator.",
        side: Direction::West,
        pos: InterfacePosition::Right(0),
        ports: SIDE_PORTS,
    },
    Interface {
        name: "Right Slot Interface",
        description:
            "Connects to the loading slot for the right-hand side of the \
             incubator.",
        side: Direction::East,
        pos: InterfacePosition::Left(0),
        ports: SIDE_PORTS,
    },
];

const SIDE_PORTS: &[InterfacePort] = &[
    InterfacePort {
        name: "Next",
        description:
            "Sends an event when a new egg is available to be loaded \
             on this side.",
        flow: PortFlow::Source,
        color: PortColor::Event,
        size: WireSize::Zero,
    },
    InterfacePort {
        name: "Load",
        description: "Send 1 to load an egg.\n  \
                      Send 0 to unload an egg.",
        flow: PortFlow::Sink,
        color: PortColor::Event,
        size: WireSize::One,
    },
    InterfacePort {
        name: "Done",
        description: "Sends 1 when an egg finishes loading.\n  \
                      Sends 0 when an egg finishes unloading.",
        flow: PortFlow::Source,
        color: PortColor::Event,
        size: WireSize::One,
    },
];

//===========================================================================//

#[derive(Clone, Copy, Debug)]
enum EggState {
    Empty(u32),     // how long the slot has been empty
    Loading(u32),   // time left to load
    Unloading(u32), // time left to unload
    Warming(u32),   // how many time steps egg has been warmed for
}

//===========================================================================//

struct IncubatorSlot {
    delays: &'static [u32],
    next_wire: WireId,
    load_port: (Coords, Direction),
    load_wire: WireId,
    done_wire: WireId,
    egg_state: EggState,
    num_eggs_completed: usize,
}

impl IncubatorSlot {
    fn is_completed(&self) -> bool {
        self.num_eggs_completed >= self.delays.len()
    }

    fn is_loading_or_unloading(&self) -> bool {
        match self.egg_state {
            EggState::Empty(_) | EggState::Warming(_) => false,
            EggState::Loading(_) | EggState::Unloading(_) => true,
        }
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        match self.egg_state {
            EggState::Empty(time) => {
                if self.num_eggs_completed < self.delays.len()
                    && time == self.delays[self.num_eggs_completed]
                {
                    state.send_event(self.next_wire, 0);
                }
            }
            EggState::Loading(0) => {
                self.egg_state = EggState::Warming(0);
                state.send_event(self.done_wire, 1);
            }
            EggState::Unloading(0) => {
                self.egg_state = EggState::Empty(0);
                state.send_event(self.done_wire, 0);
            }
            _ => {}
        }
    }

    fn end_cycle(
        &mut self,
        state: &CircuitState,
        errors: &mut Vec<EvalError>,
    ) {
        let command = state.recv_event(self.load_wire);
        if command == Some(0) {
            match self.egg_state {
                EggState::Empty(_) => {
                    let msg = format!("Tried to unload with no egg loaded");
                    errors.push(state.fatal_port_error(self.load_port, msg));
                }
                EggState::Loading(_) => {
                    let msg = format!(
                        "Tried to unload an egg that's still loading."
                    );
                    errors.push(state.fatal_port_error(self.load_port, msg));
                }
                EggState::Unloading(_) => {
                    let msg = format!(
                        "Tried to unload an egg that's already unloading."
                    );
                    errors.push(state.fatal_port_error(self.load_port, msg));
                }
                EggState::Warming(time) => {
                    if time == EGG_INCUBATION_TIME {
                        self.egg_state = EggState::Unloading(EGG_UNLOAD_TIME);
                    } else {
                        let msg = format!(
                            "Tried to unload an egg that has only incubated \
                             for {} time steps instead of {}.",
                            time, EGG_INCUBATION_TIME,
                        );
                        errors
                            .push(state.fatal_port_error(self.load_port, msg));
                    }
                }
            }
        } else if command == Some(1) {
            match self.egg_state {
                EggState::Empty(time) => {
                    if self.num_eggs_completed < self.delays.len()
                        && time >= self.delays[self.num_eggs_completed]
                    {
                        self.egg_state = EggState::Loading(EGG_LOAD_TIME);
                    } else {
                        let msg = format!(
                            "Tried to load a new egg with no egg available \
                             to load."
                        );
                        errors
                            .push(state.fatal_port_error(self.load_port, msg));
                    }
                }
                EggState::Loading(_) => {
                    let msg = format!(
                        "Tried to load a new egg with an egg already loading."
                    );
                    errors.push(state.fatal_port_error(self.load_port, msg));
                }
                EggState::Unloading(_) => {
                    let msg = format!(
                        "Tried to load a new egg with an egg still unloading."
                    );
                    errors.push(state.fatal_port_error(self.load_port, msg));
                }
                EggState::Warming(_) => {
                    let msg = format!(
                        "Tried to load a new egg with an egg already loaded."
                    );
                    errors.push(state.fatal_port_error(self.load_port, msg));
                }
            }
        }
    }

    fn end_time_step(
        &mut self,
        state: &CircuitState,
        heat_is_on: bool,
        errors: &mut Vec<EvalError>,
    ) {
        match self.egg_state {
            EggState::Empty(time) => {
                self.egg_state = EggState::Empty(time + 1);
            }
            EggState::Loading(time) => {
                debug_assert!(time > 0);
                self.egg_state = EggState::Loading(time - 1);
            }
            EggState::Unloading(time) => {
                debug_assert!(time > 0);
                let new_time = time - 1;
                self.egg_state = EggState::Unloading(new_time);
                if new_time == 0 {
                    self.num_eggs_completed += 1;
                }
            }
            EggState::Warming(time) => {
                if heat_is_on {
                    let new_time = time + 1;
                    self.egg_state = EggState::Warming(new_time);
                    if new_time > EGG_INCUBATION_TIME {
                        let msg = format!(
                            "Incubated an egg for more than {} time steps",
                            EGG_INCUBATION_TIME
                        );
                        errors
                            .push(state.fatal_port_error(self.load_port, msg));
                    }
                }
            }
        }
    }
}

//===========================================================================//

pub struct IncubatorEval {
    heat_port: (Coords, Direction),
    heat_wire: WireId,
    left_slot: IncubatorSlot,
    right_slot: IncubatorSlot,
}

impl IncubatorEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> IncubatorEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 3);
        debug_assert_eq!(slots[2].len(), 3);
        IncubatorEval {
            heat_port: slots[0][0].0,
            heat_wire: slots[0][0].1,
            left_slot: IncubatorSlot {
                delays: LEFT_EGG_DELAYS,
                next_wire: slots[1][0].1,
                load_port: slots[1][1].0,
                load_wire: slots[1][1].1,
                done_wire: slots[1][2].1,
                egg_state: EggState::Empty(0),
                num_eggs_completed: 0,
            },
            right_slot: IncubatorSlot {
                delays: RIGHT_EGG_DELAYS,
                next_wire: slots[2][0].1,
                load_port: slots[2][1].0,
                load_wire: slots[2][1].1,
                done_wire: slots[2][2].1,
                egg_state: EggState::Empty(0),
                num_eggs_completed: 0,
            },
        }
    }
}

impl PuzzleEval for IncubatorEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.left_slot.is_completed() && self.right_slot.is_completed()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        self.left_slot.begin_time_step(state);
        self.right_slot.begin_time_step(state);
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::new();
        self.left_slot.end_cycle(state, &mut errors);
        self.right_slot.end_cycle(state, &mut errors);
        errors
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::new();
        let heat_is_on = state.recv_behavior(self.heat_wire) != 0;
        if heat_is_on
            && (self.left_slot.is_loading_or_unloading()
                || self.right_slot.is_loading_or_unloading())
        {
            let msg =
                format!("Heater must be off while loading/unloading an egg.");
            errors.push(state.fatal_port_error(self.heat_port, msg));
        }
        self.left_slot.end_time_step(state, heat_is_on, &mut errors);
        self.right_slot.end_time_step(state, heat_is_on, &mut errors);
        errors
    }
}

//===========================================================================//
