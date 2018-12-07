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
use tachy::gui::{AudioQueue, Event, Window};
use tachy::state::GameState;
use tachy::view::{BeginAction, BeginView};

//===========================================================================//

pub fn run(state: &mut GameState, window: &mut Window) -> ModeChange {
    let mut view = BeginView::new(window.size().into(), state);
    let mut last_tick = Instant::now();
    let mut audio = AudioQueue::new();
    loop {
        match window.poll_event() {
            Some(Event::Quit) => return ModeChange::Quit,
            Some(event) => {
                match view.handle_event(&event, state, &mut audio) {
                    Some(BeginAction::CreateProfile(name)) => {
                        debug_log!("Creating profile {:?}", name);
                        match state.create_or_load_profile(name) {
                            Ok(()) => return ModeChange::Next,
                            Err(err) => {
                                // TODO: display error to user; don't panic
                                panic!("CreateProfile failed: {:?}", err);
                            }
                        }
                    }
                    None => {}
                }
            }
            None => {
                let now = Instant::now();
                let elapsed = now.duration_since(last_tick);
                view.handle_event(&Event::new_clock_tick(elapsed),
                                  state,
                                  &mut audio);
                last_tick = now;
                window.pump_audio(&mut audio);
                view.draw(window.resources(), state);
                window.swap();
            }
        }
    }
}

//===========================================================================//
