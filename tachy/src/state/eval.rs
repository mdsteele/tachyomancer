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

use super::check::WireId;
use crate::geom::{Coords, Direction, Fixed};
use crate::save::{HotkeyCode, InputsData, ScoreUnits};
use downcast_rs::{impl_downcast, Downcast};
use std::collections::{HashMap, HashSet};
use std::mem;

//===========================================================================//

pub const MAX_CYCLES_PER_TIME_STEP: u32 = 1000;

//===========================================================================//

#[derive(Debug)]
#[must_use = "non-`Continue` values must be handled"]
pub enum EvalResult {
    Continue,
    Breakpoint(Vec<Coords>),
    Failure,
    Victory(u32),
}

#[derive(Debug)]
pub struct EvalError {
    pub time_step: u32,
    pub port: Option<(Coords, Direction)>,
    pub fatal: bool,
    pub message: String,
}

//===========================================================================//

pub struct CircuitEval {
    total_cycles: u32,
    subcycle: usize, // index into `chips` of next chip group to eval
    errors: Vec<EvalError>,
    // Topologically-sorted list of chips, divided into parallel groups:
    chips: Vec<Vec<Box<dyn ChipEval>>>,
    wire_length: u32,
    puzzle_eval: Box<dyn PuzzleEval>,
    score_units: ScoreUnits,
    state: CircuitState,
    // Maps from coords to indices into the chips vec for chips that need it.
    coords_map: HashMap<Coords, (usize, usize)>,
}

impl CircuitEval {
    pub fn new(
        num_wire_fragments: usize,
        num_wires: usize,
        null_wires: HashSet<WireId>,
        chip_groups: Vec<Vec<Box<dyn ChipEval>>>,
        puzzle_eval: Box<dyn PuzzleEval>,
        score_units: ScoreUnits,
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
            subcycle: 0,
            errors: Vec::new(),
            chips: chip_groups,
            wire_length: num_wire_fragments as u32,
            puzzle_eval,
            score_units,
            state: CircuitState::new(num_wires, null_wires),
            coords_map,
        }
    }

    pub fn seconds_per_time_step(&self) -> f64 {
        self.puzzle_eval.seconds_per_time_step()
    }

    pub fn time_step(&self) -> u32 {
        self.state.time_step
    }

    pub fn cycle(&self) -> u32 {
        self.state.cycle
    }

    pub fn subcycle(&self) -> usize {
        self.subcycle
    }

    /// Returns the PuzzleEval object, which must have the specified type.
    /// Panics if the incorrect type is specified.
    pub fn puzzle_eval<T: PuzzleEval>(&self) -> &T {
        self.puzzle_eval.downcast_ref::<T>().unwrap()
    }

    pub fn errors(&self) -> &[EvalError] {
        &self.errors
    }

    pub fn press_button(
        &mut self,
        coords: Coords,
        sublocation: u32,
        times: u32,
    ) {
        if let Some(&(group, index)) = self.coords_map.get(&coords) {
            self.chips[group][index].on_press(sublocation, times);
        }
    }

    pub fn press_hotkey(&mut self, code: HotkeyCode) {
        self.state.press_hotkey(code);
    }

    fn num_recorded_inputs(&self) -> u32 {
        let mut total: u32 = 0;
        for &(_, _, _, _, count) in self.state.recorded_inputs.iter() {
            total = total.saturating_add(count);
        }
        total
    }

    pub fn recorded_inputs(&self, top_left: Coords) -> Option<InputsData> {
        if self.state.recorded_inputs.is_empty() {
            return None;
        }
        let mut inputs = InputsData::new();
        for &(time_step, cycle, coords, subloc, count) in
            self.state.recorded_inputs.iter()
        {
            inputs.insert(time_step, cycle, coords - top_left, subloc, count);
        }
        return Some(inputs);
    }

    pub fn wire_analog(&self, wire_id: WireId) -> Fixed {
        self.state.recv_analog(wire_id)
    }

    pub fn wire_event(&self, wire_id: WireId) -> Option<u32> {
        self.state.recv_event(wire_id)
    }

    pub fn wire_value(&self, wire_id: WireId) -> u32 {
        self.state.values[wire_id.0].0
    }

    pub fn wire_has_change(&self, wire_id: WireId) -> bool {
        self.state.values[wire_id.0].1
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
                let errors = self.puzzle_eval.end_cycle(&self.state);
                if self.errors_are_fatal(errors) {
                    return EvalResult::Failure;
                }
                needs_another_cycle |=
                    self.puzzle_eval.needs_another_cycle(&self.state);
                self.subcycle = 0;
                self.state.cycle += 1;
                self.total_cycles += 1;
                if needs_another_cycle {
                    if self.cycle() >= MAX_CYCLES_PER_TIME_STEP {
                        self.errors.push(self.state.fatal_error(format!(
                            "Exceeded {} cycles.",
                            MAX_CYCLES_PER_TIME_STEP
                        )));
                        return EvalResult::Failure;
                    }
                    debug_log!(
                        "  Cycle {} complete, starting another cycle",
                        self.state.cycle - 1
                    );
                    self.state.reset_for_cycle();
                    self.puzzle_eval.begin_additional_cycle(&mut self.state);
                    return EvalResult::Continue;
                }
                let errors = self.puzzle_eval.end_time_step(&self.state);
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
                    self.cycle()
                );
                self.state.reset_for_cycle();
                self.state.cycle = 0;
                self.state.time_step += 1;
                return EvalResult::Continue;
            }
            if self.cycle() == 0 && self.subcycle == 0 {
                if self.puzzle_eval.task_is_completed(&self.state) {
                    if self.errors.is_empty() {
                        let score = match self.score_units {
                            ScoreUnits::Cycles => self.total_cycles,
                            ScoreUnits::ManualInputs => {
                                self.num_recorded_inputs()
                            }
                            ScoreUnits::Time => self.time_step(),
                            ScoreUnits::WireLength => self.wire_length,
                        };
                        return EvalResult::Victory(score);
                    } else {
                        debug_log!("Errors: {:?}", self.errors);
                        return EvalResult::Failure;
                    }
                }
                self.puzzle_eval.begin_time_step(&mut self.state);
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
        let current_cycle = self.cycle();
        while self.time_step() == current_time_step
            && self.cycle() == current_cycle
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
    cycle: u32,
    // "Null" wires are ports that have no wire fragments connected to them.
    // We treat them as wires for ease of evaluation, but we don't count the
    // circuit state as having "changed" for the purposes of debug stepping
    // when one of these ports changes values.
    null_wires: HashSet<WireId>,
    values: Vec<(u32, bool)>,
    breakpoints: Vec<Coords>,
    hotkey_presses: HashMap<HotkeyCode, u32>,
    recorded_inputs: Vec<(u32, u32, Coords, u32, u32)>,
    changed: bool,
}

