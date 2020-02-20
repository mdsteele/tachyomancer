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
use cgmath::{self, Deg, Matrix4, Rad, Vector2};
use tachy::geom::{AsFloat, AsInt, Color3, Color4, MatrixExt, Rect, RectSize};

//===========================================================================//

const BUTTON_WIDTH: i32 = 200;
const BUTTON_HEIGHT: i32 = 44;
const BUTTON_SPACING: i32 = 100;

const ERROR_COLOR: Color4 = Color4::new(0.75, 0.0, 0.0, 1.0);
const FONT_SIZE: f32 = 40.0;
const RANK_PATCH_SPEED: f32 = 1000.0; // pixels/second

//===========================================================================//

pub enum BeginAction {
    CreateProfile(String),
}

//===========================================================================//

enum BeginPhase {
    Entry,
    Confirm(String),
    ErrorEmpty,
    ErrorTaken,
}

//===========================================================================//

pub struct BeginView {
    width: f32,
    height: f32,
    phase: BeginPhase,
    rank_patch: RankPatch,
    back_button: TextButton<()>,
    confirm_button: TextButton<()>,
    error_dialog: Option<ButtonDialogBox<()>>,
}

impl BeginView {
    pub fn new(window_size: RectSize<i32>, _state: &GameState) -> BeginView {
        let button_top = window_size.height * 87 / 100 - BUTTON_HEIGHT / 2;

        let back_button_rect = Rect::new(
            (window_size.width - BUTTON_SPACING) / 2 - BUTTON_WIDTH,
            button_top,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
        );
        let back_button = TextButton::new(back_button_rect, "Go back", ());

        let confirm_button_rect = Rect::new(
            (window_size.width + BUTTON_SPACING) / 2,
            button_top,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
        );
        let confirm_button =
            TextButton::new(confirm_button_rect, "That's right", ());

        let window_size = window_size.as_f32();
        BeginView {
            width: window_size.width,
            height: window_size.height,
            phase: BeginPhase::Entry,
            rank_patch: RankPatch::new(window_size),
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
        self.rank_patch.draw(resources, &matrix);
        let font = resources.fonts().roman();
        if let BeginPhase::Confirm(ref name) = self.phase {
            font.draw(
                &matrix,
                FONT_SIZE,
                Align::MidCenter,
                (0.5 * self.width, (0.77 * self.height).round()),
                &format!("\"Commander {}, is it?\"", name),
            );
            self.back_button.draw(resources, &matrix, true);
            self.confirm_button.draw(resources, &matrix, true);
        } else {
            font.draw(
                &matrix,
                FONT_SIZE,
                Align::MidCenter,
                (0.5 * self.width, (0.1 * self.height).round()),
                "Type your name, then press ENTER:",
            );
        }
        if let BeginPhase::ErrorEmpty = self.phase {
            font.draw_style(
                &matrix,
                FONT_SIZE,
                Align::MidCenter,
                (0.5 * self.width, (0.9 * self.height).round()),
                &ERROR_COLOR,
                0.0,
                "Your name must not be empty.",
            );
        } else if let BeginPhase::ErrorTaken = self.phase {
            font.draw_style(
                &matrix,
                FONT_SIZE,
                Align::MidCenter,
                (0.5 * self.width, (0.9 * self.height).round()),
                &ERROR_COLOR,
                0.0,
                "That name is already taken.",
            );
        }
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
                BeginPhase::Entry | BeginPhase::Confirm(_) => {}
                BeginPhase::ErrorEmpty | BeginPhase::ErrorTaken => {
                    self.phase = BeginPhase::Entry;
                    ui.request_redraw();
                }
            },
            _ => {}
        }

