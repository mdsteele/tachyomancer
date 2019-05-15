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
use tachy::gui::{Event, Window};
use tachy::state::GameState;
use tachy::view::{CutsceneAction, CutsceneView};

//===========================================================================//

pub fn run(state: &mut GameState, window: &mut Window) -> ModeChange {
    window.set_cursor_visible(false);
    let mode_change = run_internal(state, window);
    window.set_cursor_visible(true);
    mode_change
}

fn run_internal(state: &mut GameState, window: &mut Window) -> ModeChange {
    debug_assert!(state.cutscene().is_some());
    let mut view = CutsceneView::new(window.size());
    view.init(&mut window.ui(), state.cutscene_mut_and_prefs().unwrap());
    let mut last_tick = Instant::now();
    loop {
        let mut finished = false;
        match window.poll_event() {
            Some(Event::Quit) => return ModeChange::Quit,
            Some(event) => {
                match view.on_event(&event,
                                    &mut window.ui(),
                                    state.cutscene_mut_and_prefs().unwrap()) {
                    Some(CutsceneAction::Finished) => finished = true,
                    None => {}
                }
                window.pump_cursor();
            }
            None => {
                let now = Instant::now();
                let elapsed = now.duration_since(last_tick);
                let action =
                    view.on_event(&Event::new_clock_tick(elapsed),
                                  &mut window.ui(),
                                  state.cutscene_mut_and_prefs().unwrap());
                match action {
                    Some(CutsceneAction::Finished) => finished = true,
                    None => {}
                }
                window.pump_cursor();
                last_tick = now;
                window.pump_audio();
                view.draw(window.resources(), state.cutscene().unwrap());
                window.pump_video();
            }
        }
        if finished {
            state.clear_cutscene();
            return ModeChange::Next;
        }
    }
}

//===========================================================================//
