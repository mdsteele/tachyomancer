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

use super::button::{TEXT_BUTTON_FONT, TEXT_BUTTON_FONT_SIZE, TextBox,
                    TextButton};
use super::paragraph::Paragraph;
use cgmath::Matrix4;
use tachy::geom::{AsFloat, Color4, Rect, RectSize};
use tachy::gui::{Cursor, Event, Keycode, Resources, Ui};
use tachy::save::Prefs;
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const DIALOG_COLOR_1: Color4 = Color4::CYAN4;
const DIALOG_COLOR_2: Color4 = Color4::ORANGE5;
const DIALOG_COLOR_3: Color4 = Color4::PURPLE0.with_alpha(0.8);

const BUTTON_HEIGHT: i32 = 40;
const BUTTON_INNER_MARGIN: i32 = 10;
const BUTTON_MIN_WIDTH: i32 = 80;
const BUTTON_SPACING: i32 = 14;
const BUTTON_TOP_MARGIN: i32 = 16;

const FONT_SIZE: f32 = 20.0;
const LINE_HEIGHT: f32 = 24.0;
const MARGIN: i32 = 24;
const MAX_PARAGRAPH_WIDTH: f32 = 600.0;
const TEXTBOX_HEIGHT: i32 = 32;
const TEXTBOX_TOP_MARGIN: i32 = 16;

//===========================================================================//

pub struct ButtonDialogBox<T> {
    rect: Rect<i32>,
    paragraph: Paragraph,
    buttons: Vec<TextButton<T>>,
}

impl<T: Clone> ButtonDialogBox<T> {
    pub fn new(window_size: RectSize<i32>, prefs: &Prefs, format: &str,
               buttons: &[(&str, T, Option<Keycode>)])
               -> ButtonDialogBox<T> {
        let paragraph = Paragraph::compile(FONT_SIZE,
                                           LINE_HEIGHT,
                                           MAX_PARAGRAPH_WIDTH,
                                           prefs,
                                           format);

        let mut width = 0;
        let buttons: Vec<(&str, T, Option<Keycode>, i32, i32)> =
            buttons
                .iter()
                .map(|&(label, ref value, key)| {
                    let label_width = TEXT_BUTTON_FONT.ratio() *
                        TEXT_BUTTON_FONT_SIZE *
                        (label.width() as f32);
                    let button_width = BUTTON_MIN_WIDTH
                        .max((label_width.ceil() as i32) +
                                 2 * BUTTON_INNER_MARGIN);
                    if width > 0 {
                        width += BUTTON_SPACING;
                    }
                    width += button_width;
                    (label, value.clone(), key, width, button_width)
                })
                .collect();
        width = width.max(paragraph.width().ceil() as i32);
        width += 2 * MARGIN;

        let button_top = MARGIN + (paragraph.height().ceil() as i32) +
            BUTTON_TOP_MARGIN;
        let height = button_top + BUTTON_HEIGHT + MARGIN;

        let rect = Rect::new((window_size.width - width) / 2,
                             (window_size.height - height) / 2,
                             width,
                             height);

        let buttons = buttons
            .into_iter()
            .map(|(label, value, key, button_offset, button_width)| {
                     let button_rect = Rect::new(rect.right() - MARGIN -
                                                     button_offset,
                                                 rect.y + button_top,
                                                 button_width,
                                                 BUTTON_HEIGHT);
                     TextButton::new_with_key(button_rect, label, value, key)
                 })
            .collect();

        ButtonDialogBox {
            rect,
            paragraph,
            buttons,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let rect = self.rect.as_f32();
        resources.shaders().ui().draw_dialog(&matrix,
                                             &rect,
                                             &DIALOG_COLOR_1,
                                             &DIALOG_COLOR_2,
                                             &DIALOG_COLOR_3);

        let left = (self.rect.x + MARGIN) as f32;
        let top = (self.rect.y + MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));

        for button in self.buttons.iter() {
            button.draw(resources, matrix, true);
        }
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui) -> Option<T> {
        for button in self.buttons.iter_mut() {
            if let Some(value) = button.on_event(event, ui, true) {
                return Some(value);
            }
        }
        return None;
    }
}

//===========================================================================//

pub struct TextDialogBox {
    rect: Rect<i32>,
    paragraph: Paragraph,
    textbox: TextBox,
    ok_button: TextButton<()>,
    cancel_button: TextButton<()>,
}

impl TextDialogBox {
    pub fn new(window_size: RectSize<i32>, prefs: &Prefs, format: &str,
               initial: &str, max_len: usize)
               -> TextDialogBox {
        let paragraph = Paragraph::compile(FONT_SIZE,
                                           LINE_HEIGHT,
                                           MAX_PARAGRAPH_WIDTH,
                                           prefs,
                                           format);
        let textbox_width = (paragraph.width().ceil() as i32)
            .max(2 * BUTTON_MIN_WIDTH + BUTTON_SPACING);
        let width = textbox_width + 2 * MARGIN;

        let textbox_top = MARGIN + (paragraph.height().ceil() as i32) +
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
        let ok_button = TextButton::new_with_key(ok_button_rect,
                                                 "OK",
                                                 (),
                                                 Some(Keycode::Return));

        let cancel_button_rect = Rect::new(ok_button_rect.x - BUTTON_SPACING -
                                               BUTTON_MIN_WIDTH,
                                           rect.y + button_top,
                                           BUTTON_MIN_WIDTH,
                                           BUTTON_HEIGHT);
        let cancel_button = TextButton::new_with_key(cancel_button_rect,
                                                     "Cancel",
                                                     (),
                                                     Some(Keycode::Escape));

        TextDialogBox {
            rect,
            paragraph,
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
        let rect = self.rect.as_f32();
        resources.shaders().ui().draw_dialog(&matrix,
                                             &rect,
                                             &DIALOG_COLOR_1,
                                             &DIALOG_COLOR_2,
                                             &DIALOG_COLOR_3);

        let left = (self.rect.x + MARGIN) as f32;
        let top = (self.rect.y + MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));

        self.textbox.draw(resources, matrix);
        let valid = is_valid(&self.textbox.string());
        self.ok_button.draw(resources, matrix, valid);
        self.cancel_button.draw(resources, matrix, true);
    }

    pub fn on_event<F>(&mut self, event: &Event, ui: &mut Ui, is_valid: F)
                       -> Option<Option<String>>
    where
        F: Fn(&str) -> bool,
    {
        self.textbox.on_event(event, ui);
        let string = self.textbox.string();
        let valid = is_valid(string);
        if let Some(()) = self.ok_button.on_event(event, ui, valid) {
            return Some(Some(string.to_string()));
        }
        if let Some(()) = self.cancel_button.on_event(event, ui, true) {
            return Some(None);
        }
        if event.is_mouse() {
            ui.cursor().request(Cursor::default());
        }
        return None;
    }
}

//===========================================================================//
