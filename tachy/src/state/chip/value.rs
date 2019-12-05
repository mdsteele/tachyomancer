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
use super::data::{AbstractConstraint, AbstractPort, ChipData};
use crate::geom::Direction;
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow};
use rand;

//===========================================================================//

const COERCE_PORTS: &[AbstractPort] = &[
    (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
    (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
];

const COERCE_CHIP_DATA_1: &ChipData = &ChipData {
    ports: COERCE_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::One),
        AbstractConstraint::Exact(1, WireSize::One),
    ],
    dependencies: &[(0, 1)],
};

const COERCE_CHIP_DATA_2: &ChipData = &ChipData {
    ports: COERCE_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Two),
        AbstractConstraint::Exact(1, WireSize::Two),
    ],
    dependencies: &[(0, 1)],
};

const COERCE_CHIP_DATA_4: &ChipData = &ChipData {
    ports: COERCE_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Four),
        AbstractConstraint::Exact(1, WireSize::Four),
    ],
    dependencies: &[(0, 1)],
};

const COERCE_CHIP_DATA_8: &ChipData = &ChipData {
    ports: COERCE_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Eight),
        AbstractConstraint::Exact(1, WireSize::Eight),
    ],
    dependencies: &[(0, 1)],
};

const COERCE_CHIP_DATA_16: &ChipData = &ChipData {
    ports: COERCE_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Sixteen),
        AbstractConstraint::Exact(1, WireSize::Sixteen),
    ],
    dependencies: &[(0, 1)],
};

pub fn coerce_chip_data(size: WireSize) -> &'static ChipData {
    match size {
        WireSize::Zero | WireSize::One => COERCE_CHIP_DATA_1,
        WireSize::Two => COERCE_CHIP_DATA_2,
        WireSize::Four => COERCE_CHIP_DATA_4,
        WireSize::Eight => COERCE_CHIP_DATA_8,
        WireSize::Sixteen => COERCE_CHIP_DATA_16,
    }
}

pub struct CoerceChipEval {
    input: usize,
    output: usize,
}

impl CoerceChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), 2);
        let chip_eval =
            CoerceChipEval { input: slots[0].0, output: slots[1].0 };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for CoerceChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let value = state.recv_behavior(self.input);
        state.send_behavior(self.output, value);
    }
}

//===========================================================================//

const CONST_PORTS: &[AbstractPort] =
    &[(PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East)];

const CONST_CHIP_DATA_1: &ChipData =
    &ChipData { ports: CONST_PORTS, constraints: &[], dependencies: &[] };

const CONST_CHIP_DATA_2: &ChipData = &ChipData {
    ports: CONST_PORTS,
    constraints: &[AbstractConstraint::AtLeast(0, WireSize::Two)],
    dependencies: &[],
};

const CONST_CHIP_DATA_4: &ChipData = &ChipData {
    ports: CONST_PORTS,
    constraints: &[AbstractConstraint::AtLeast(0, WireSize::Four)],
    dependencies: &[],
};

const CONST_CHIP_DATA_8: &ChipData = &ChipData {
    ports: CONST_PORTS,
    constraints: &[AbstractConstraint::AtLeast(0, WireSize::Eight)],
    dependencies: &[],
};

const CONST_CHIP_DATA_16: &ChipData = &ChipData {
    ports: CONST_PORTS,
    constraints: &[AbstractConstraint::AtLeast(0, WireSize::Sixteen)],
    dependencies: &[],
};

pub fn const_chip_data(value: u16) -> &'static ChipData {
    match WireSize::min_for_value(value) {
        WireSize::Zero | WireSize::One => CONST_CHIP_DATA_1,
        WireSize::Two => CONST_CHIP_DATA_2,
        WireSize::Four => CONST_CHIP_DATA_4,
        WireSize::Eight => CONST_CHIP_DATA_8,
        WireSize::Sixteen => CONST_CHIP_DATA_16,
    }
}

pub struct ConstChipEval {
    output: usize,
    value: u32,
}

impl ConstChipEval {
    pub fn new_evals(
        value: u16,
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), const_chip_data(value).ports.len());
        let chip_eval =
            ConstChipEval { value: value.into(), output: slots[0].0 };
        vec![(0, Box::new(chip_eval))]
    }
}