        if let BeginPhase::Confirm(ref name) = self.phase {
            if event.is_clock_tick() {
                self.rank_patch.on_event(event, ui, -0.12 * self.height);
            }
            if let Some(()) = self.confirm_button.on_event(event, ui, true) {
                return Some(BeginAction::CreateProfile(name.clone()));
            }
            if let Some(()) = self.back_button.on_event(event, ui, true) {
                self.phase = BeginPhase::Entry;
                ui.request_redraw();
            }
        } else if let Some(name) = self.rank_patch.on_event(event, ui, 0.0) {
            if name.is_empty() {
                ui.audio().play_sound(Sound::Beep);
                self.phase = BeginPhase::ErrorEmpty;
                ui.request_redraw();
            } else if state.has_profile(&name) {
                ui.audio().play_sound(Sound::Beep);
                self.phase = BeginPhase::ErrorTaken;
                ui.request_redraw();
            } else {
                self.phase = BeginPhase::Confirm(name);
                ui.request_redraw();
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

struct RankPatch {
    radius: f32,
    origin: Vector2<f32>,
    offset: Vector2<f32>,
    text_entry: TextEntry,
}

impl RankPatch {
    fn new(window_size: RectSize<f32>) -> RankPatch {
        let half_width = 0.5 * window_size.width;
        let half_height = 0.5 * window_size.height;
        let radius = 0.3 * window_size.height;
        RankPatch {
            radius,
            origin: Vector2::new(half_width, half_height),
            offset: Vector2::new(half_width + radius, 0.0),
            text_entry: TextEntry::new(0, 89),
        }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let matrix = matrix
            * Matrix4::trans2v(self.origin + self.offset)
            * Matrix4::from_angle_z(Rad(self.offset.x / self.radius))
            * Matrix4::from_scale(self.radius / 256.0);
        resources.shaders().diagram().draw(
            &matrix,
            Rect::new(-256.0, -256.0, 512.0, 512.0),
            Rect::new(0.0, 0.0, 1.0, 1.0),
            resources.textures().diagram_patch(),
        );
        let font = resources.fonts().bold();
        font.draw_style(
            &matrix,
            42.0,
            Align::MidCenter,
            (0.0, -131.0),
            &Color4::ORANGE5,
            0.0,
            "COMMANDER",
        );

        let theta_step = Deg(7.5);
        let label = b"JOINT FEDERATION";
        let mut theta = theta_step * (-0.5 * ((label.len() - 1) as f32));
        for &chr in label.iter() {
            font.draw_chars(
                &(matrix * Matrix4::from_angle_z(theta)),
                56.0,
                Align::MidCenter,
                (0.0, -221.5),
                &Color4::ORANGE4,
                0.0,
                &[chr],
            );
            theta += theta_step;
        }

        let label = b"INTERSTELLAR FRONTIER CORPS";
        let mut theta = theta_step * (0.5 * ((label.len() - 1) as f32));
        for &chr in label.iter() {
            font.draw_chars(
                &(matrix * Matrix4::from_angle_z(theta)),
                56.0,
                Align::MidCenter,
                (0.0, 221.5),
                &Color4::ORANGE4,
                0.0,
                &[chr],
            );
            theta -= theta_step;
        }

        self.text_entry.draw(resources, &matrix);
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        offset_y_goal: f32,
    ) -> Option<String> {
        match event {
            Event::ClockTick(tick) => {
                if self.offset.x != 0.0 {
                    self.offset.x = (self.offset.x
                        - (tick.elapsed as f32) * RANK_PATCH_SPEED)
                        .max(0.0);
                    ui.request_redraw();
                }
                if self.offset.y < offset_y_goal {
                    self.offset.y = (self.offset.y
                        + (tick.elapsed as f32) * RANK_PATCH_SPEED)
                        .min(offset_y_goal);
                    ui.request_redraw();
                } else if self.offset.y > offset_y_goal {
                    self.offset.y = (self.offset.y
                        - (tick.elapsed as f32) * RANK_PATCH_SPEED)
                        .max(offset_y_goal);
                    ui.request_redraw();
                }
            }
            _ => {}
        }
        if self.offset.x == 0.0 {
            self.text_entry.on_event(event, ui)
        } else {
            None
        }
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
        if !self.text.is_empty() {
            let num_chars = self.text.chars().count();
            let font_size = if num_chars <= 5 {
                110.0
            } else {
                let factor = ((num_chars - 5) as f32)
                    / ((PROFILE_NAME_MAX_CHARS - 5) as f32);
                (550.0 + 80.0 * factor) / (num_chars as f32)
            };
            resources.fonts().bold().draw_style(
                matrix,
                font_size,
                Align::MidCenter,
                self.origin,
                &Color4::ORANGE5,
                0.0,
                &self.text,
            );
        }
    }

    fn on_event(&mut self, event: &Event, ui: &mut Ui) -> Option<String> {
        match event {
            Event::KeyDown(key) => match key.code {
                Keycode::Return => return Some(self.text.clone()),
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
        return None;
    }
}

//===========================================================================//
