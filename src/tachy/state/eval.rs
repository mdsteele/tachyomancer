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
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;
use tachy::geom::{Coords, Direction};

//===========================================================================//

#[must_use = "non-`Continue` values must be handled"]
pub enum EvalResult {
    Continue,
    Breakpoint(Vec<Coords>),
    Failure,
    Victory(EvalScore),
}

pub struct EvalError {
    pub time_step: u32,
    pub port: Option<(Coords, Direction)>,
    pub message: String,
}

#[allow(dead_code)]
pub enum EvalScore {
    /// Score is equal to the supplied value.
    Value(i32),
    /// Score is equal to the number of wire fragments in the circuit.
    WireLength,
}

//===========================================================================//

pub struct CircuitEval {
    time_step: u32, // which time step we're on
    cycle: u32, // which cycle of the time step we're on
    subcycle: usize, // index into `chips` of next chip group to eval
    errors: Vec<EvalError>,
    // Topologically-sorted list of chips, divided into parallel groups:
    chips: Vec<Vec<Box<ChipEval>>>,
    puzzle: Box<PuzzleEval>,
    state: CircuitState,
    interact: Rc<RefCell<CircuitInteraction>>,
}

impl CircuitEval {
    pub fn new(num_wires: usize, chip_groups: Vec<Vec<Box<ChipEval>>>,
               puzzle: Box<PuzzleEval>,
               interact: Rc<RefCell<CircuitInteraction>>)
               -> CircuitEval {
        CircuitEval {
            time_step: 0,
            cycle: 0,
            subcycle: 0,
            errors: Vec::new(),
            chips: chip_groups,
            puzzle,
            state: CircuitState::new(num_wires),
            interact,
        }
    }

    pub fn time_step(&self) -> u32 { self.time_step }

    pub fn verification_data(&self) -> &[u64] {
        self.puzzle.verification_data()
    }

    pub fn errors(&self) -> &[EvalError] { &self.errors }

    pub fn interaction(&mut self) -> RefMut<CircuitInteraction> {
        self.interact.borrow_mut()
    }

    pub fn wire_value(&self, wire_index: usize) -> u32 {
        self.state.values[wire_index].0
    }

    pub fn step_subcycle(&mut self) -> EvalResult {
        self.errors.extend(self.puzzle.end_subcycle(&self.state));
        self.state.reset_for_subcycle();
        while !self.state.changed {
            if self.subcycle >= self.chips.len() {
                let mut needs_another_cycle = false;
                for group in self.chips.iter_mut() {
                    for chip in group.iter_mut() {
                        needs_another_cycle |=
                            chip.needs_another_cycle(&self.state);
                    }
                }
                self.subcycle = 0;
                self.cycle += 1;
                self.state.reset_for_cycle();
                if needs_another_cycle {
                    debug_log!("  Cycle {} complete, starting another cycle",
                               self.cycle);
                    self.puzzle.begin_cycle(&mut self.state);
                    return EvalResult::Continue;
                }
                self.errors.extend(self.puzzle.end_time_step(self.time_step,
                                                             &self.state));
                debug_log!("Time step {} complete after {} cycle(s)",
                           self.time_step,
                           self.cycle);
                self.cycle = 0;
                self.time_step += 1;
                return EvalResult::Continue;
            }
            if self.cycle == 0 && self.subcycle == 0 {
                if let Some(score) =
                    self.puzzle
                        .begin_time_step(self.time_step, &mut self.state)
                {
                    return if self.errors.is_empty() {
                        EvalResult::Victory(score)
                    } else {
                        EvalResult::Failure
                    };
                }
            }
            for chip in self.chips[self.subcycle].iter_mut() {
                chip.eval(&mut self.state);
            }
            debug_log!("    Subcycle {} complete, changed={}",
                       self.subcycle,
                       self.state.changed);
            self.subcycle += 1;
            if !self.state.breakpoints.is_empty() {
                debug_log!("Triggered {} breakpoint(s)",
                           self.state.breakpoints.len());
                let coords_vec = mem::replace(&mut self.state.breakpoints,
                                              Vec::new());
                return EvalResult::Breakpoint(coords_vec);
            }
        }
        return EvalResult::Continue;
    }

    pub fn step_cycle(&mut self) -> EvalResult {
        let current_time_step = self.time_step;
        let current_cycle = self.cycle;
        while self.time_step == current_time_step &&
            self.cycle == current_cycle
        {
            match self.step_subcycle() {
                EvalResult::Continue => {}
                result => return result,
            }
        }
        EvalResult::Continue
    }