impl ChipEval for ConstChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        state.send_behavior(self.output, self.value);
    }
}

//===========================================================================//

pub const DISCARD_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::AtLeast(0, WireSize::One),
        AbstractConstraint::Exact(1, WireSize::Zero),
    ],
    dependencies: &[(0, 1)],
};

pub struct DiscardChipEval {
    input: usize,
    output: usize,
}

impl DiscardChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), DISCARD_CHIP_DATA.ports.len());
        let chip_eval =
            DiscardChipEval { input: slots[0].0, output: slots[1].0 };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for DiscardChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input) {
            state.send_event(self.output, 0);
        }
    }
}

//===========================================================================//

pub const JOIN_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Equal(0, 2),
        AbstractConstraint::Equal(1, 2),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct JoinChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl JoinChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), JOIN_CHIP_DATA.ports.len());
        let chip_eval = JoinChipEval {
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for JoinChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input1) {
            state.send_event(self.output, value);
        } else if let Some(value) = state.recv_event(self.input2) {
            state.send_event(self.output, value);
        }
    }
}

//===========================================================================//

pub const PACK_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::North),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Equal(0, 1),
        AbstractConstraint::Double(2, 0),
        AbstractConstraint::Double(2, 1),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct PackChipEval {
    input_size: WireSize,
    input1: usize,
    input2: usize,
    output: usize,
}

impl PackChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), PACK_CHIP_DATA.ports.len());
        let chip_eval = PackChipEval {
            input_size: slots[0].1,
            input1: slots[0].0,
            input2: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for PackChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1);
        let input2 = state.recv_behavior(self.input2);
        let output = input1 | (input2 << self.input_size.num_bits());
        state.send_behavior(self.output, output);
    }
}

//===========================================================================//

pub const RANDOM_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Zero),
        AbstractConstraint::AtLeast(1, WireSize::One),
    ],
    dependencies: &[(0, 1)],
};

pub struct RandomChipEval {
    input: usize,
    output: usize,
    size: WireSize,
}

impl RandomChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), RANDOM_CHIP_DATA.ports.len());
        let chip_eval = RandomChipEval {
            input: slots[0].0,
            output: slots[1].0,
            size: slots[1].1,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for RandomChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input) {
            let value = rand::random::<u32>() & self.size.mask();
            state.send_event(self.output, value);
        }
    }
}

//===========================================================================//

pub const SAMPLE_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::South),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Zero),
        AbstractConstraint::Equal(1, 2),
    ],
    dependencies: &[(0, 2), (1, 2)],
};

pub struct SampleChipEval {
    input_e: usize,
    input_b: usize,
    output: usize,
}

impl SampleChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), SAMPLE_CHIP_DATA.ports.len());
        let chip_eval = SampleChipEval {
            input_e: slots[0].0,
            input_b: slots[1].0,
            output: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for SampleChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input_e) {
            let value = state.recv_behavior(self.input_b);
            state.send_event(self.output, value);
        }
    }
}

//===========================================================================//

pub const UNPACK_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East),
        (PortFlow::Send, PortColor::Behavior, (0, 0), Direction::North),
    ],
    constraints: &[
        AbstractConstraint::Equal(1, 2),
        AbstractConstraint::Double(0, 1),
        AbstractConstraint::Double(0, 2),
    ],
    dependencies: &[(0, 1), (0, 2)],
};

pub struct UnpackChipEval {
    output_size: WireSize,
    input: usize,
    output1: usize,
    output2: usize,
}

impl UnpackChipEval {
    pub fn new_evals(
        slots: &[(usize, WireSize)],
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), UNPACK_CHIP_DATA.ports.len());
        let chip_eval = UnpackChipEval {
            output_size: slots[2].1,
            input: slots[0].0,
            output1: slots[1].0,
            output2: slots[2].0,
        };
        vec![(2, Box::new(chip_eval))]
    }
}

impl ChipEval for UnpackChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input = state.recv_behavior(self.input);
        let output1 = input & self.output_size.mask();
        let output2 = input >> self.output_size.num_bits();
        state.send_behavior(self.output1, output1);
        state.send_behavior(self.output2, output2);
    }
}

//===========================================================================//
