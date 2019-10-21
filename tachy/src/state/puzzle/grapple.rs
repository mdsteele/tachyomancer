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
use super::iface::{Interface, InterfacePort, InterfacePosition};
use crate::geom::{Coords, Direction};
use crate::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

const CHARGE_DELTA_PER_TIME_STEP: u32 = 5;
const MIN_CHARGE_TO_FIRE: u32 = 40;
const MAX_CHARGE_TO_FIRE: u32 = 60;
const MAX_CHARGE: u32 = 100;

const STARTING_CHARGES: &[(u32, u32)] = &[
    (50, 55),
    (90, 15),
    (30, 95),
    (5, 25),
    (80, 5),
    (75, 85),
    (15, 10),
    (5, 75),
    (45, 50),
    (95, 80),
];

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Port Coil Interface",
        description:
            "Controls the port-side magnetic coil for the grapple launcher.",
        side: Direction::West,
        pos: InterfacePosition::Right(1),
        ports: &[
            InterfacePort {
                name: "Ctrl",
                description: "Controls the port coil charge.\n    \
                              Send 0 to do nothing.\n    \
                              Send 1 to decrease the charge.\n    \
                              Send 2 to increase the charge.\n    \
                              Send 3 to fire the coil.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
            InterfacePort {
                name: "Chrg",
                description:
                    "Outputs the current charge for the port coil (0-100).",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
        ],
    },
    Interface {
        name: "Starboard Coil Interface",
        description:
            "Controls the starboard-side magnetic coil for the grapple \
             launcher.",
        side: Direction::East,
        pos: InterfacePosition::Left(1),
        ports: &[
            InterfacePort {
                name: "Ctrl",
                description: "Controls the starboard coil charge.\n    \
                              Send 0 to do nothing.\n    \
                              Send 1 to decrease the charge.\n    \
                              Send 2 to increase the charge.\n    \
                              Send 3 to fire the coil.",
                flow: PortFlow::Recv,
                color: PortColor::Behavior,
                size: WireSize::Two,
            },
            InterfacePort {
                name: "Chrg",
                description:
                    "Outputs the current charge for the starboard coil \
                     (0-100).",
                flow: PortFlow::Send,
                color: PortColor::Behavior,
                size: WireSize::Eight,
            },
        ],
    },
];

//===========================================================================//

pub struct GrappleEval {
    port_ctrl_port: (Coords, Direction),
    port_ctrl_wire: usize,
    port_charge_wire: usize,
    stbd_ctrl_port: (Coords, Direction),
    stbd_ctrl_wire: usize,
    stbd_charge_wire: usize,
    current_port_charge: u32,
    current_stbd_charge: u32,
    num_coils_fired: usize,
}

impl GrappleEval {
    pub fn new(slots: Vec<Vec<((Coords, Direction), usize)>>) -> GrappleEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 2);
        debug_assert_eq!(slots[1].len(), 2);
        GrappleEval {
            port_ctrl_port: slots[0][0].0,
            port_ctrl_wire: slots[0][0].1,
            port_charge_wire: slots[0][1].1,
            stbd_ctrl_port: slots[1][0].0,
            stbd_ctrl_wire: slots[1][0].1,
            stbd_charge_wire: slots[1][1].1,
            current_port_charge: STARTING_CHARGES[0].0,
            current_stbd_charge: STARTING_CHARGES[0].1,
            num_coils_fired: 0,
        }
    }

    pub fn current_port_charge(&self) -> u32 {
        self.current_port_charge
    }

    pub fn current_stbd_charge(&self) -> u32 {
        self.current_stbd_charge
    }

    pub fn num_coils_fired(&self) -> usize {
        self.num_coils_fired
    }
}

impl PuzzleEval for GrappleEval {
    fn begin_time_step(
        &mut self,
        time_step: u32,
        state: &mut CircuitState,
    ) -> Option<EvalScore> {
        state.send_behavior(self.port_charge_wire, self.current_port_charge);
        state.send_behavior(self.stbd_charge_wire, self.current_stbd_charge);
        if self.num_coils_fired >= STARTING_CHARGES.len() {
            Some(EvalScore::Value(time_step))
        } else {
            None
        }
    }

    fn end_time_step(
        &mut self,
        time_step: u32,
        state: &CircuitState,
    ) -> Vec<EvalError> {
        let mut errors = Vec::new();
        let old_port_charge = self.current_port_charge;
        let old_stbd_charge = self.current_stbd_charge;
        let mut fired = false;
        fired |= coil_control(
            time_step,
            self.port_ctrl_port,
            state.recv_behavior(self.port_ctrl_wire).0,
            &mut self.current_port_charge,
            old_stbd_charge,
            &mut errors,
        );
        fired |= coil_control(
            time_step,
            self.stbd_ctrl_port,
            state.recv_behavior(self.stbd_ctrl_wire).0,
            &mut self.current_stbd_charge,
            old_port_charge,
            &mut errors,
        );
        if fired {
            self.num_coils_fired += 1;
            if self.num_coils_fired < STARTING_CHARGES.len() {
                let (port, stbd) = STARTING_CHARGES[self.num_coils_fired];
                self.current_port_charge = port;
                self.current_stbd_charge = stbd;
            }
        }
        errors
    }
}

fn coil_control(
    time_step: u32,
    port: (Coords, Direction),
    ctrl: u32,
    current_charge: &mut u32,
    other_charge: u32,
    errors: &mut Vec<EvalError>,
) -> bool {
    match ctrl {
        0x1 => {
            *current_charge =
                current_charge.saturating_sub(CHARGE_DELTA_PER_TIME_STEP);
        }
        0x2 => {
            *current_charge =
                (*current_charge + CHARGE_DELTA_PER_TIME_STEP).min(MAX_CHARGE);
        }
        0x3 => {
            if *current_charge < MIN_CHARGE_TO_FIRE
                || *current_charge > MAX_CHARGE_TO_FIRE
            {
                let message = format!(
                    "Can't fire coil: charge ({}) is not in {}-{} range.",
                    *current_charge, MIN_CHARGE_TO_FIRE, MAX_CHARGE_TO_FIRE,
                );
                let error = EvalError { time_step, port: Some(port), message };
                errors.push(error);
                return false;
            }
            if other_charge >= MIN_CHARGE_TO_FIRE
                && other_charge <= MAX_CHARGE_TO_FIRE
            {
                let message = format!(
                    "Can't fire coil: other coil's charge ({}) is not \
                     outside {}-{} range.",
                    other_charge, MIN_CHARGE_TO_FIRE, MAX_CHARGE_TO_FIRE,
                );
                let error = EvalError { time_step, port: Some(port), message };
                errors.push(error);
                return false;
            }
            return true;
        }
        _ => {}
    }
    return false;
}

//===========================================================================//
