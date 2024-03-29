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

use super::super::eval::{ChipEval, CircuitState};
use super::data::{AbstractConstraint, ChipData};
use crate::geom::Direction;
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow, WireId};

//===========================================================================//

pub const AND_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Source, PortColor::Behavior, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct AndChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl AndChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), AND_CHIP_DATA.ports.len());
        let chip_eval = AndChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for AndChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        state.send_behavior(self.output, input1 & input2);
    }
}

//===========================================================================//

pub const DEMUX_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Sink, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Source, PortColor::Event, (0, 0), Direction::South),
        (PortFlow::Source, PortColor::Event, (0, 0), Direction::East),
        (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
        AbstractConstraint::Exact(3, WireSize::One),
    ],
    dependencies: &[(0, 1), (0, 2), (3, 1), (3, 2)],
};

pub struct DemuxChipEval {
    input: WireId,
    output1: WireId,
    output2: WireId,
    control: WireId,
}

impl DemuxChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), DEMUX_CHIP_DATA.ports.len());
        let chip_eval = DemuxChipEval {
            input: slots[0].0,
            output1: slots[1].0,
            output2: slots[2].0,
            control: slots[3].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for DemuxChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input) {
            if state.recv_behavior(self.control) != 0 {
                state.send_event(self.output1, value);
            } else {
                state.send_event(self.output2, value);
            }
        }
    }
}

//===========================================================================//

pub const MUX_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Source, PortColor::Behavior, (0, 0), Direction::East),
        (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
        AbstractConstraint::Exact(3, WireSize::One),
    ],
    dependencies: &[(0, 2), (1, 2), (3, 2)],
};

pub struct MuxChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
    control: WireId,
}

impl MuxChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), MUX_CHIP_DATA.ports.len());
        let chip_eval = MuxChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
            control: slots[3].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for MuxChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let output = if state.recv_behavior(self.control) == 0 {
            state.recv_behavior(self.input1)
        } else {
            state.recv_behavior(self.input2)
        };
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//

pub const NOT_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Source, PortColor::Behavior, (0, 0), Direction::East),
    ],
    constraints: &[AbstractConstraint::Equal(0, 1)],
    dependencies: &[(0, 1)],
};

pub struct NotChipEval {
    size: WireSize,
    input: WireId,
    output: WireId,
}

impl NotChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), NOT_CHIP_DATA.ports.len());
        let chip_eval = NotChipEval {
            size: slots[1].1,
            input: slots[0].0,
            output: slots[1].0,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for NotChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input = state.recv_behavior(self.input);
        state.send_behavior(self.output, (!input) & self.size.mask());
    }
}

//===========================================================================//

pub const OR_CHIP_DATA: &ChipData = AND_CHIP_DATA;

pub struct OrChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl OrChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), OR_CHIP_DATA.ports.len());
        let chip_eval = OrChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for OrChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        state.send_behavior(self.output, input1 | input2);
    }
}

//===========================================================================//

pub const RELAY_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Sink, PortColor::Analog, (0, 0), Direction::West),
        (PortFlow::Sink, PortColor::Analog, (0, 0), Direction::South),
        (PortFlow::Source, PortColor::Analog, (0, 0), Direction::East),
        (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::ANALOG),
        AbstractConstraint::Exact(1, WireSize::ANALOG),
        AbstractConstraint::Exact(2, WireSize::ANALOG),
        AbstractConstraint::Exact(3, WireSize::One),
    ],
    dependencies: &[(0, 2), (1, 2), (3, 2)],
};

pub struct RelayChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
    control: WireId,
}

impl RelayChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), RELAY_CHIP_DATA.ports.len());
        let chip_eval = RelayChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
            control: slots[3].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for RelayChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let output = if state.recv_behavior(self.control) == 0 {
            state.recv_analog(self.input1)
        } else {
            state.recv_analog(self.input2)
        };
        state.send_analog(self.output, output);
    }
}

//===========================================================================//

pub const XOR_CHIP_DATA: &ChipData = AND_CHIP_DATA;

pub struct XorChipEval {
    input1: WireId,
    input2: WireId,
    output: WireId,
}

impl XorChipEval {
    pub fn new_evals(
        slots: &[(WireId, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), XOR_CHIP_DATA.ports.len());
        let chip_eval = XorChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for XorChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        state.send_behavior(self.output, input1 ^ input2);
    }
}

//===========================================================================//
