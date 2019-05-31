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

use super::common::ModeChange;
use tachy::gui::{Event, Window};
use tachy::state::GameState;
use tachy::view::{CircuitAction, CircuitView};

//===========================================================================//

pub fn run(state: &mut GameState, window: &mut Window) -> ModeChange {
    debug_assert!(state.profile().is_some());
    debug_assert!(state.edit_grid().is_some());
    let mut view = {
        let grid = state.edit_grid().unwrap();
        let puzzle = grid.puzzle();
        let allowed = grid.allowed_chips();
        CircuitView::new(window.size(), puzzle, allowed, state.prefs())
    };
    loop {
        match window.next_event() {
            Event::Quit => return ModeChange::Quit,
            Event::Redraw => {
                window.pump_audio();
                view.draw(window.resources(), state.edit_grid().unwrap());
                window.pump_video();
                state.maybe_autosave_circuit();
            }
            event => {
                window.ui().request_redraw(); // TODO: only do this when needed
                match view.on_event(&event,
                                    &mut window.ui(),
                                    state.edit_grid_mut_and_prefs().unwrap()) {
                    Some(CircuitAction::BackToMenu) => {
                        match state.save() {
                            Ok(()) => {
                                state.clear_edit_grid();
                                return ModeChange::Next;
                            }
                            Err(err) => {
                                // TODO: display error to user; don't panic
                                panic!("BackToMenu failed: {:?}", err);
                            }
                        }
                    }
                    Some(CircuitAction::Victory(area, score)) => {
                        record_score(state, area, score);
                    }
                    None => {}
                }
                window.pump_cursor();
            }
        }
    }
}

fn record_score(state: &mut GameState, area: i32, score: i32) {
    match state.record_current_puzzle_score(area, score) {
        Ok(()) => {}
        Err(err) => {
            // TODO: display error to user; don't panic
            panic!("Victory failed: {:?}", err);
        }
    }
}

//===========================================================================//