    pub fn step_time(&mut self) -> EvalResult {
        let current_time_step = self.time_step;
        while self.time_step == current_time_step {
            match self.step_subcycle() {
                EvalResult::Continue => {}
                result => return result,
            }
        }
        EvalResult::Continue
    }
}

//===========================================================================//

pub struct CircuitState {
    values: Vec<(u32, bool)>,
    breakpoints: Vec<Coords>,
    changed: bool,
}

impl CircuitState {
    fn new(num_values: usize) -> CircuitState {
        CircuitState {
            values: vec![(0, false); num_values],
            breakpoints: vec![],
            changed: false,
        }
    }

    pub fn recv_behavior(&self, slot: usize) -> (u32, bool) {
        self.values[slot]
    }

    pub fn recv_event(&self, slot: usize) -> Option<u32> {
        let (value, has_event) = self.values[slot];
        if has_event { Some(value) } else { None }
    }

    pub fn has_event(&self, slot: usize) -> bool { self.values[slot].1 }

    pub fn send_behavior(&mut self, slot: usize, value: u32) {
        if self.values[slot].0 != value {
            self.values[slot] = (value, true);
            self.changed = true; // TODO: don't marked changed for null wires
        }
    }

    pub fn send_event(&mut self, slot: usize, value: u32) {
        self.values[slot] = (value, true);
        self.changed = true; // TODO: don't marked changed for null wires
    }

    pub fn breakpoint(&mut self, coords: Coords) {
        self.breakpoints.push(coords);
    }

    fn reset_for_cycle(&mut self) {
        for &mut (_, ref mut changed) in self.values.iter_mut() {
            *changed = false;
        }
    }

    fn reset_for_subcycle(&mut self) {
        debug_assert!(self.breakpoints.is_empty());
        self.changed = false;
    }
}

//===========================================================================//

/// Stores player interations with the circuit that take place during
/// evaluation (such as pressing button parts on the board).
pub struct CircuitInteraction {
    buttons: HashMap<Coords, i32>,
}

impl CircuitInteraction {
    pub fn new() -> Rc<RefCell<CircuitInteraction>> {
        let interact = CircuitInteraction { buttons: HashMap::new() };
        Rc::new(RefCell::new(interact))
    }

    pub fn press_button(&mut self, coords: Coords) {
        self.buttons.entry(coords).and_modify(|n| *n += 1).or_insert(1);
    }
}

//===========================================================================//

pub trait PuzzleEval {
    /// Returns the opaque data array that should be passed to this puzzle's
    /// verification view.
    fn verification_data(&self) -> &[u64];

    /// Called at the beginning of each time step; sets up input values for the
    /// circuit.
    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> Option<EvalScore>;

    /// Called at the beginning of each cycle; optionally sends additional
    /// events for that time step.  The default implementation is a no-op.
    fn begin_cycle(&mut self, _state: &mut CircuitState) {}

    /// Called at the end of each subcycle; returns a list of errors (if any)
    /// that cause the puzzle to be failed (e.g. if an invalid value was sent
    /// to an interface receiver).  The default implementation always returns
    /// no errors.
    fn end_subcycle(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        Vec::new()
    }

    /// Called at the end of each time step; returns a list of errors (if any)
    /// that cause the puzzle to be failed (e.g. if an invalid value was sent
    /// to an interface receiver).  The default implementation always returns
    /// no errors.
    fn end_time_step(&mut self, _time_step: u32, _state: &CircuitState)
                     -> Vec<EvalError> {
        Vec::new()
    }
}

pub struct NullPuzzleEval();

impl PuzzleEval for NullPuzzleEval {
    fn verification_data(&self) -> &[u64] { &[] }

    fn begin_time_step(&mut self, _step: u32, _state: &mut CircuitState)
                       -> Option<EvalScore> {
        None
    }
}

//===========================================================================//

pub trait ChipEval {
    /// Called once per cycle, sometime during this chip's subcycle; updates
    /// outputs and/or internal state based on inputs.
    fn eval(&mut self, state: &mut CircuitState);

    /// Called at the end of each cycle; returns true if another cycle is
    /// needed.  The default implementation always returns false.
    fn needs_another_cycle(&mut self, _state: &CircuitState) -> bool { false }

