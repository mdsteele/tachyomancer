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

const REAR_POSITION: i32 = 0;
const FRONT_POSITION: i32 = 725594112;
const MIDDLE_POSITION: i32 = (FRONT_POSITION - REAR_POSITION) / 2;

const SPEEDS: &[i32] = &[
    15116544, 20155392, 22674816, 30233088, 40310784, 45349632, 60466176,
    90699264, 120932352, 181398528, 362797056,
];
const INIT_SPEED_INDEX: usize = 2;

//===========================================================================//

pub const INTERFACES: &[Interface] = &[
    Interface {
        name: "Rear Detector",
        description:
            "Connects to the radiation wave detector at the rear of the \
             resonator crystal.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Rear",
            description:
                "Sends an event when the radiation wave reflects off the rear \
                 side.",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Forward Detector",
        description:
            "Connects to the radiation wave detector at the front of the \
             resonator crystal.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Front",
            description:
                "Sends an event when the radiation wave reflects off the \
                 front side.",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Pulse Emitter",
        description: "Connects to the crystal's pulse emitter.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Pulse",
            description:
                "Send 1 here to create a forward pulse.  Send 0 here to \
                 create a backward pulse.",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::One,
        }],
    },
];

//===========================================================================//

pub struct ResonatorEval {
    rear_wire: WireId,
    front_wire: WireId,
    pulse_wire: WireId,
    wave_position: i32,
    wave_dir: i32,
    speed_index: usize,
}

impl ResonatorEval {
    pub fn new(
        slots: Vec<Vec<((Coords, Direction), WireId)>>,
    ) -> ResonatorEval {
        debug_assert_eq!(slots.len(), 3);
        debug_assert_eq!(slots[0].len(), 1);
        debug_assert_eq!(slots[1].len(), 1);
        debug_assert_eq!(slots[2].len(), 1);
        ResonatorEval {
            rear_wire: slots[0][0].1,
            front_wire: slots[1][0].1,
            pulse_wire: slots[2][0].1,
            wave_position: FRONT_POSITION - SPEEDS[INIT_SPEED_INDEX],
            wave_dir: 1,
            speed_index: INIT_SPEED_INDEX,
        }
    }
}

impl PuzzleEval for ResonatorEval {
    fn task_is_completed(&self, _state: &CircuitState) -> bool {
        self.speed_index + 1 >= SPEEDS.len()
    }

    fn begin_time_step(&mut self, state: &mut CircuitState) {
        if self.wave_position == REAR_POSITION {
            state.send_event(self.rear_wire, 0);
        }
        if self.wave_position == FRONT_POSITION {
            state.send_event(self.front_wire, 0);
        }
    }

    fn end_cycle(&mut self, state: &CircuitState) -> Vec<EvalError> {
        if let Some(dir) = state.recv_event(self.pulse_wire) {
            if self.wave_position == MIDDLE_POSITION
                && (dir == 1 && self.wave_dir == 1
                    || dir == 0 && self.wave_dir == -1)
            {
                self.speed_index += 1;
                if self.speed_index % 2 == 0 {
                    self.wave_dir = -self.wave_dir;
                }
                self.wave_position += self.wave_dir;
            } else if self.speed_index > 0 {
                self.speed_index -= 1;
            }
        }
        Vec::new()
    }

    fn end_time_step(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        self.wave_position += SPEEDS[self.speed_index] * self.wave_dir;
        if self.wave_position <= REAR_POSITION {
            self.wave_position = REAR_POSITION;
            self.wave_dir = 1;
        } else if self.wave_position >= FRONT_POSITION {
            self.wave_position = FRONT_POSITION;
            self.wave_dir = -1;
        }
        Vec::new()
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{FRONT_POSITION, REAR_POSITION, SPEEDS};

    #[test]
    fn speeds_divide_semirange() {
        let range = FRONT_POSITION - REAR_POSITION;
        assert_eq!(range % 2, 0);
        let semirange = range / 2;
        for &speed in SPEEDS {
            assert_eq!(semirange % speed, 0);
        }
    }

    #[test]
    fn speeds_are_sorted() {
        let mut prev_speed = 0;
        for &speed in SPEEDS {
            assert!(speed > prev_speed);
            prev_speed = speed;
        }
    }
}

//===========================================================================//
