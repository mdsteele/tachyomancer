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

const NUM_BATCHES_FOR_VICTORY: u32 = 8;
const MIX_AMOUNT_PER_TANK: u32 = 5;
const MIX_TIME: u32 = 6;
const NUM_TANKS: usize = 2;
const TANK_CAPACITY: u32 = 10;

const PUMP_PERIODS: [u32; NUM_TANKS] = [3, 7];
const PUMP_OFFSETS: [u32; NUM_TANKS] = [0, 5];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Intake #1 Interface",
        description: "Connects to the intake valve/pump for reagent tank #1.",
        side: Direction::North,
        pos: InterfacePosition::Right(0),
        ports: TANK_PORTS,
    },
    Interface {
        name: "Intake #2 Interface",
        description: "Connects to the intake valve/pump for reagent tank #2.",
        side: Direction::North,
        pos: InterfacePosition::Left(0),
        ports: TANK_PORTS,
    },
    Interface {
        name: "Mixer Interface",
        description: "Connects to the mixer unit.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Mix",
                description:
                    "Send an event here to drain 5 units from each tank and \
                     mix them together.   Both intake valves must remain \
                     closed during mixing.",
                flow: PortFlow::Sink,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Done",
                description:
                    "Sends an event when mixing is finished and the intake \
                     valves can be reopened.",
                flow: PortFlow::Source,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
];

const TANK_PORTS: &[InterfacePort] = &[
    InterfacePort {
        name: "Valve",
        description:
            "Set to 1 to open the intake valve, or 0 to close it.  It is an \
             error for the valve to be open during mixing, or if the tank is \
             already full.",
        flow: PortFlow::Sink,
        color: PortColor::Behavior,
        size: WireSize::One,
    },
    InterfacePort {
        name: "Pump",
        description:
            "Sends an event whenever a unit of reagent is pumped into the \
             tank (this can only happen if the intake valve is open).  The \
             tank can hold up to 10 units of reagent at once.",
        flow: PortFlow::Source,
        color: PortColor::Event,
        size: WireSize::Zero,
    },
];

//===========================================================================//

struct TankState {
    index: usize,
    valve_port: (Coords, Direction),
    valve_wire: WireId,
    pump_wire: WireId,
    amount: u32,
    to_be_drained: u32,
    did_pump: bool,
}

impl TankState {
    fn new(
        index: usize,
        slots: &[((Coords, Direction), WireId)],
    ) -> TankState {
        debug_assert_eq!(slots.len(), 2);
        TankState {
            index,
            valve_port: slots[0].0,
            valve_wire: slots[0].1,
            pump_wire: slots[1].1,
            amount: 0,
            to_be_drained: 0,
            did_pump: false,
        }
    }

    fn tank_number(&self) -> u32 {
        (self.index as u32) + 1
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        if self.did_pump {
            state.send_event(self.pump_wire, 0);
            self.did_pump = false;
        }
    }

    fn end_time_step(
        &mut self,
        state: &CircuitState,
        is_mixing: bool,
        errors: &mut Vec<EvalError>,
    ) {
        debug_assert!(self.to_be_drained <= self.amount);
        if self.to_be_drained > 0 {
            self.to_be_drained -= 1;
            self.amount -= 1;
        }
        if state.recv_behavior(self.valve_wire) > 0 {
            if is_mixing {
                let msg = format!(
                    "Intake valve #{} was left open during mixing.",
                    self.tank_number()
                );
                errors.push(state.fatal_port_error(self.valve_port, msg));
            }
            if self.amount >= TANK_CAPACITY {
                let msg = format!(
                    "Tank #{} is full, but the intake valve was left open.",
                    self.tank_number()
                );
                errors.push(state.fatal_port_error(self.valve_port, msg));
            } else {
                let period = PUMP_PERIODS[self.index];
                let offset = PUMP_OFFSETS[self.index];
                if (state.time_step() + offset) % (2 * period) < period {
                    self.amount += 1;
                    self.did_pump = true;
                }
            }
        }
    }
}

//===========================================================================//

pub struct FuelEval {
    tanks: [TankState; NUM_TANKS],
    mix_port: (Coords, Direction),
    mix_wire: WireId,
    done_wire: WireId,
    mix_timer: Option<u32>,
    num_batches_finished: u32,
}

impl FuelEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), WireId)>>) -> FuelEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 2);
        debug_assert_eq!(slots[2].len(), 2);
        FuelEval {
            tanks: [
                TankState::new(0, &slots[0]),
                TankState::new(1, &slots[1]),
            ],
            mix_port: slots[2][0].0,
            mix_wire: slots[2][0].1,
            done_wire: slots[2][1].1,
            mix_timer: None,
            num_batches_finished: 0,
        }
    }

    pub fn tank_amounts(&self) -> [u32; NUM_TANKS] {
        [self.tanks[0].amount, self.tanks[1].amount]
    }

    pub fn num_batches_finished(&self) -> u32 {
        self.num_batches_finished
    }
}

impl PuzzleEval for FuelEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.num_batches_finished >= NUM_BATCHES_FOR_VICTORY
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        for tank in self.tanks.iter_mut() {
            tank.begin_time_step(state);
        }
        if self.mix_timer == Some(0) {
            self.mix_timer = None;
            state.send_event(self.done_wire, 0);
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        if state.has_event(self.mix_wire) {
            if self.mix_timer.is_some() {
                let msg = format!(
                    "Tried to start mixing while already in progress."
                );
                errors.push(state.fatal_port_error(self.mix_port, msg));
            } else if !self
                .tanks
                .iter()
                .all(|tank| tank.amount >= MIX_AMOUNT_PER_TANK)
            {
                let msg = format!(
                    "Tried to start mixing with not enough reagent in each \
                     tank."
                );
                errors.push(state.fatal_port_error(self.mix_port, msg));
            } else {
                const_assert!(MIX_TIME >= MIX_AMOUNT_PER_TANK);
                self.mix_timer = Some(MIX_TIME);
                for tank in self.tanks.iter_mut() {
                    tank.to_be_drained = MIX_AMOUNT_PER_TANK;
                }
            }
        }
        errors
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        for tank in self.tanks.iter_mut() {
            tank.end_time_step(state, self.mix_timer.is_some(), &mut errors);
        }
        if let Some(ref mut time) = self.mix_timer {
            *time -= 1;
            if *time == 0 {
                self.num_batches_finished += 1;
            }
        }
        errors
    }
}

//===========================================================================//