    /// Updates internal chip state between time steps.  The default
    /// implementation is a no-op.
    fn on_time_step(&mut self) {}
}

//===========================================================================//
// TODO: Move these to a separate module.

pub struct AddChipEval {
    size: WireSize,
    input1: usize,
    input2: usize,
    output1: usize,
    output2: usize,
}

impl AddChipEval {
    pub fn new(size: WireSize, input1: usize, input2: usize, output1: usize,
               output2: usize)
               -> Box<ChipEval> {
        Box::new(AddChipEval {
                     size,
                     input1,
                     input2,
                     output1,
                     output2,
                 })
    }
}

impl ChipEval for AddChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let (input1, changed1) = state.recv_behavior(self.input1);
        let (input2, changed2) = state.recv_behavior(self.input2);
        if changed1 || changed2 {
            let sum = (input1 as u64) + (input2 as u64);
            let lo = (sum & (self.size.mask() as u64)) as u32;
            let hi = (sum >> self.size.num_bits()) as u32;
            state.send_behavior(self.output1, lo);
            state.send_behavior(self.output2, hi);
        }
    }
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
    fn eval(&mut self, state: &mut CircuitState) {
        let (input1, changed1) = state.recv_behavior(self.input1);
        let (input2, changed2) = state.recv_behavior(self.input2);
        if changed1 || changed2 {
            state.send_behavior(self.output, input1 & input2);
        }
    }
}

pub struct BreakChipEval {
    input: usize,
    output: usize,
    coords: Coords,
}

impl BreakChipEval {
    pub fn new(input: usize, output: usize, coords: Coords) -> Box<ChipEval> {
        Box::new(BreakChipEval {
                     input,
                     output,
                     coords,
                 })
    }
}

impl ChipEval for BreakChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input) {
            state.send_event(self.output, value);
            state.breakpoint(self.coords);
        }
    }
}

pub struct ButtonChipEval {
    output: usize,
    coords: Coords,
    press_count: i32,
    interact: Rc<RefCell<CircuitInteraction>>,
}

impl ButtonChipEval {
    pub fn new(output: usize, coords: Coords,
               interact: Rc<RefCell<CircuitInteraction>>)
               -> Box<ChipEval> {
        Box::new(ButtonChipEval {
                     output,
                     coords,
                     press_count: 0,
                     interact,
                 })
    }
}

impl ChipEval for ButtonChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(count) = self.interact
            .borrow_mut()
            .buttons
            .remove(&self.coords)
        {
            debug_log!("Button at ({}, {}) was pressed {} time(s)",
                       self.coords.x,
                       self.coords.y,
                       count);
            self.press_count += count;
        }
        if self.press_count > 0 {
            self.press_count -= 1;
            state.send_event(self.output, 0);
        }
    }

    fn needs_another_cycle(&mut self, _state: &CircuitState) -> bool {
        self.press_count > 0
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
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input) {
            self.received = true;
        }
        if self.should_send {
            state.send_event(self.output, 0);
            self.should_send = false;
        }
    }

    fn on_time_step(&mut self) {
        self.should_send = self.received;
        self.received = false;
    }
}

pub struct CompareChipEval {
    input1: usize,
    input2: usize,
    output1: usize,
    output2: usize,
}

impl CompareChipEval {
    pub fn new(input1: usize, input2: usize, output1: usize, output2: usize)
               -> Box<ChipEval> {
        Box::new(CompareChipEval {
                     input1,
                     input2,
                     output1,
                     output2,
                 })
    }
}

impl ChipEval for CompareChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let input1 = state.recv_behavior(self.input1).0;
        let input2 = state.recv_behavior(self.input2).0;
        let (output1, output2) = if input1 < input2 { (1, 0) } else { (0, 1) };
        state.send_behavior(self.output1, output1);
        state.send_behavior(self.output2, output2);
    }
}

pub struct ConstChipEval {
    output: usize,
    value: u32,
}

impl ConstChipEval {
    pub fn new(value: u32, output: usize) -> Box<ChipEval> {
        Box::new(ConstChipEval { output, value })
    }
}

impl ChipEval for ConstChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        state.send_behavior(self.output, self.value);
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
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = self.value.take() {
            debug_log!("Delay chip is sending value {}", value);
            state.send_event(self.output, value);
        }
    }

    fn needs_another_cycle(&mut self, state: &CircuitState) -> bool {
        if let Some(value) = state.recv_event(self.input) {
            debug_log!("Delay chip is storing value {}", value);
            self.value = Some(value);
            true
        } else {
            false
        }
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
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input) {
            state.send_event(self.output, 0);
        }
    }
}

