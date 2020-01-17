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

use crate::geom::{Coords, Direction};
use crate::save::HotkeyCode;
use downcast_rs::{impl_downcast, Downcast};
use std::collections::{HashMap, HashSet};
use std::mem;

//===========================================================================//

const MAX_CYCLES_PER_TIME_STEP: u32 = 1000;

//===========================================================================//

#[derive(Debug)]
#[must_use = "non-`Continue` values must be handled"]
pub enum EvalResult {
    Continue,
    Breakpoint(Vec<Coords>),
    Failure,
    Victory(EvalScore),
}

#[derive(Debug)]
pub struct EvalError {
    pub time_step: u32,
    pub port: Option<(Coords, Direction)>,
    pub fatal: bool,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvalScore {
    /// Score is equal to the number of cycles.
    Cycles,
    /// Score is equal to the supplied value.
    Value(u32),
    /// Score is equal to the number of wire fragments in the circuit.
    WireLength,
}

//===========================================================================//

pub struct CircuitEval {
    total_cycles: u32,
    cycle: u32,      // which cycle of the time step we're on
    subcycle: usize, // index into `chips` of next chip group to eval
    errors: Vec<EvalError>,
    // Topologically-sorted list of chips, divided into parallel groups:
    chips: Vec<Vec<Box<dyn ChipEval>>>,
    puzzle: Box<dyn PuzzleEval>,
    state: CircuitState,
    // Maps from coords to indices into the chips vec for chips that need it.
    coords_map: HashMap<Coords, (usize, usize)>,
}

impl CircuitEval {
    pub fn new(
        num_wires: usize,
        null_wires: HashSet<usize>,
        chip_groups: Vec<Vec<Box<dyn ChipEval>>>,
        puzzle: Box<dyn PuzzleEval>,
    ) -> CircuitEval {
        let mut coords_map = HashMap::new();
        for (group_index, group) in chip_groups.iter().enumerate() {
            for (chip_index, chip_eval) in group.iter().enumerate() {
                if let Some(coords) = chip_eval.coords() {
                    coords_map.insert(coords, (group_index, chip_index));
                }
            }
        }
        CircuitEval {
            total_cycles: 0,
            cycle: 0,
            subcycle: 0,
            errors: Vec::new(),
            chips: chip_groups,
            puzzle,
            state: CircuitState::new(num_wires, null_wires),
            coords_map,
        }
    }

    pub fn seconds_per_time_step(&self) -> f64 {
        self.puzzle.seconds_per_time_step()
    }

    pub fn time_step(&self) -> u32 {
        self.state.time_step
    }

    pub fn total_cycles(&self) -> u32 {
        self.total_cycles
    }

    pub fn subcycle(&self) -> usize {
        self.subcycle
    }

    /// Returns the PuzzleEval object, which must have the specified type.
    /// Panics if the incorrect type is specified.
    pub fn puzzle_eval<T: PuzzleEval>(&self) -> &T {
        self.puzzle.downcast_ref::<T>().unwrap()
    }

    pub fn errors(&self) -> &[EvalError] {
        &self.errors
    }

    pub fn press_button(&mut self, coords: Coords, sublocation: u32) {
        if let Some(&(group, index)) = self.coords_map.get(&coords) {
            self.chips[group][index].on_press(sublocation);
        }
    }

    pub fn press_hotkey(&mut self, code: HotkeyCode) {
        self.state.press_hotkey(code);
    }

    pub fn wire_event(&self, wire_index: usize) -> Option<u32> {
        self.state.recv_event(wire_index)
    }

    pub fn wire_value(&self, wire_index: usize) -> u32 {
        self.state.values[wire_index].0
    }

    pub fn wire_has_change(&self, wire_index: usize) -> bool {
        self.state.values[wire_index].1
    }

    /// Returns display data for the chip at the given coordinates, if any.
    pub fn display_data(&self, coords: Coords) -> &[u8] {
        if let Some(&(group, index)) = self.coords_map.get(&coords) {
            self.chips[group][index].display_data()
        } else {
            &[]
        }
    }

    /// Appends the given errors and returns true if any were fatal.
    fn errors_are_fatal(&mut self, errors: Vec<EvalError>) -> bool {
        let fatal = errors.iter().any(|error| error.fatal);
        self.errors.extend(errors);
        fatal
    }

