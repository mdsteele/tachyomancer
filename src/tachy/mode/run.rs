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
use tachy::gui::Window;
use tachy::state::GameState;

//===========================================================================//

pub fn run_mode(state: &mut GameState, window: &mut Window) -> ModeChange {
    if state.cutscene().is_some() {
        super::cutscene::run(state, window)
    } else if state.profile().is_none() {
        super::begin::run(state, window)
    } else if state.edit_grid().is_some() {
        super::circuit::run(state, window)
    } else {
        super::menu::run(state, window)
    }
}

//===========================================================================//
