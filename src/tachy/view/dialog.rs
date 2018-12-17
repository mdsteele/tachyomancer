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

use super::button::TextButton;
use cgmath::Matrix4;
use tachy::font::Align;
use tachy::geom::{Rect, RectSize};
use tachy::gui::{Event, Resources};

//===========================================================================//

const BUTTON_HEIGHT: i32 = 40;
const BUTTON_INNER_MARGIN: i32 = 10;
const BUTTON_MIN_WIDTH: i32 = 80;
const BUTTON_SPACING: i32 = 14;
const BUTTON_TOP_MARGIN: i32 = 16;
const FONT_SIZE: f32 = 20.0;
const LINE_HEIGHT: i32 = 24;
const MARGIN: i32 = 24;

//===========================================================================//

pub struct ButtonDialogBox<T> {
    rect: Rect<i32>,
    strings: Vec<String>,
    buttons: Vec<TextButton<T>>,
}

impl<T: Clone> ButtonDialogBox<T> {
    pub fn new(window_size: RectSize<i32>, text: &str, buttons: &[(&str, T)])
               -> ButtonDialogBox<T> {
        let strings: Vec<String> =
            text.split('\n').map(str::to_string).collect();

        let mut width = 0;
        let buttons: Vec<(&str, T, i32, i32)> = buttons
            .iter()
            .map(|&(label, ref value)| {
                let button_width =
                    BUTTON_MIN_WIDTH
                        .max(string_width(label) + 2 * BUTTON_INNER_MARGIN);
                if width > 0 {
                    width += BUTTON_SPACING;
                }
                width += button_width;
                (label, value.clone(), width, button_width)
            })
            .collect();
        for string in strings.iter() {
            width =
                width.max((0.5 * FONT_SIZE * (string.len() as f32))
                              .ceil() as i32);
        }
        width += 2 * MARGIN;

        let button_top = MARGIN + (strings.len() as i32) * LINE_HEIGHT +
            BUTTON_TOP_MARGIN;
        let height = button_top + BUTTON_HEIGHT + MARGIN;

        let rect = Rect::new((window_size.width - width) / 2,
                             (window_size.height - height) / 2,
                             width,
                             height);

        let buttons = buttons
            .into_iter()
            .map(|(label, value, button_offset, button_width)| {
                     let button_rect = Rect::new(rect.right() - MARGIN -
                                                     button_offset,
                                                 rect.y + button_top,
                                                 button_width,
                                                 BUTTON_HEIGHT);
                     TextButton::new(button_rect, label, value)
                 })
            .collect();

        ButtonDialogBox {
            rect,
            strings,
            buttons,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = (0.9, 0.9, 0.9);
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);

        let left = (self.rect.x + MARGIN) as f32;
        let mut top = (self.rect.y + MARGIN) as f32;
        for string in self.strings.iter() {
            resources
                .fonts()
                .roman()
                .draw(matrix, FONT_SIZE, Align::TopLeft, (left, top), string);
            top += LINE_HEIGHT as f32;
        }

        for button in self.buttons.iter() {
            button.draw(resources, matrix);
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<T> {
        for button in self.buttons.iter_mut() {
            if let Some(value) = button.handle_event(event) {
                return Some(value);
            }
        }
        return None;
    }
}

//===========================================================================//

fn string_width(string: &str) -> i32 {
    // TODO: string.len() isn't really correct here; use unicode_width
    (0.5 * FONT_SIZE * (string.len() as f32)).ceil() as i32
}

//===========================================================================//
