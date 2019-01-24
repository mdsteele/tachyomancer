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

use cgmath::{self, Matrix4};
use tachy::font::Align;
use tachy::geom::{Rect, RectSize};
use tachy::gui::{AudioQueue, Event, Keycode, Resources};
use tachy::state::GameState;

//===========================================================================//

pub enum BeginAction {
    CreateProfile(String),
}

//===========================================================================//

pub struct BeginView {
    width: f32,
    height: f32,
    text_entry: TextEntry,
}

impl BeginView {
    pub fn new(window_size: RectSize<u32>, _state: &GameState) -> BeginView {
        BeginView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            text_entry: TextEntry::new(window_size.width / 2, 300),
        }
    }

    pub fn draw(&self, resources: &Resources, _state: &GameState) {
        let projection =
            cgmath::ortho(0.0, self.width, self.height, 0.0, -1.0, 1.0);
        let rect = Rect::new(0.0, 0.0, self.width, self.height);
        resources
            .shaders()
            .solid()
            .fill_rect(&projection, (0.1, 0.1, 0.1), rect);
        resources.fonts().roman().draw(&projection,
                                       40.0,
                                       Align::TopCenter,
                                       (0.5 * self.width, 100.0),
                                       "Enter new profile name:");
        self.text_entry.draw(resources, &projection);
    }

    pub fn on_event(&mut self, event: &Event, state: &mut GameState,
                    audio: &mut AudioQueue)
                    -> Option<BeginAction> {
        if let Some(name) = self.text_entry.on_event(event, audio) {
            if name.is_empty() {
                // TODO: display error to user
                debug_log!("Profile name must be non-empty");
            } else if state.has_profile(&name) {
                // TODO: display error to user
                debug_log!("Profile {:?} already exists", name);
            } else {
                return Some(BeginAction::CreateProfile(name));
            }
        }
        return None;
    }
}

//===========================================================================//

struct TextEntry {
    origin: (f32, f32),
    text: String,
}

impl TextEntry {
    fn new(origin_x: u32, origin_y: u32) -> TextEntry {
        TextEntry {
            origin: (origin_x as f32, origin_y as f32),
            text: String::new(),
        }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        resources
            .fonts()
            .roman()
            .draw(matrix, 60.0, Align::TopCenter, self.origin, &self.text);
    }

    fn on_event(&mut self, event: &Event, _audio: &mut AudioQueue)
                -> Option<String> {
        match event {
            Event::KeyDown(key) => {
                match key.code {
                    Keycode::Return => return Some(self.text.clone()),
                    Keycode::Backspace => {
                        if self.text.pop().is_some() {
                            // TODO: play sound
                        }
                    }
                    _ => {}
                }
            }
            Event::TextInput(text) => {
                for chr in text.chars() {
                    if (chr >= ' ' && chr <= '~') ||
                        (chr >= '\u{a1}' && chr <= '\u{ff}')
                    {
                        self.text.push(chr);
                    }
                }
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
