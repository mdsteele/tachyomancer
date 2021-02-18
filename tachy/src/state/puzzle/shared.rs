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
use super::super::interface::{Interface, InterfacePort};
use crate::geom::{Coords, Direction, Fixed};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow, WireId};
use std::collections::HashSet;
use std::u32;

//===========================================================================//

const EPSILON: Fixed = Fixed::from_ratio(5, 1000);

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TutorialBubblePosition {
    Bounds(Direction),
    ControlsTray,
    PartsTray,
}

//===========================================================================//

pub const NIL: u32 = u32::MAX;

pub struct FabricationData {
    pub(super) interfaces: &'static [Interface],
    pub(super) expected_table_values: &'static [u32],
}

impl FabricationData {
    pub fn table_column_ports(&self) -> Vec<&'static InterfacePort> {
        let mut ports = Vec::new();
        for interface in self.interfaces.iter() {
            for port in interface.ports.iter() {
                ports.push(port);
            }
        }
        ports
    }

    pub fn expected_table_values(&self) -> &[u32] {
        self.expected_table_values
    }
}

//===========================================================================//

pub struct FabricationEval {
    interfaces: &'static [Interface],
    slots: Vec<Vec<((Coords, Direction), WireId)>>,
    num_columns: usize,
    expected_table_values: &'static [u32],
    table_values: Vec<u32>,
    has_received_events: HashSet<WireId>,
}

impl FabricationEval {
    pub fn new(
        data: &FabricationData,
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> FabricationEval {
        let mut num_columns: usize = 0;
        debug_assert_eq!(slots.len(), data.interfaces.len());
        for (index, interface) in data.interfaces.iter().enumerate() {
            debug_assert_eq!(slots[index].len(), interface.ports.len());
            num_columns += interface.ports.len();
        }
        debug_assert_eq!(data.expected_table_values.len() % num_columns, 0);
        FabricationEval {
            interfaces: data.interfaces,
            slots,
            num_columns,
            expected_table_values: data.expected_table_values,
            table_values: data.expected_table_values.to_vec(),
            has_received_events: HashSet::new(),
        }
    }

    pub fn table_values(&self) -> &[u32] {
        &self.table_values
    }
}

impl PuzzleEval for FabricationEval {
    fn task_is_completed(&self, state: &CircuitState) -> bool {
        let start = (state.time_step() as usize) * self.num_columns;
        start >= self.expected_table_values.len()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        self.has_received_events.clear();
        let start = (state.time_step() as usize) * self.num_columns;
        let mut column_index = 0;
        for iface_index in 0..self.interfaces.len() {
            let interface = &self.interfaces[iface_index];
            for port_index in 0..interface.ports.len() {
                let port = &interface.ports[port_index];
                if port.flow == PortFlow::Send {
                    let wire = self.slots[iface_index][port_index].1;
                    let value =
                        self.expected_table_values[start + column_index];
                    match port.color {
                        PortColor::Behavior => {
                            debug_assert!(value <= port.size.mask());
                            state.send_behavior(wire, value);
                        }
                        PortColor::Event => {
                            if value != NIL {
                                debug_assert!(value <= port.size.mask());
                                state.send_event(wire, value);
                            }
                        }
                        PortColor::Analog => {
                            // TODO: Smoothly vary analog value over time.
                            debug_assert_eq!(port.size, WireSize::ANALOG);
                            let fixed = Fixed::from_encoded(value);
                            state.send_analog(wire, fixed);
                        }
                    }
                }
                column_index += 1;
            }
        }
        debug_assert_eq!(column_index, self.num_columns);
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let start = (state.time_step() as usize) * self.num_columns;
        debug_assert!(start < self.expected_table_values.len());
        let mut errors = Vec::new();
        let mut column_index = 0;
        for iface_index in 0..self.interfaces.len() {
            let interface = &self.interfaces[iface_index];
            for port_index in 0..interface.ports.len() {
                let port = &interface.ports[port_index];
                if port.flow == PortFlow::Recv
                    && port.color == PortColor::Event
                {
                    let (loc, wire) = self.slots[iface_index][port_index];
                    if let Some(actual) = state.recv_event(wire) {
                        let expected =
                            self.expected_table_values[start + column_index];
                        self.table_values[start + column_index] = actual;
                        if expected == NIL {
                            let msg = if port.size == WireSize::Zero {
                                format!(
                                    "No event expected for {}, \
                                     but got an event.",
                                    port.name
                                )
                            } else {
                                format!(
                                    "No event expected for {}, \
                                     but got event value of {}.",
                                    port.name, actual
                                )
                            };
                            errors.push(state.port_error(loc, msg));
                        } else if self.has_received_events.contains(&wire) {
                            let msg = format!(
                                "Expected only one event for {}, \
                                 but got more than one.",
                                port.name
                            );
                            errors.push(state.port_error(loc, msg));
                        } else if actual != expected {
                            let msg = format!(
                                "Expected event value of {} for {}, \
                                 but got event value of {}.",
                                expected, port.name, actual
                            );
                            errors.push(state.port_error(loc, msg));
                        }
                        self.has_received_events.insert(wire);
                    }
                }
                column_index += 1;
            }
        }
        debug_assert_eq!(column_index, self.num_columns);
        errors
    }

    fn end_time_step(&mut self, state: &CircuitState) -> Vec<EvalError> {
        let start = (state.time_step() as usize) * self.num_columns;
        debug_assert!(start < self.expected_table_values.len());
        let mut errors = Vec::new();
        let mut column_index = 0;
        for iface_index in 0..self.interfaces.len() {
            let interface = &self.interfaces[iface_index];
            for port_index in 0..interface.ports.len() {
                let port = &interface.ports[port_index];
                if port.flow == PortFlow::Recv {
                    let (loc, wire) = self.slots[iface_index][port_index];
                    let expected =
                        self.expected_table_values[start + column_index];
                    match port.color {
                        PortColor::Behavior => {
                            let actual = state.recv_behavior(wire);
                            self.table_values[start + column_index] = actual;
                            if actual != expected {
                                let msg = format!(
                                    "Expected value of {} for {}, \
                                     but got value of {}.",
                                    expected, port.name, actual
                                );
                                errors.push(state.port_error(loc, msg));
                            }
                        }
                        PortColor::Event => {
                            if !self.has_received_events.contains(&wire) {
                                self.table_values[start + column_index] = NIL;
                                if expected != NIL {
                                    let msg = if port.size == WireSize::Zero {
                                        format!(
                                            "Expected an event for \
                                             {}, but got no event.",
                                            port.name
                                        )
                                    } else {
                                        format!(
                                            "Expected event value of {} for \
                                             {}, but got no event.",
                                            expected, port.name
                                        )
                                    };
                                    errors.push(state.port_error(loc, msg));
                                }
                            }
                        }
                        PortColor::Analog => {
                            let actual_fixed = state.recv_analog(wire);
                            self.table_values[start + column_index] =
                                actual_fixed.to_encoded();
                            let expected_fixed = Fixed::from_encoded(expected);
                            let delta = (actual_fixed - expected_fixed).abs();
                            if delta >= EPSILON {
                                let msg = format!(
                                    "Expected value of {} for {}, \
                                     but got value of {}.",
                                    expected_fixed, port.name, actual_fixed
                                );
                                errors.push(state.port_error(loc, msg));
                            }
                        }
                    }
                }
                column_index += 1;
            }
        }
        debug_assert_eq!(column_index, self.num_columns);
        errors
    }
}

//===========================================================================//
