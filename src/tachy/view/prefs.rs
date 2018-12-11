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
use tachy::font::Align;
use tachy::gui::{Event, Resources};
use tachy::state::{GameState, Rect};

//===========================================================================//

pub enum PrefsAction {
    NewProfile,
    SwitchProfile(String),
}

//===========================================================================//

pub struct PrefsView {
    profiles_list: ListView<String>,
    new_button: NewButton,
}

impl PrefsView {
    pub fn new(rect: Rect<i32>, state: &GameState) -> PrefsView {
        debug_assert!(state.profile().is_some());
        let current_profile_name = state.profile().unwrap().name();
        let list_items = state
            .savedir()
            .profile_names()
            .map(|name| (name.to_string(), name.to_string()))
            .collect();
        PrefsView {
            profiles_list: ListView::new(Rect::new(rect.x,
                                                   rect.y,
                                                   300,
                                                   rect.height),
                                         current_profile_name,
                                         list_items),
            new_button: NewButton::new(Rect::new(rect.right() - 150,
                                                 rect.bottom() - 40,
                                                 150,
                                                 40)),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        debug_assert!(state.profile().is_some());
        let current_profile_name = state.profile().unwrap().name();
        self.profiles_list.draw(resources, matrix, current_profile_name);
        self.new_button.draw(resources, matrix);
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState)
                        -> Option<PrefsAction> {
        debug_assert!(state.profile().is_some());
        let current_profile_name = state.profile().unwrap().name();
        if let Some(profile_name) =
            self.profiles_list.handle_event(event, current_profile_name)
        {
            return Some(PrefsAction::SwitchProfile(profile_name));
        }
        if let Some(action) = self.new_button.handle_event(event) {
            return Some(action);
        }
        return None;
    }

    pub fn unfocus(&mut self) { self.profiles_list.unfocus(); }
}

//===========================================================================//

struct NewButton {
    rect: Rect<i32>,
}

impl NewButton {
    pub fn new(rect: Rect<i32>) -> NewButton { NewButton { rect } }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = (0.7, 0.1, 0.1);
        let rect = (self.rect.x as f32,
                    self.rect.y as f32,
                    self.rect.width as f32,
                    self.rect.height as f32);
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       20.0,
                                       Align::Center,
                                       ((self.rect.x as f32) +
                                            0.5 * (self.rect.width as f32),
                                        (self.rect.y as f32) +
                                            0.5 * (self.rect.height as f32) -
                                            10.0),
                                       "New Profile");
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<PrefsAction> {
        match event {
            Event::MouseDown(mouse) => {
                if mouse.left && self.rect.contains_point(mouse.pt) {
                    return Some(PrefsAction::NewProfile);
                }
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