    pub fn step_subcycle(&mut self) -> EvalResult {
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
                let errors = self.puzzle.end_cycle(&self.state);
                if self.errors_are_fatal(errors) {
                    return EvalResult::Failure;
                }
                needs_another_cycle |=
                    self.puzzle.needs_another_cycle(self.time_step());
                if self.cycle + 1 >= MAX_CYCLES_PER_TIME_STEP {
                    self.errors.push(self.state.fatal_error(format!(
                        "Exceeded {} cycles",
                        MAX_CYCLES_PER_TIME_STEP
                    )));
                    return EvalResult::Failure;
                }
                self.subcycle = 0;
                self.cycle += 1;
                self.total_cycles += 1;
                if needs_another_cycle {
                    debug_log!(
                        "  Cycle {} complete, starting another cycle",
                        self.cycle
                    );
                    self.state.reset_for_cycle();
                    self.puzzle.begin_additional_cycle(&mut self.state);
                    return EvalResult::Continue;
                }
                let errors = self.puzzle.end_time_step(&self.state);
                if self.errors_are_fatal(errors) {
                    return EvalResult::Failure;
                }
                for group in self.chips.iter_mut() {
                    for chip in group.iter_mut() {
                        chip.on_time_step();
                    }
                }
                debug_log!(
                    "Time step {} complete after {} cycle(s)",
                    self.time_step(),
                    self.cycle
                );
                self.state.reset_for_cycle();
                self.cycle = 0;
                self.state.time_step += 1;
                return EvalResult::Continue;
            }
            if self.cycle == 0 && self.subcycle == 0 {
                if let Some(score) =
                    self.puzzle.begin_time_step(&mut self.state)
                {
                    return if self.errors.is_empty() {
                        EvalResult::Victory(score)
                    } else {
                        debug_log!("Errors: {:?}", self.errors);
                        EvalResult::Failure
                    };
                }
            }
            for chip in self.chips[self.subcycle].iter_mut() {
                chip.eval(&mut self.state);
            }
            debug_log!(
                "    Subcycle {} complete, changed={}",
                self.subcycle,
                self.state.changed
            );
            self.subcycle += 1;
            if !self.state.breakpoints.is_empty() {
                debug_log!(
                    "Triggered {} breakpoint(s)",
                    self.state.breakpoints.len()
                );
                let coords_vec =
                    mem::replace(&mut self.state.breakpoints, Vec::new());
                return EvalResult::Breakpoint(coords_vec);
            }
        }
        return EvalResult::Continue;
    }

    pub fn step_cycle(&mut self) -> EvalResult {
        let current_time_step = self.time_step();
        let current_cycle = self.cycle;
        while self.time_step() == current_time_step
            && self.cycle == current_cycle
        {
            match self.step_subcycle() {
                EvalResult::Continue => {}
                result => return result,
            }
        }
        EvalResult::Continue
    }

    pub fn step_time(&mut self) -> EvalResult {
        let current_time_step = self.time_step();
        while self.time_step() == current_time_step {
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
    time_step: u32,
    // "Null" wires are ports that have no wire fragments connected to them.
    // We treat them as wires for ease of evaluation, but we don't count the
    // circuit state as having "changed" for the purposes of debug stepping
    // when one of these ports changes values.
    null_wires: HashSet<usize>,
    values: Vec<(u32, bool)>,
    breakpoints: Vec<Coords>,
    hotkey_presses: HashMap<HotkeyCode, i32>,
    changed: bool,
}

impl CircuitState {
    fn new(num_values: usize, null_wires: HashSet<usize>) -> CircuitState {
        CircuitState {
            time_step: 0,
            null_wires,
            values: vec![(0, false); num_values],
            breakpoints: vec![],
            hotkey_presses: HashMap::new(),
            changed: false,
        }
    }

    pub fn time_step(&self) -> u32 {
        self.time_step
    }

    pub fn recv_behavior(&self, slot: usize) -> u32 {
        self.values[slot].0
    }

    pub fn behavior_changed(&self, slot: usize) -> bool {
        self.values[slot].1
    }

    pub fn recv_event(&self, slot: usize) -> Option<u32> {
        let (value, has_event) = self.values[slot];
        if has_event {
            Some(value)
        } else {
            None
        }
    }

    pub fn has_event(&self, slot: usize) -> bool {
        self.values[slot].1
    }

    pub fn send_behavior(&mut self, slot: usize, value: u32) {
        if self.values[slot].0 != value {
            self.values[slot] = (value, true);
            self.changed = !self.null_wires.contains(&slot);
        }
    }

    pub fn send_event(&mut self, slot: usize, value: u32) {
        self.values[slot] = (value, true);
        self.changed = !self.null_wires.contains(&slot);
    }

    pub fn breakpoint(&mut self, coords: Coords) {
        self.breakpoints.push(coords);
    }

    fn press_hotkey(&mut self, code: HotkeyCode) {
        self.hotkey_presses.entry(code).and_modify(|n| *n += 1).or_insert(1);
    }

    pub fn pop_hotkey_presses(&mut self, code: HotkeyCode) -> i32 {
        self.hotkey_presses.remove(&code).unwrap_or(0)
    }

    pub fn fatal_error(&self, message: String) -> EvalError {
        EvalError {
            time_step: self.time_step,
            port: None,
            fatal: true,
            message,
        }
    }

    pub fn port_error(
        &self,
        port: (Coords, Direction),
        message: String,
    ) -> EvalError {
        EvalError {
            time_step: self.time_step,
            port: Some(port),
            fatal: false,
            message,
        }
    }

    pub fn fatal_port_error(
        &self,
        port: (Coords, Direction),
        message: String,
    ) -> EvalError {
        EvalError {
            time_step: self.time_step,
            port: Some(port),
            fatal: true,
            message,
        }
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

pub trait PuzzleEval: Downcast {
    fn seconds_per_time_step(&self) -> f64 {
        0.1
    }

    /// Called at the beginning of each time step; sets up input values for the
    /// circuit.
    fn begin_time_step(
        &mut self,
        state: &mut CircuitState,
    ) -> Option<EvalScore>;

    /// Called at the beginning of each cycle except the first; optionally
    /// sends additional events for that time step.  The default implementation
    /// is a no-op.
    fn begin_additional_cycle(&mut self, _state: &mut CircuitState) {}

    /// Called at the end of each cycle; returns a list of errors (if any) that
    /// cause the puzzle to be failed (e.g. if an invalid value was sent to an
    /// interface receiver).  The default implementation always returns no
    /// errors.
    ///
    /// This is the method that should be used for receiving events at puzzle
    /// interface ports.
    fn end_cycle(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        Vec::new()
    }

    /// Called after end_cycle(); returns true if another cycle is needed.  The
    /// default implementation always returns false.
    fn needs_another_cycle(&self, _time_step: u32) -> bool {
        false
    }

    /// Called at the end of each time step; returns a list of errors (if any)
    /// that cause the puzzle to be failed (e.g. if an invalid value was sent
    /// to an interface receiver).  The default implementation always returns
    /// no errors.
    ///
    /// This is the method that should be used for receiving behavior values at
    /// puzzle interface ports.
    fn end_time_step(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        Vec::new()
    }
}
impl_downcast!(PuzzleEval);

//===========================================================================//

pub trait ChipEval {
    /// Called once per cycle, sometime during this chip's subcycle; updates
    /// outputs and/or internal state based on inputs.
    fn eval(&mut self, state: &mut CircuitState);

    /// Called at the end of each cycle; returns true if another cycle is
    /// needed.  The default implementation always returns false.
    fn needs_another_cycle(&mut self, _state: &CircuitState) -> bool {
        false
    }

    /// Updates internal chip state between time steps.  The default
    /// implementation is a no-op.
    fn on_time_step(&mut self) {}

    /// Returns the coords that this chip is at.  Chip types that override
    /// `display_data` or `on_press` must also override this to return a
    /// non-`None` value; other chips don't need to implement this.
    fn coords(&self) -> Option<Coords> {
        None
    }

    /// Provides chip-type-specific data used to help draw the state of the
    /// chip during evaluation.  For example, the `Screen` chip uses this to
    /// provide the contents of the screen cells.
    fn display_data(&self) -> &[u8] {
        &[]
    }

    /// Handle the chip being clicked with the mouse during evaluation.  The
    /// `sublocation` parameter is a chip-type-specific way to indicate which
    /// part of the chip was clicked, for chip types that have multiple
    /// clickable parts.  For example, the `Screen` chip uses the sublocation
    /// to know which cell of the screen was clicked.
    fn on_press(&mut self, _sublocation: u32) {}
}

//===========================================================================//
