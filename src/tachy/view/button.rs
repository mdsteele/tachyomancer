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

use cgmath::{Matrix4, Point2};
use tachy::font::Align;
use tachy::geom::Rect;
use tachy::gui::{Event, Keycode, Resources};
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const CHECKBOX_BOX_SIZE: i32 = 28;
const CHECKBOX_BOX_SPACING: i32 = 8;
const CHECKBOX_CHECK_PADDING: i32 = 4;
const CHECKBOX_CHECK_SIZE: i32 = CHECKBOX_BOX_SIZE -
    2 * CHECKBOX_CHECK_PADDING;
const CHECKBOX_FONT_SIZE: f32 = 20.0;
const TEXT_BOX_FONT_SIZE: f32 = 20.0;
const TEXT_BOX_INNER_MARGIN: f32 = 5.0;
const TEXT_BUTTON_FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct Checkbox {
    rect: Rect<i32>,
    label: String,
    hovering: bool,
}

impl Checkbox {
    pub fn new(mid_left: Point2<i32>, label: &str) -> Checkbox {
        let top = mid_left.y - CHECKBOX_BOX_SIZE / 2;
        let width = CHECKBOX_BOX_SIZE + CHECKBOX_BOX_SPACING +
            (0.5 * CHECKBOX_FONT_SIZE * (label.width() as f32)).ceil() as i32;
        Checkbox {
            rect: Rect::new(mid_left.x, top, width, CHECKBOX_BOX_SIZE),
            label: label.to_string(),
            hovering: false,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                checked: bool, enabled: bool) {
        let box_rect = Rect::new(self.rect.x,
                                 self.rect.y,
                                 CHECKBOX_BOX_SIZE,
                                 CHECKBOX_BOX_SIZE);
        let (box_color, check_color) = if !enabled {
            ((0.4, 0.4, 0.4), (0.8, 0.8, 0.8))
        } else if self.hovering {
            ((1.0, 0.2, 0.2), (1.0, 0.8, 0.8))
        } else {
            ((0.7, 0.1, 0.1), (1.0, 0.5, 0.5))
        };
        resources
            .shaders()
            .solid()
            .fill_rect(&matrix, box_color, box_rect.as_f32());
        if checked {
            let check_rect = Rect::new(box_rect.x + CHECKBOX_CHECK_PADDING,
                                       box_rect.y + CHECKBOX_CHECK_PADDING,
                                       CHECKBOX_CHECK_SIZE,
                                       CHECKBOX_CHECK_SIZE);
            resources
                .shaders()
                .solid()
                .fill_rect(&matrix, check_color, check_rect.as_f32());
        }
        resources.fonts().roman().draw(&matrix,
                                       CHECKBOX_FONT_SIZE,
                                       Align::MidLeft,
                                       ((box_rect.x + CHECKBOX_BOX_SIZE +
                                             CHECKBOX_BOX_SPACING) as
                                            f32,
                                        (box_rect.y + CHECKBOX_BOX_SIZE / 2) as
                                            f32),
                                       &self.label);
    }

    pub fn handle_event(&mut self, event: &Event, checked: bool,
                        enabled: bool)
                        -> Option<bool> {
        match event {
            Event::MouseDown(mouse) => {
                if enabled && mouse.left &&
                    self.rect.contains_point(mouse.pt)
                {
                    return Some(!checked);
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
            }
            Event::Unfocus => self.hovering = false,
            _ => {}
        }
        return None;
    }
}

//===========================================================================//

pub struct RadioButton<T> {
    inner: TextButton<T>,
}

impl<T: Clone + PartialEq> RadioButton<T> {
    pub fn new(rect: Rect<i32>, label: &str, value: T) -> RadioButton<T> {
        RadioButton { inner: TextButton::new(rect, label, value) }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                value: &T) {
        self.inner.draw(resources, matrix, value != &self.inner.value);
    }

    pub fn handle_event(&mut self, event: &Event, value: &T) -> Option<T> {
        let enabled = value != &self.inner.value;
        self.inner.handle_event(event, enabled)
    }
}

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
                                       TEXT_BOX_FONT_SIZE,
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
    hovering: bool,
}

impl<T: Clone> TextButton<T> {
    pub fn new(rect: Rect<i32>, label: &str, value: T) -> TextButton<T> {
        TextButton {
            rect,
            label: label.to_string(),
            value,
            hovering: false,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                enabled: bool) {
        let color = if !enabled {
            (0.4, 0.4, 0.4)
        } else if self.hovering {
            (1.0, 0.2, 0.2)
        } else {
            (0.7, 0.1, 0.1)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       TEXT_BUTTON_FONT_SIZE,
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
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
            }
            Event::Unfocus => self.hovering = false,
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
