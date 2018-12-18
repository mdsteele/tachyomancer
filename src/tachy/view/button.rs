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
use tachy::font::Align;
use tachy::geom::Rect;
use tachy::gui::{Event, Keycode, Resources};
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const TEXT_BOX_INNER_MARGIN: f32 = 5.0;

//===========================================================================//

pub struct TextBox {
    rect: Rect<i32>,
    string: String,
    max_len: usize,
}

impl TextBox {
    pub fn new(rect: Rect<i32>, initial: &str, max_len: usize) -> TextBox {
        TextBox {
            rect,
            string: initial.to_string(),
            max_len,
        }
    }

    pub fn string(&self) -> &str { &self.string }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = (0.0, 0.0, 0.0);
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       20.0,
                                       Align::MidLeft,
                                       (rect.x + TEXT_BOX_INNER_MARGIN,
                                        rect.y + 0.5 * rect.height),
                                       &self.string);
        // TODO: draw cursor
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::KeyDown(key) => {
                match key.code {
                    Keycode::Backspace => {
                        if self.string.pop().is_some() {
                            // TODO: play sound
                        }
                    }
                    // TODO: arrows should move cursor
                    _ => {}
                }
            }
            Event::TextInput(text) => {
                for chr in text.chars() {
                    if self.string.width() >= self.max_len {
                        break;
                    }
                    if (chr >= ' ' && chr <= '~') ||
                        (chr >= '\u{a1}' && chr <= '\u{ff}')
                    {
                        self.string.push(chr);
                        // TODO: play sound
                    }
                }
            }
            _ => {}
        }
    }
}

//===========================================================================//

pub struct TextButton<T> {
    rect: Rect<i32>,
    label: String,
    value: T,
}

impl<T: Clone> TextButton<T> {
    pub fn new(rect: Rect<i32>, label: &str, value: T) -> TextButton<T> {
        TextButton {
            rect,
            label: label.to_string(),
            value,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                enabled: bool) {
        let color = if enabled {
            (0.7, 0.1, 0.1)
        } else {
            (0.4, 0.4, 0.4)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       20.0,
                                       Align::MidCenter,
                                       (rect.x + 0.5 * rect.width,
                                        rect.y + 0.5 * rect.height),
                                       &self.label);
    }

    pub fn handle_event(&mut self, event: &Event, enabled: bool) -> Option<T> {
        match event {
            Event::MouseDown(mouse) => {
                if enabled && mouse.left &&
                    self.rect.contains_point(mouse.pt)
                {
                    return Some(self.value.clone());
                }
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