impl CircuitState {
    fn new(num_values: usize, null_wires: HashSet<WireId>) -> CircuitState {
        CircuitState {
            time_step: 0,
            cycle: 0,
            null_wires,
            values: vec![(0, false); num_values],
            breakpoints: vec![],
            hotkey_presses: HashMap::new(),
            recorded_inputs: Vec::new(),
            changed: false,
        }
    }

    pub fn time_step(&self) -> u32 {
        self.time_step
    }

    pub fn cycle(&self) -> u32 {
        self.cycle
    }

    pub fn is_null_wire(&self, slot: WireId) -> bool {
        self.null_wires.contains(&slot)
    }

    pub fn recv_analog(&self, slot: WireId) -> Fixed {
        Fixed::from_encoded(self.values[slot.0].0)
    }

    pub fn recv_behavior(&self, slot: WireId) -> u32 {
        self.values[slot.0].0
    }

    pub fn behavior_changed(&self, slot: WireId) -> bool {
        self.values[slot.0].1
    }

    pub fn recv_event(&self, slot: WireId) -> Option<u32> {
        let (value, has_event) = self.values[slot.0];
        if has_event {
            Some(value)
        } else {
            None
        }
    }

    pub fn has_event(&self, slot: WireId) -> bool {
        self.values[slot.0].1
    }

    pub fn send_analog(&mut self, slot: WireId, value: Fixed) {
        let encoded = value.to_encoded();
        if self.values[slot.0].0 != encoded {
            self.values[slot.0] = (encoded, true);
            self.changed |= !self.null_wires.contains(&slot);
        }
    }

    pub fn send_behavior(&mut self, slot: WireId, value: u32) {
        if self.values[slot.0].0 != value {
            self.values[slot.0] = (value, true);
            self.changed |= !self.null_wires.contains(&slot);
        }
    }

    pub fn send_event(&mut self, slot: WireId, value: u32) {
        self.values[slot.0] = (value, true);
        self.changed |= !self.null_wires.contains(&slot);
    }

    pub fn breakpoint(&mut self, coords: Coords) {
        self.breakpoints.push(coords);
    }

    fn press_hotkey(&mut self, code: HotkeyCode) {
        let num_presses = self.hotkey_presses.entry(code).or_insert(0);
        *num_presses = num_presses.saturating_add(1);
    }

    pub fn pop_hotkey_presses(&mut self, code: HotkeyCode) -> u32 {
        self.hotkey_presses.remove(&code).unwrap_or(0)
    }

    pub fn record_input(&mut self, coords: Coords, subloc: u32, count: u32) {
        // TODO: more efficient storage
        self.recorded_inputs.push((
            self.time_step,
            self.cycle,
            coords,
            subloc,
            count,
        ));
    }

    #[must_use = "Did you forget to push the EvalError into the errors Vec?"]
    pub fn fatal_error(&self, message: String) -> EvalError {
        EvalError {
            time_step: self.time_step,
            port: None,
            fatal: true,
            message,
        }
    }

    #[must_use = "Did you forget to push the EvalError into the errors Vec?"]
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

    #[must_use = "Did you forget to push the EvalError into the errors Vec?"]
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

    /// Called between time steps; if this returns true, evaluation will halt,
    /// and it will count as a victory if there are no errors.
    fn task_is_completed(&self, state: &CircuitState) -> bool;

    /// Called at the beginning of each time step; sets up input values for the
    /// circuit.
    fn begin_time_step(&mut self, state: &mut CircuitState);

    /// Called at the beginning of each cycle except the first; optionally
    /// sends additional events for that time step or updates analog values.
    /// The default implementation is a no-op.
    fn begin_additional_cycle(&mut self, _state: &mut CircuitState) {}

    /// Called at the end of each cycle; returns a list of errors (if any) that
    /// cause the puzzle to be failed (e.g. if an invalid value was sent to an
    /// interface sink).  The default implementation always returns no errors.
    ///
    /// This is the method that should be used for receiving events at puzzle
    /// interface ports.
    fn end_cycle(&mut self, _state: &CircuitState) -> Vec<EvalError> {
        Vec::new()
    }

    /// Called after end_cycle(); returns true if another cycle is needed.  The
    /// default implementation always returns false.
    fn needs_another_cycle(&self, _state: &CircuitState) -> bool {
        false
    }

    /// Called at the end of each time step; returns a list of errors (if any)
    /// that cause the puzzle to be failed (e.g. if an invalid value was sent
    /// to an interface sink).  The default implementation always returns no
    /// errors.
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
    fn on_press(&mut self, _sublocation: u32, _num_times: u32) {}
}

//===========================================================================//
