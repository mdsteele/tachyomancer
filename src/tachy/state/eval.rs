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

use super::size::WireSize;
use std::cell::RefCell;
use std::rc::Rc;

//===========================================================================//

pub struct CircuitEval {
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

    pub fn wire_value(&self, wire_index: usize) -> u32 {
        self.state.values[wire_index].0
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

pub struct CircuitState {
    pub values: Vec<(u32, bool)>,
    pub needs_another_cycle: bool,
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

pub trait ChipEval {
    /// If any chip inputs have changed/fired, updates outputs and/or internal
    /// state; returns true if any outputs were updated.
    fn eval(&mut self, state: &mut CircuitState) -> bool;

    /// Updates internal chip state for the next time step.  The default
    /// implementation is a no-op.
    fn on_time_step(&mut self) {}
}

pub struct AndChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl AndChipEval {
    pub fn new(input1: usize, input2: usize, output: usize) -> Box<ChipEval> {
        Box::new(AndChipEval {
                     input1,
                     input2,
                     output,
                 })
    }
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

pub struct ClockChipEval {
    input: usize,
    output: usize,
    received: bool,
    should_send: bool,
}

impl ClockChipEval {
    pub fn new(input: usize, output: usize) -> Box<ChipEval> {
        Box::new(ClockChipEval {
                     input,
                     output,
                     received: false,
                     should_send: false,
                 })
    }
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

pub struct ConstChipEval {
    output: usize,
    value: u32,
    should_send: bool,
}

impl ConstChipEval {
    pub fn new(value: u32, output: usize) -> Box<ChipEval> {
        Box::new(ConstChipEval {
                     output,
                     value,
                     should_send: true,
                 })
    }
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

pub struct DelayChipEval {
    input: usize,
    output: usize,
    value: Option<u32>,
}

impl DelayChipEval {
    pub fn new(input: usize, output: usize) -> Box<ChipEval> {
        Box::new(DelayChipEval {
                     input,
                     output,
                     value: None,
                 })
    }
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

pub struct DiscardChipEval {
    input: usize,
    output: usize,
}

impl DiscardChipEval {
    pub fn new(input: usize, output: usize) -> Box<ChipEval> {
        Box::new(DiscardChipEval { input, output })
    }
}

impl ChipEval for DiscardChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        let has_event = state.values[self.input].1;
        if has_event {
            state.values[self.output] = (0, true);
        }
        has_event
    }
}

pub struct NotChipEval {
    size: WireSize,
    input: usize,
    output: usize,
}

impl NotChipEval {
    pub fn new(size: WireSize, input: usize, output: usize) -> Box<ChipEval> {
        Box::new(NotChipEval {
                     size,
                     input,
                     output,
                 })
    }
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

pub struct PackChipEval {
    output_size: WireSize,
    input1: usize,
    input2: usize,
    output: usize,
}

impl PackChipEval {
    pub fn new(output_size: WireSize, input1: usize, input2: usize,
               output: usize)
               -> Box<ChipEval> {
        Box::new(PackChipEval {
                     output_size,
                     input1,
                     input2,
                     output,
                 })
    }
}

impl ChipEval for PackChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        let (input1, changed1) = state.values[self.input1];
        let (input2, changed2) = state.values[self.input2];
        if changed1 || changed2 {
            let num_bits = self.output_size.num_bits();
            let output = input1 | (input2 << (num_bits / 2));
            state.values[self.output] = (output, true);
            true
        } else {
            false
        }
    }
}

pub struct RamChipEval {
    input_b: usize,
    input_e: usize,
    output: usize,
    storage: Rc<RefCell<Vec<u32>>>,
}

impl RamChipEval {
    pub fn new(input_b: usize, input_e: usize, output: usize,
               storage: Rc<RefCell<Vec<u32>>>)
               -> Box<ChipEval> {
        Box::new(RamChipEval {
                     input_b,
                     input_e,
                     output,
                     storage,
                 })
    }
}

impl ChipEval for RamChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        let (addr, addr_changed) = state.values[self.input_b];
        let (value, has_event) = state.values[self.input_e];
        let mut storage = self.storage.borrow_mut();
        if has_event {
            storage[addr as usize] = value;
        }
        if has_event || addr_changed {
            state.values[self.output] = (storage[addr as usize], true);
            true
        } else {
            false
        }
    }
}

pub struct SampleChipEval {
    input_e: usize,
    input_b: usize,
    output: usize,
}

impl SampleChipEval {
    pub fn new(input_e: usize, input_b: usize, output: usize)
               -> Box<ChipEval> {
        Box::new(SampleChipEval {
                     input_e,
                     input_b,
                     output,
                 })
    }
}

impl ChipEval for SampleChipEval {
    fn eval(&mut self, state: &mut CircuitState) -> bool {
        let has_event = state.values[self.input_e].1;
        if has_event {
            state.values[self.output] = (state.values[self.input_b].0, true);
        }
        has_event
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
