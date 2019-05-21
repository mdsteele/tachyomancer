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
use tachy::save::MenuSection;
use tachy::state::{Cutscene, GameState};
use tachy::view::{BeginAction, BeginView};

//===========================================================================//

pub fn run(state: &mut GameState, window: &mut Window) -> ModeChange {
    let mut view = BeginView::new(window.size(), state);
    let mut last_tick = Instant::now();
    loop {
        match window.poll_event() {
            Some(Event::Quit) => return ModeChange::Quit,
            Some(event) => {
                match view.on_event(&event, &mut window.ui(), state) {
                    Some(BeginAction::CreateProfile(name)) => {
                        debug_log!("Creating profile {:?}", name);
                        match state.create_or_load_profile(name) {
                            Ok(()) => {
                                state.set_menu_section(MenuSection::Messages);
                                state.set_cutscene(Cutscene::Intro.script());
                                return ModeChange::Next;
                            }
                            Err(err) => {
                                // TODO: display error to user; don't panic
                                panic!("CreateProfile failed: {:?}", err);
                            }
                        }
                    }
                    None => {}
                }
                window.pump_cursor();
            }
            None => {
                let now = Instant::now();
                let elapsed = now.duration_since(last_tick);
                view.on_event(&Event::new_clock_tick(elapsed),
                              &mut window.ui(),
                              state);
                window.pump_cursor();
                last_tick = now;
                window.pump_audio();
                view.draw(window.resources(), state);
                window.pump_video();
            }
        }
    }
}

//===========================================================================//
