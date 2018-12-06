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

use super::list::ListView;
use cgmath::Matrix4;
use tachy::gui::{Event, Resources};
use tachy::state::{GameState, Rect};

//===========================================================================//

pub enum PrefsAction {
    SwitchProfile(String),
}

//===========================================================================//

pub struct PrefsView {
    profiles_list: ListView<String>,
}

impl PrefsView {
    pub fn new(rect: Rect<i32>, state: &GameState) -> PrefsView {
        debug_assert!(state.profile().is_some());
        let current_profile_name = state.profile().unwrap().name().to_string();
        let list_items = state
            .savedir()
            .profile_names()
            .iter()
            .map(|name| (name.clone(), name.clone()))
            .collect();
        PrefsView {
            profiles_list: ListView::new(Rect::new(rect.x,
                                                   rect.y,
                                                   300,
                                                   rect.height),
                                         &current_profile_name,
                                         list_items),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        debug_assert!(state.profile().is_some());
        let current_profile_name = state.profile().unwrap().name().to_string();
        self.profiles_list.draw(resources, matrix, &current_profile_name);
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState)
                        -> Option<PrefsAction> {
        debug_assert!(state.profile().is_some());
        let current_profile_name = state.profile().unwrap().name().to_string();
        if let Some(profile_name) =
            self.profiles_list.handle_event(event, &current_profile_name)
        {
            return Some(PrefsAction::SwitchProfile(profile_name));
        }
        return None;
    }

    pub fn unfocus(&mut self) { self.profiles_list.unfocus(); }
}

//===========================================================================//
