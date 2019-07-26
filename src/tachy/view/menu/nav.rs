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

use cgmath::Matrix4;
use tachy::geom::{Rect, RectSize};
use tachy::gui::{Event, Resources, Ui};
use tachy::state::GameState;

//===========================================================================//

pub struct NavigationView {}

impl NavigationView {
    pub fn new(_screen_size: RectSize<f32>, _rect: Rect<i32>, _ui: &mut Ui,
               _state: &GameState)
               -> NavigationView {
        NavigationView {}
    }

    pub fn draw(&self, _resources: &Resources, _matrix: &Matrix4<f32>,
                _state: &GameState) {
    }

    pub fn on_event(&mut self, _event: &Event, _ui: &mut Ui,
                    _state: &mut GameState) {
    }
}

//===========================================================================//
