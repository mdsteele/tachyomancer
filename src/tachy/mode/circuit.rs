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
use tachy::view::CircuitView;

//===========================================================================//

pub fn run(state: &mut GameState, window: &mut Window) -> ModeChange {
    let mut view = CircuitView::new(window.size().into());
    let grid = state.edit_grid_mut().unwrap(); // TODO
    let mut last_tick = Instant::now();
    let mut audio = AudioQueue::new();
    loop {
        match window.poll_event() {
            Some(Event::Quit) => return ModeChange::Quit,
            Some(event) => {
                let toggle = view.handle_event(&event, grid, &mut audio);
                if toggle {
                    let mut window_options = window.options();
                    window_options.fullscreen = !window_options.fullscreen;
                    return ModeChange::RebootWindow(window_options);
                }
            }
            None => {
                let now = Instant::now();
                let elapsed = now.duration_since(last_tick);
                view.handle_event(&Event::new_clock_tick(elapsed),
                                  grid,
                                  &mut audio);
                last_tick = now;
                window.pump_audio(&mut audio);
                view.draw(window.resources(), grid);
                window.swap();
            }
        }
    }
}

//===========================================================================//
