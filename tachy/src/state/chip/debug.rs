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
use crate::geom::{Coords, Direction};
use crate::save::{HotkeyCode, WireSize};
use crate::state::{PortColor, PortFlow};

//===========================================================================//

pub const BREAK_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Recv, PortColor::Event, (0, 0), Direction::West),
        (PortFlow::Send, PortColor::Event, (0, 0), Direction::East),
    ],
    constraints: &[AbstractConstraint::Equal(0, 1)],
    dependencies: &[(0, 1)],
};

pub struct BreakChipEval {
    enabled: bool,
    input: usize,
    output: usize,
    coords: Coords,
}

impl BreakChipEval {
    pub fn new_evals(
        enabled: bool,
        slots: &[(usize, WireSize)],
        coords: Coords,
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), BREAK_CHIP_DATA.ports.len());
        let chip_eval = BreakChipEval {
            enabled,
            input: slots[0].0,
            output: slots[1].0,
            coords,
        };
        vec![(1, Box::new(chip_eval))]
    }
}

impl ChipEval for BreakChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(value) = state.recv_event(self.input) {
            state.send_event(self.output, value);
            if self.enabled {
                state.breakpoint(self.coords);
            }
        }
    }

    fn coords(&self) -> Option<Coords> {
        Some(self.coords)
    }

    fn display_data(&self) -> &[u8] {
        if self.enabled {
            &[1]
        } else {
            &[0]
        }
    }

    fn on_press(&mut self, _sublocation: u32, num_times: u32) {
        if num_times % 2 != 0 {
            self.enabled = !self.enabled;
        }
    }
}

//===========================================================================//

pub const BUTTON_CHIP_DATA: &ChipData = &ChipData {
    ports: &[(PortFlow::Send, PortColor::Event, (0, 0), Direction::East)],
    constraints: &[AbstractConstraint::Exact(0, WireSize::Zero)],
    dependencies: &[],
};

pub struct ButtonChipEval {
    output: usize,
    coords: Coords,
    hotkey: Option<HotkeyCode>,
    press_count: u32,
}

impl ButtonChipEval {
    pub fn new_evals(
        hotkey: Option<HotkeyCode>,
        slots: &[(usize, WireSize)],
        coords: Coords,
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), BUTTON_CHIP_DATA.ports.len());
        let chip_eval = ButtonChipEval {
            output: slots[0].0,
            coords,
            hotkey,
            press_count: 0,
        };
        vec![(0, Box::new(chip_eval))]
    }
}

impl ChipEval for ButtonChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if let Some(code) = self.hotkey {
            self.press_count = self
                .press_count
                .saturating_add(state.pop_hotkey_presses(code));
        }
        if self.press_count > 0 {
            self.press_count -= 1;
            state.send_event(self.output, 0);
            state.record_input(self.coords, 0, 1);
        }
    }

    fn needs_another_cycle(&mut self, _state: &CircuitState) -> bool {
        self.press_count > 0
    }

    fn coords(&self) -> Option<Coords> {
        Some(self.coords)
    }

    fn on_press(&mut self, _sublocation: u32, num_times: u32) {
        self.press_count = self.press_count.saturating_add(num_times);
    }
}

//===========================================================================//

pub const COMMENT_CHIP_DATA: &ChipData =
    &ChipData { ports: &[], constraints: &[], dependencies: &[] };

//===========================================================================//

pub const DISPLAY_CHIP_DATA: &ChipData = &ChipData {
    ports: &[(PortFlow::Recv, PortColor::Behavior, (0, 0), Direction::South)],
    constraints: &[],
    dependencies: &[],
};

//===========================================================================//

pub const TOGGLE_CHIP_DATA: &ChipData = &ChipData {
    ports: &[(PortFlow::Send, PortColor::Behavior, (0, 0), Direction::East)],
    constraints: &[AbstractConstraint::Exact(0, WireSize::One)],
    dependencies: &[],
};

pub struct ToggleChipEval {
    output: usize,
    value: bool,
    coords: Coords,
    toggle_count: u32,
}

impl ToggleChipEval {
    pub fn new_evals(
        value: bool,
        slots: &[(usize, WireSize)],
        coords: Coords,
    ) -> Vec<(usize, Box<dyn ChipEval>)> {
        debug_assert_eq!(slots.len(), TOGGLE_CHIP_DATA.ports.len());
        let chip_eval = ToggleChipEval {
            output: slots[0].0,
            value,
            coords,
            toggle_count: 0,
        };
        vec![(0, Box::new(chip_eval))]
    }
}

impl ChipEval for ToggleChipEval {
    fn eval(&mut self, state: &mut CircuitState) {
        if self.toggle_count > 0 {
            state.record_input(self.coords, 0, self.toggle_count);
            self.toggle_count = 0;
        }
        state.send_behavior(self.output, self.value.into());
    }

    fn coords(&self) -> Option<Coords> {
        Some(self.coords)
    }

    fn on_press(&mut self, _sublocation: u32, num_times: u32) {
        if num_times % 2 != 0 {
            self.value = !self.value;
        }
        self.toggle_count = self.toggle_count.saturating_add(num_times);
    }
}

//===========================================================================//
