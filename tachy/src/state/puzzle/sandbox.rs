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

use super::super::eval::{CircuitState, PuzzleEval, MAX_CYCLES_PER_TIME_STEP};
use super::super::interface::{Interface, InterfacePort, InterfacePosition};
use crate::geom::{Coords, Direction, Fixed};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow, WireId};

//===========================================================================//

const SINE_PERIOD: u32 = 10;

//===========================================================================//

const STARTUP_PORTS: &[InterfacePort] = &[InterfacePort {
    name: "Init",
    description: "Sends a single event at the start of the first time step.",
    flow: PortFlow::Send,
    color: PortColor::Event,
    size: WireSize::Zero,
}];

pub const ANALOG_INTERFACES: &[Interface] = &[
    Interface {
        name: "Startup Interface",
        description: "Connects to the power supply.",
        side: Direction::North,
        pos: InterfacePosition::Right(0),
        ports: STARTUP_PORTS,
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
            InterfacePort {
                name: "Sine",
                description:
                    "Outputs a sine wave with a period of 10 time steps.",
                flow: PortFlow::Send,
                color: PortColor::Analog,
                size: WireSize::ANALOG,
            },
        ],
    },
];

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
        ports: STARTUP_PORTS,
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

pub struct SandboxAnalogEval {
    init_wire: WireId,
    time_wire: WireId,
    tick_wire: WireId,
    sine_wire: WireId,
}

impl SandboxAnalogEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> SandboxAnalogEval {
        debug_assert_eq!(slots.len(), 2);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 3);
        SandboxAnalogEval {
            init_wire: slots[0][0].1,
            time_wire: slots[1][0].1,
            tick_wire: slots[1][1].1,
            sine_wire: slots[1][2].1,
        }
    }

    fn send_sine(&self, state: &mut CircuitState) {
        let time = (state.time_step() % SINE_PERIOD)
            * MAX_CYCLES_PER_TIME_STEP
            + state.cycle();
        let dtheta = std::f64::consts::TAU
            / ((SINE_PERIOD * MAX_CYCLES_PER_TIME_STEP) as f64);
        let theta = dtheta * (time as f64);
        state.send_analog(self.sine_wire, Fixed::from_f64(theta.sin()));
    }
}

impl PuzzleEval for SandboxAnalogEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        false
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        if state.time_step() == 0 {
            state.send_event(self.init_wire, 0);
        }
        state.send_behavior(self.time_wire, state.time_step() & 0xff);
        state.send_event(self.tick_wire, 0);
        self.send_sine(state);
    }

    fn begin_additional_cycle(&mut self, state: &mut CircuitState) {
        self.send_sine(state);
    }

    fn needs_another_cycle(&self, state: &CircuitState) -> bool {
        state.cycle() + 1 < MAX_CYCLES_PER_TIME_STEP
            && !state.is_null_wire(self.sine_wire)
    }
}

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