pub struct JoinChipEval {
    input1: usize,
    input2: usize,
    output: usize,
}

impl JoinChipEval {
    pub fn new(input1: usize, input2: usize, output: usize) -> Box<ChipEval> {
        Box::new(JoinChipEval {
                     input1,
                     input2,
                     output,
                 })
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

pub struct LatestChipEval {
    input: usize,
    output: usize,
}

impl LatestChipEval {
    pub fn new(input: usize, output: usize) -> Box<ChipEval> {
        Box::new(LatestChipEval { input, output })
    }
}

impl ChipEval for LatestChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input) {
            state.send_behavior(self.output, value);
        }
    }
}

pub struct MuxChipEval {
    input1: usize,
    input2: usize,
    output: usize,
    control: usize,
}

impl MuxChipEval {
    pub fn new(input1: usize, input2: usize, output: usize, control: usize)
               -> Box<ChipEval> {
        Box::new(MuxChipEval {
                     input1,
                     input2,
                     output,
                     control,
                 })
    }
}

impl ChipEval for MuxChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let output = if state.recv_behavior(self.control).0 == 0 {
            state.recv_behavior(self.input1).0
        } else {
            state.recv_behavior(self.input2).0
        };
        state.send_behavior(self.output, output);
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
    fn eval(&mut self, state: &mut CircuitState) {
        let (input, _) = state.recv_behavior(self.input);
        state.send_behavior(self.output, (!input) & self.size.mask());
    }
}

pub struct PackChipEval {
    input_size: WireSize,
    input1: usize,
    input2: usize,
    output: usize,
}

impl PackChipEval {
    pub fn new(input_size: WireSize, input1: usize, input2: usize,
               output: usize)
               -> Box<ChipEval> {
        Box::new(PackChipEval {
                     input_size,
                     input1,
                     input2,
                     output,
                 })
    }
}

impl ChipEval for PackChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let (input1, changed1) = state.recv_behavior(self.input1);
        let (input2, changed2) = state.recv_behavior(self.input2);
        if changed1 || changed2 {
            let output = input1 | (input2 << self.input_size.num_bits());
            state.send_behavior(self.output, output);
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
    fn eval(&mut self, state: &mut CircuitState) {
        let mut storage = self.storage.borrow_mut();
        let (addr, _) = state.recv_behavior(self.input_b);
        if let Some(value) = state.recv_event(self.input_e) {
            storage[addr as usize] = value;
        }
        state.send_behavior(self.output, storage[addr as usize]);
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
    fn eval(&mut self, state: &mut CircuitState) {
        if state.has_event(self.input_e) {
            let (value, _) = state.recv_behavior(self.input_b);
            state.send_event(self.output, value);
        }
    }
}

pub struct UnpackChipEval {
    output_size: WireSize,
    input: usize,
    output1: usize,
    output2: usize,
}

impl UnpackChipEval {
    pub fn new(output_size: WireSize, input: usize, output1: usize,
               output2: usize)
               -> Box<ChipEval> {
        Box::new(UnpackChipEval {
                     output_size,
                     input,
                     output1,
                     output2,
                 })
    }
}

impl ChipEval for UnpackChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        let (input, changed) = state.recv_behavior(self.input);
        if changed {
            let output1 = input & self.output_size.mask();
            let output2 = input >> self.output_size.num_bits();
            state.send_behavior(self.output1, output1);
            state.send_behavior(self.output2, output2);
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{AndChipEval, ChipEval, CircuitEval, CircuitInteraction,
                NotChipEval, NullPuzzleEval, WireSize};

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
        let mut eval = CircuitEval::new(6,
                                        chips,
                                        Box::new(NullPuzzleEval()),
                                        CircuitInteraction::new());
        for &inputs in &[(0, 0), (0, 1), (1, 0), (1, 1)] {
            eval.state.values[0] = (inputs.0, true);
            eval.state.values[1] = (inputs.1, true);
            let _ = eval.step_time();
            let output = eval.state.values[5].0;
            assert_eq!(output, inputs.0 | inputs.1, "inputs: {:?}", inputs);
        }
    }
}

//===========================================================================//
