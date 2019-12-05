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

use super::super::eval::{CircuitState, EvalError, EvalScore, PuzzleEval};
use super::super::interface::{Interface, InterfacePort, InterfacePosition};
use crate::geom::{Coords, Direction};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow};
use std::collections::{BTreeSet, HashMap};

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
const CHARGE_DELAYS: &[u8; 256] = &[
    36, 33, 53,  7, 57, 32, 33, 20, 28,  8, 11, 32, 16, 54, 35, 47,
    48, 39, 60, 39, 18, 30, 45, 56,  9, 53, 31, 52, 51, 12, 54, 38,
    36, 19, 16,  8, 55, 47, 22, 25,  9, 17, 16, 38, 15, 27, 23, 38,
    16, 56,  8,  8, 13, 59, 36, 40, 41, 12, 29, 47, 57, 52, 17, 50,
    45, 35, 10, 58, 18, 24, 24, 22, 26, 24, 17, 40, 20, 52, 60,  8,
    33, 21,  8, 32, 15, 32, 24, 56, 49, 16, 13, 40, 17, 55, 36, 44,
    18, 54, 29, 17, 43, 54, 60, 37, 32, 49, 19, 45, 40, 27, 34, 24,
    36, 46, 51, 31, 10,  9, 20, 29, 60, 44, 20, 44, 27, 45,  5, 34,
    48, 53, 23, 21, 49, 42,  5, 32, 45, 59, 26, 16, 22, 43, 57, 43,
    52, 24, 13, 30, 49, 58, 45, 16, 47, 49, 59, 27, 49, 48, 56, 34,
    28, 37, 33, 31,  6, 12, 53, 46, 41, 51, 17, 42, 27, 17, 26, 29,
    16, 13, 55,  8, 43, 36, 37, 55, 19, 22, 51, 50, 24, 12, 58, 55,
    52, 50, 31, 29, 23, 11,  6, 11, 19, 30, 11,  7, 23, 54, 35, 17,
    38, 38, 25,  6, 35, 19, 59, 18, 59, 50, 17,  8, 39, 23, 46,  8,
    12, 31, 47,  9, 39, 60, 22, 16, 37, 52, 53, 20, 60, 39, 11, 48,
    26, 42,  6, 27, 40,  5, 27, 12, 29, 51, 15,  5, 57, 20, 57, 55,
];

fn delay_for(charge: u32) -> u32 {
    CHARGE_DELAYS[charge as usize] as u32
}

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Proximity Interface",
        description: "Connects to the missile's proximity sensor.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Proxy",
            description:
                "Indicates that the warhead should be detonated.  This will \
                 send a single event, at the beginning of the simulation.",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Detonator Interface",
        description:
            "Connects to the detonators for all of the warhead's explosive \
             charges.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Ping",
                description:
                    "Send an event here with a charge number (0-255) to send \
                     a test signal.  An event will arrive at the Pong port \
                     after 2N time steps, where N is the delay for that \
                     charge.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Fire",
                description:
                    "Send an event here with a charge number (0-255) to \
                     ignite that explosive charge.  It will detonate after N \
                     time steps, where N is the delay for that charge.",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
            InterfacePort {
                name: "Pong",
                description:
                    "When a test signal from the Ping port completes, this \
                     sends an event with the charge number.",
                flow: PortFlow::Send,
                color: PortColor::Event,
                size: WireSize::Eight,
            },
        ],
    },
];

//===========================================================================//

pub struct XUnitEval {
    proxy_wire: usize,
    ping_wire: usize,
    fire_wire: usize,
    pong_wire: usize,
    test_signals: HashMap<u32, BTreeSet<u32>>,
    detonations: HashMap<u32, BTreeSet<u32>>,
    any_detonated: bool,
}

impl XUnitEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> XUnitEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 3);
        XUnitEval {
            proxy_wire: slots[0][0].1,
            ping_wire: slots[1][0].1,
            fire_wire: slots[1][1].1,
            pong_wire: slots[1][2].1,
            test_signals: HashMap::new(),
            detonations: HashMap::new(),
            any_detonated: false,
        }
    }

    fn send_next_pong(&mut self, state: &mut CircuitState) {
        let time_step = state.time_step();
        if let Some(charges) = self.test_signals.get_mut(&time_step) {
            debug_assert!(!charges.is_empty());
            if let Some(&charge) = charges.iter().next() {
                state.send_event(self.pong_wire, charge);
                charges.remove(&charge);
            }
            if charges.is_empty() {
                self.test_signals.remove(&time_step);
            }
        }
    }
}

impl PuzzleEval for XUnitEval {
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        if state.time_step() == 0 {
            state.send_event(self.proxy_wire, 0);
        }
        self.send_next_pong(state);
        if self.any_detonated {
            Some(EvalScore::Value(state.time_step()))
        } else {
            None
        }
    }

    fn begin_additional_cycle(&mut self, state: &mut CircuitState) {
        self.send_next_pong(state);
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        if let Some(charge) = state.recv_event(self.ping_wire) {
            let pong_time = state.time_step() + 2 * delay_for(charge);
            self.test_signals
                .entry(pong_time)
                .or_insert_with(BTreeSet::new)
                .insert(charge);
        }
        if let Some(charge) = state.recv_event(self.fire_wire) {
            let fire_time = state.time_step() + delay_for(charge);
            self.detonations
                .entry(fire_time)
                .or_insert_with(BTreeSet::new)
                .insert(charge);
        }
        Vec::new()
    }

    fn needs_another_cycle(&self, time_step: u32) -> bool {
        self.test_signals.contains_key(&time_step)
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let mut errors = Vec::<EvalError>::new();
        match self.detonations.remove(&state.time_step()) {
            Some(charges) => {
                debug_assert!(!charges.is_empty());
                self.any_detonated = true;
                if charges.len() < CHARGE_DELAYS.len() {
                    errors.push(state.fatal_error(format!(
                        "Only {} out of {} charges were detonated at once",
                        charges.len(),
                        CHARGE_DELAYS.len()
                    )));
                }
            }
            None => {}
        }
        errors
    }
}

//===========================================================================//
