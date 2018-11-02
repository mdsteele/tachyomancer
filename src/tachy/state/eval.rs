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

use cgmath::Bounded;

//===========================================================================//

struct CircuitEval {
    time_step: u32, // which time step we're on
    cycle: u32, // which cycle of the time step we're on
    subcycle: usize, // index into `chips` of next chip group to eval
    // Topologically-sorted list of chips, divided into parallel groups:
    chips: Vec<Vec<Box<ChipEval>>>,
    state: CircuitState,
}

impl CircuitEval {
    pub fn new(num_wires: usize, chip_groups: Vec<Vec<Box<ChipEval>>>)
               -> CircuitEval {
        CircuitEval {
            time_step: 0,
            cycle: 0,
            subcycle: 0,
            chips: chip_groups,
            state: CircuitState::new(num_wires),
        }
    }

    fn eval_subcycle(&mut self) {
        let mut changed = false;
        while !changed && self.subcycle < self.chips.len() {
            for chip in self.chips[self.subcycle].iter_mut() {
                changed |= chip.eval(&mut self.state);
            }
            self.subcycle += 1;
        }
    }

    fn eval_cycle(&mut self) {
        while self.subcycle < self.chips.len() {
            self.eval_subcycle();
        }
        self.cycle += 1;
        self.subcycle = 0;
    }

    pub fn eval_time_step(&mut self) {
        loop {
            self.eval_cycle();
            if !self.state.needs_another_cycle {
                break;
            }
            self.state.reset_for_cycle();
        }
        self.time_step += 1;
        self.cycle = 0;
        for group in self.chips.iter_mut() {
            for chip in group.iter_mut() {
                chip.on_time_step();
            }
        }
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WireSize {
    Zero,
    One,
    Two,
    Four,
    Eight,
    Sixteen,
    ThirtyTwo,
}

impl WireSize {
    fn mask(&self) -> u32 {
        match &self {
            WireSize::Zero => 0x0,
            WireSize::One => 0x1,
            WireSize::Two => 0x3,
            WireSize::Four => 0xf,
            WireSize::Eight => 0xff,
            WireSize::Sixteen => 0xffff,
            WireSize::ThirtyTwo => 0xffff_ffff,
        }
    }
}

impl Bounded for WireSize {
    fn min_value() -> WireSize { WireSize::Zero }
    fn max_value() -> WireSize { WireSize::ThirtyTwo }
}

//===========================================================================//

struct CircuitState {
    values: Vec<(u32, bool)>,
    needs_another_cycle: bool,
}

impl CircuitState {
    fn new(num_values: usize) -> CircuitState {
        CircuitState {
            values: vec![(0, false); num_values],
            needs_another_cycle: false,
        }
    }

    fn reset_for_cycle(&mut self) {
        for &mut (_, ref mut changed) in self.values.iter_mut() {
            *changed = false;
        }
        self.needs_another_cycle = false;
    }
}

//===========================================================================//

trait ChipEval {
    /// If any chip inputs have changed/fired, updates outputs and/or internal
    /// state; returns true if any outputs were updated.
    fn eval(&mut self, state: &mut CircuitState) -> bool;

    /// Updates internal chip state for the next time step.  The default
    /// implementation is a no-op.
    fn on_time_step(&mut self) {}
}

struct AndChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl ChipEval for AndChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        let (input1, changed1) = state.values[self.input1];
        let (input2, changed2) = state.values[self.input2];
        if changed1 || changed2 {
            state.values[self.output] = (input1 & input2, true);
            true
        } else {
            false
        }
    }
}

struct ClockChipEval {
    input: usize,
    output: usize,
    received: bool,
    should_send: bool,
}

impl ChipEval for ClockChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        if state.values[self.input].1 {
            self.received = true;
        }
        if self.should_send {
            state.values[self.output] = (0, true);
            self.should_send = false;
            true
        } else {
            false
        }
    }

    fn on_time_step(&mut self) {
        self.should_send = self.received;
        self.received = false;
    }
}

struct ConstChipEval {
    output: usize,
    value: u32,
    should_send: bool,
}

impl ChipEval for ConstChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        if self.should_send {
            state.values[self.output] = (self.value, true);
            self.should_send = false;
            true
        } else {
            false
        }
    }
}

struct DelayChipEval {
    input: usize,
    output: usize,
    value: Option<u32>,
}

impl ChipEval for DelayChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        let updated = if let Some(value) = self.value.take() {
            state.values[self.output] = (value, true);
            true
        } else {
            false
        };
        let (value, has_event) = state.values[self.input];
        if has_event {
            self.value = Some(value);
            state.needs_another_cycle = true;
        }
        updated
    }
}

struct NotChipEval {
    size: WireSize,
    input: usize,
    output: usize,
}

impl ChipEval for NotChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        let (input, changed) = state.values[self.input];
        if changed {
            let output = (!input) & self.size.mask();
            state.values[self.output] = (output, true);
        }
        changed
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{AndChipEval, ChipEval, CircuitEval, NotChipEval, WireSize};

    #[test]
    fn evaluate_boolean_or_circuit() {
        let chips: Vec<Vec<Box<ChipEval>>> = vec![
            vec![
                Box::new(NotChipEval {
                             size: WireSize::One,
                             input: 0,
                             output: 2,
                         }),
                Box::new(NotChipEval {
                             size: WireSize::One,
                             input: 1,
                             output: 3,
                         }),
            ],
            vec![
                Box::new(AndChipEval {
                             input1: 2,
                             input2: 3,
                             output: 4,
                         }),
            ],
            vec![
                Box::new(NotChipEval {
                             size: WireSize::One,
                             input: 4,
                             output: 5,
                         }),
            ],
        ];
        let mut eval = CircuitEval::new(6, chips);
        for &inputs in &[(0, 0), (0, 1), (1, 0), (1, 1)] {
            eval.state.values[0] = (inputs.0, true);
            eval.state.values[1] = (inputs.1, true);
            eval.eval_time_step();
            let output = eval.state.values[5].0;
            assert_eq!(output, inputs.0 | inputs.1, "inputs: {:?}", inputs);
        }
    }
}

//===========================================================================//
