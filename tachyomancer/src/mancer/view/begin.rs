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
use super::dialog::ButtonDialogBox;
use super::paragraph::Paragraph;
use crate::mancer::font::Align;
use crate::mancer::gui::{Event, Keycode, Resources, Sound, Ui};
use crate::mancer::save::PROFILE_NAME_MAX_CHARS;
use crate::mancer::state::GameState;
use cgmath::{self, Matrix4};
use tachy::geom::{AsInt, Color3, Rect, RectSize};

//===========================================================================//

const BUTTON_WIDTH: i32 = 200;
const BUTTON_HEIGHT: i32 = 50;
const BUTTON_SPACING: i32 = 100;

const FONT_SIZE: f32 = 40.0;

//===========================================================================//

pub enum BeginAction {
    CreateProfile(String),
}

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
enum BeginPhase {
    Entry,
    Confirm,
    ErrorEmpty,
    ErrorTaken,
}

//===========================================================================//

pub struct BeginView {
    width: f32,
    height: f32,
    phase: BeginPhase,
    text_entry: TextEntry,
    back_button: TextButton<()>,
    confirm_button: TextButton<()>,
    error_dialog: Option<ButtonDialogBox<()>>,
}

impl BeginView {
    pub fn new(window_size: RectSize<i32>, _state: &GameState) -> BeginView {
        let back_button_rect = Rect::new(
            (window_size.width - BUTTON_SPACING) / 2 - BUTTON_WIDTH,
            600,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
        );
        let back_button = TextButton::new(back_button_rect, "Go back", ());

        let confirm_button_rect = Rect::new(
            (window_size.width + BUTTON_SPACING) / 2,
            600,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
        );
        let confirm_button =
            TextButton::new(confirm_button_rect, "That's right", ());

        BeginView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            phase: BeginPhase::Entry,
            text_entry: TextEntry::new(window_size.width / 2, 300),
            back_button,
            confirm_button,
            error_dialog: None,
        }
    }

    pub fn draw(&self, resources: &Resources, _state: &GameState) {
        let matrix =
            cgmath::ortho(0.0, self.width, self.height, 0.0, -1.0, 1.0);
        let rect = Rect::new(0.0, 0.0, self.width, self.height);
        resources.shaders().solid().fill_rect(
            &matrix,
            Color3::new(0.1, 0.1, 0.1),
            rect,
        );
        let font = resources.fonts().roman();
        if let BeginPhase::Confirm = self.phase {
            font.draw(
                &matrix,
                FONT_SIZE,
                Align::TopCenter,
                (0.5 * self.width, 500.0),
                &format!("\"Commander {}, is it?\"", self.text_entry.text),
            );
            self.back_button.draw(resources, &matrix, true);
            self.confirm_button.draw(resources, &matrix, true);
        } else {
            font.draw(
                &matrix,
                FONT_SIZE,
                Align::TopCenter,
                (0.5 * self.width, 100.0),
                "Type your name, then press ENTER:",
            );
        }
        if self.phase == BeginPhase::ErrorEmpty {
            font.draw(
                &matrix,
                FONT_SIZE,
                Align::TopCenter,
                (0.5 * self.width, 500.0),
                "Your name must not be empty.",
            );
        } else if self.phase == BeginPhase::ErrorTaken {
            font.draw(
                &matrix,
                FONT_SIZE,
                Align::TopCenter,
                (0.5 * self.width, 500.0),
                "That name is already taken.",
            );
        }
        self.text_entry.draw(resources, &matrix);
        if let Some(ref dialog) = self.error_dialog {
            dialog.draw(resources, &matrix);
        }
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<BeginAction> {
        if let Some(ref mut dialog) = self.error_dialog {
            match dialog.on_event(event, ui) {
                Some(()) => self.error_dialog = None,
                None => {}
            }
            if !event.is_clock_tick() {
                return None;
            }
        }

        match event {
            Event::KeyDown(_) | Event::MouseDown(_) => match self.phase {
                BeginPhase::Entry | BeginPhase::Confirm => {}
                BeginPhase::ErrorEmpty | BeginPhase::ErrorTaken => {
                    self.phase = BeginPhase::Entry;
                    ui.request_redraw();
                }
            },
            _ => {}
        }
        if let BeginPhase::Confirm = self.phase {
            if let Some(()) = self.back_button.on_event(event, ui, true) {
                self.phase = BeginPhase::Entry;
                ui.request_redraw();
            }
            if let Some(()) = self.confirm_button.on_event(event, ui, true) {
                let name = self.text_entry.text.clone();
                return Some(BeginAction::CreateProfile(name));
            }
        } else {
            if self.text_entry.on_event(event, ui) {
                let name = self.text_entry.text.as_str();
                if name.is_empty() {
                    ui.audio().play_sound(Sound::Beep);
                    self.phase = BeginPhase::ErrorEmpty;
                    ui.request_redraw();
                } else if state.has_profile(name) {
                    ui.audio().play_sound(Sound::Beep);
                    self.phase = BeginPhase::ErrorTaken;
                    ui.request_redraw();
                } else {
                    self.phase = BeginPhase::Confirm;
                    ui.request_redraw();
                }
            }
        }
        return None;
    }

    pub fn show_error(
        &mut self,
        ui: &mut Ui,
        state: &mut GameState,
        unable: &str,
        error: &str,
    ) {
        debug_log!("ERROR: Unable to {}: {}", unable, error);
        // TODO: Play sound for error dialog popup.
        self.on_event(&Event::Unfocus, ui, state);
        let size = RectSize::new(self.width, self.height).as_i32_round();
        let format = format!(
            "$R$*ERROR:$*$D Unable to {}.\n\n{}",
            unable,
            Paragraph::escape(error)
        );
        let buttons = &[("OK", (), Some(Keycode::Return))];
        let dialog =
            ButtonDialogBox::new(size, state.prefs(), &format, buttons);
        self.error_dialog = Some(dialog);
    }
}

//===========================================================================//

struct TextEntry {
    origin: (f32, f32),
    text: String,
}

impl TextEntry {
    fn new(origin_x: i32, origin_y: i32) -> TextEntry {
        TextEntry {
            origin: (origin_x as f32, origin_y as f32),
            text: String::new(),
        }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        resources.fonts().roman().draw(
            matrix,
            60.0,
            Align::TopCenter,
            self.origin,
            &self.text,
        );
    }

    fn on_event(&mut self, event: &Event, ui: &mut Ui) -> bool {
        match event {
            Event::KeyDown(key) => match key.code {
                Keycode::Return => return true,
                Keycode::Backspace => {
                    if self.text.pop().is_some() {
                        ui.request_redraw();
                        ui.audio().play_sound(Sound::TypeKey);
                    }
                }
                _ => {}
            },
            Event::TextInput(text) => {
                for chr in text.chars() {
                    if self.text.chars().count() >= PROFILE_NAME_MAX_CHARS {
                        break;
                    }
                    if (chr >= ' ' && chr <= '~')
                        || (chr >= '\u{a1}' && chr <= '\u{ff}')
                    {
                        self.text.push(chr);
                        ui.request_redraw();
                        ui.audio().play_sound(Sound::TypeKey);
                    }
                }
            }
            _ => {}
        }
        return false;
    }
}

//===========================================================================//
