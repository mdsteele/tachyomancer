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
use std::time::Instant;
use tachy::gui::{AudioQueue, Event, NextCursor, Window};
use tachy::state::GameState;
use tachy::view::{CircuitAction, CircuitView};

//===========================================================================//

pub fn run(state: &mut GameState, window: &mut Window) -> ModeChange {
    debug_assert!(state.profile().is_some());
    debug_assert!(state.edit_grid().is_some());
    let puzzle = state.current_puzzle();
    let mut view = CircuitView::new(window.size(), puzzle);
    let mut last_tick = Instant::now();
    let mut audio = AudioQueue::new();
    loop {
        let mut cursor = NextCursor::new();
        match window.poll_event() {
            Some(Event::Quit) => return ModeChange::Quit,
            Some(event) => {
                match view.on_event(&event,
                                      state.edit_grid_mut_and_prefs().unwrap(),
                                      &mut audio,
                                      window.clipboard(),
                                      &mut cursor) {
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
                window.cursors().set(cursor);
            }
            None => {
                let now = Instant::now();
                let elapsed = now.duration_since(last_tick);
                match view.on_event(&Event::new_clock_tick(elapsed),
                                      state.edit_grid_mut_and_prefs().unwrap(),
                                      &mut audio,
                                      window.clipboard(),
                                      &mut cursor) {
                    Some(CircuitAction::Victory(area, score)) => {
                        record_score(state, area, score);
                    }
                    Some(_) => unreachable!(),
                    None => {}
                }
                window.cursors().set(cursor);
                last_tick = now;
                window.pump_audio(&mut audio);
                view.draw(window.resources(), state.edit_grid().unwrap());
                window.swap();
                state.maybe_autosave_circuit();
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
