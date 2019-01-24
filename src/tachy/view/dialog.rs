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

use super::button::{TextBox, TextButton};
use cgmath::Matrix4;
use tachy::font::Align;
use tachy::geom::{Rect, RectSize};
use tachy::gui::{Event, Resources};
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const BUTTON_HEIGHT: i32 = 40;
const BUTTON_INNER_MARGIN: i32 = 10;
const BUTTON_MIN_WIDTH: i32 = 80;
const BUTTON_SPACING: i32 = 14;
const BUTTON_TOP_MARGIN: i32 = 16;
const FONT_SIZE: f32 = 20.0;
const LINE_HEIGHT: i32 = 24;
const MARGIN: i32 = 24;
const TEXTBOX_HEIGHT: i32 = 32;
const TEXTBOX_TOP_MARGIN: i32 = 16;

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
            width = width.max(string_width(string));
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
            button.draw(resources, matrix, true);
        }
    }

    pub fn on_event(&mut self, event: &Event) -> Option<T> {
        for button in self.buttons.iter_mut() {
            if let Some(value) = button.on_event(event, true) {
                return Some(value);
            }
        }
        return None;
    }
}

//===========================================================================//

pub struct TextDialogBox {
    rect: Rect<i32>,
    strings: Vec<String>,
    textbox: TextBox,
    ok_button: TextButton<()>,
    cancel_button: TextButton<()>,
}

impl TextDialogBox {
    pub fn new(window_size: RectSize<i32>, text: &str, initial: &str,
               max_len: usize)
               -> TextDialogBox {
        let strings: Vec<String> =
            text.split('\n').map(str::to_string).collect();

        let mut width = 16 +
            (0.5 * FONT_SIZE * (max_len as f32)).ceil() as i32;
        for string in strings.iter() {
            width = width.max(string_width(string));
        }
        let textbox_width = width;
        width += 2 * MARGIN;

        let textbox_top = MARGIN + (strings.len() as i32) * LINE_HEIGHT +
            TEXTBOX_TOP_MARGIN;
        let button_top = textbox_top + TEXTBOX_HEIGHT + BUTTON_TOP_MARGIN;
        let height = button_top + BUTTON_HEIGHT + MARGIN;

        let rect = Rect::new((window_size.width - width) / 2,
                             (window_size.height - height) / 2,
                             width,
                             height);

        let textbox_rect = Rect::new(rect.x + MARGIN,
                                     rect.y + textbox_top,
                                     textbox_width,
                                     TEXTBOX_HEIGHT);
        let textbox = TextBox::new(textbox_rect, initial, max_len);

        let ok_button_rect = Rect::new(rect.right() - MARGIN -
                                           BUTTON_MIN_WIDTH,
                                       rect.y + button_top,
                                       BUTTON_MIN_WIDTH,
                                       BUTTON_HEIGHT);
        let ok_button = TextButton::new(ok_button_rect, "OK", ());

        let cancel_button_rect = Rect::new(ok_button_rect.x - BUTTON_SPACING -
                                               BUTTON_MIN_WIDTH,
                                           rect.y + button_top,
                                           BUTTON_MIN_WIDTH,
                                           BUTTON_HEIGHT);
        let cancel_button = TextButton::new(cancel_button_rect, "Cancel", ());

        TextDialogBox {
            rect,
            strings,
            textbox,
            ok_button,
            cancel_button,
        }
    }

    pub fn draw<F>(&self, resources: &Resources, matrix: &Matrix4<f32>,
                   is_valid: F)
    where
        F: Fn(&str) -> bool,
    {
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

        self.textbox.draw(resources, matrix);
        let valid = is_valid(&self.textbox.string());
        self.ok_button.draw(resources, matrix, valid);
        self.cancel_button.draw(resources, matrix, true);
    }

    pub fn on_event<F>(&mut self, event: &Event, is_valid: F)
                       -> Option<Option<String>>
    where
        F: Fn(&str) -> bool,
    {
        self.textbox.on_event(event);
        let string = self.textbox.string();
        let valid = is_valid(string);
        if let Some(()) = self.ok_button.on_event(event, valid) {
            return Some(Some(string.to_string()));
        }
        if let Some(()) = self.cancel_button.on_event(event, true) {
            return Some(None);
        }
        return None;
    }
}

//===========================================================================//

fn string_width(string: &str) -> i32 {
    (0.5 * FONT_SIZE * (string.width() as f32)).ceil() as i32
}

//===========================================================================//
