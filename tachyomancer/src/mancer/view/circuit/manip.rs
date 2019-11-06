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

use super::super::button::HoverPulse;
use crate::mancer::gui::{Event, Resources, Sound, Ui};
use crate::mancer::save::Hotkey;
use cgmath::{Matrix4, Point2, Vector2};
use tachy::geom::{AsFloat, Color4, CoordsRect, Rect};

//===========================================================================//

const BUTTON_RADIUS: i32 = 16;

const TOOLTIP_FLIP_HORZ: &str =
    "$*Flip horizontally$* $>$G$*$[FlipHorz]$*$D$<\n\
     Flips the selection horizontally.\n\
     You can also use the hotkey $[FlipHorz] while dragging.";
const TOOLTIP_FLIP_VERT: &str =
    "$*Flip vertically$* $>$G$*$[FlipVert]$*$D$<\n\
     Flips the selection vertically.\n\
     You can also use the hotkey $[FlipVert] while dragging.";
const TOOLTIP_ROTATE_CCW: &str =
    "$*Rotate counterclockwise$* $>$G$*$[RotateCcw]$*$D$<\n\
     Rotates the selection 90° counterclockwise.\n\
     You can also use the hotkey $[RotateCcw] while dragging.";
const TOOLTIP_ROTATE_CW: &str =
    "$*Rotate clockwise$* $>$G$*$[RotateCw]$*$D$<\n\
     Rotates the selection 90° clockwise.\n\
     You can also use the hotkey $[RotateCw] while dragging.";

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ManipulationAction {
    FlipHorz = 0,
    FlipVert,
    RotateCcw,
    RotateCw,
}

impl ManipulationAction {
    pub fn from_hotkey(hotkey: Hotkey) -> Option<ManipulationAction> {
        match hotkey {
            Hotkey::FlipHorz => Some(ManipulationAction::FlipHorz),
            Hotkey::FlipVert => Some(ManipulationAction::FlipVert),
            Hotkey::RotateCcw => Some(ManipulationAction::RotateCcw),
            Hotkey::RotateCw => Some(ManipulationAction::RotateCw),
            _ => None,
        }
    }

    fn icon_index(self) -> usize {
        self as usize
    }

    pub fn tooltip_format(self) -> String {
        match self {
            ManipulationAction::FlipHorz => TOOLTIP_FLIP_HORZ.to_string(),
            ManipulationAction::FlipVert => TOOLTIP_FLIP_VERT.to_string(),
            ManipulationAction::RotateCcw => TOOLTIP_ROTATE_CCW.to_string(),
            ManipulationAction::RotateCw => TOOLTIP_ROTATE_CW.to_string(),
        }
    }
}

//===========================================================================//

pub struct ManipulationButtons {
    buttons: Vec<ManipButton>,
}

impl ManipulationButtons {
    pub fn new() -> ManipulationButtons {
        ManipulationButtons {
            buttons: vec![
                ManipButton::new(ManipulationAction::FlipHorz, -42, 2),
                ManipButton::new(ManipulationAction::FlipVert, 2, -42),
                ManipButton::new(ManipulationAction::RotateCcw, -42, -42),
                ManipButton::new(ManipulationAction::RotateCw, 46, -42),
            ],
        }
    }

    pub fn hovered_action(&self) -> Option<ManipulationAction> {
        for button in self.buttons.iter() {
            if button.hover_pulse.is_hovering() {
                return Some(button.action);
            }
        }
        return None;
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        selected_rect: CoordsRect,
        grid_cell_size: f32,
    ) {
        let left = ((selected_rect.x as f32) * grid_cell_size).round() as i32;
        let top = ((selected_rect.y as f32) * grid_cell_size).round() as i32;
        let is_square = selected_rect.width == selected_rect.height;
        for button in self.buttons.iter() {
            if is_square || !button.needs_square() {
                button.draw(resources, matrix, left, top);
            }
        }
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        selected_rect: CoordsRect,
        selection_top_left_screen_pt: Point2<i32>,
    ) -> Option<ManipulationAction> {
        let left = selection_top_left_screen_pt.x;
        let top = selection_top_left_screen_pt.y;
        let is_square = selected_rect.width == selected_rect.height;
        for button in self.buttons.iter_mut() {
            if is_square || !button.needs_square() {
                if let Some(action) = button.on_event(event, ui, left, top) {
                    return Some(action);
                }
            }
        }
        return None;
    }

    pub fn unfocus(&mut self) {
        for button in self.buttons.iter_mut() {
            button.unfocus();
        }
    }
}

//===========================================================================//

struct ManipButton {
    action: ManipulationAction,
    offset: Vector2<i32>,
    hover_pulse: HoverPulse,
}

impl ManipButton {
    fn new(
        action: ManipulationAction,
        offset_x: i32,
        offset_y: i32,
    ) -> ManipButton {
        ManipButton {
            action,
            offset: Vector2::new(offset_x, offset_y),
            hover_pulse: HoverPulse::new(),
        }
    }

    fn needs_square(&self) -> bool {
        match self.action {
            ManipulationAction::FlipHorz => false,
            ManipulationAction::FlipVert => false,
            ManipulationAction::RotateCcw => true,
            ManipulationAction::RotateCw => true,
        }
    }

    fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        left: i32,
        top: i32,
    ) {
        let rect = Rect::new(
            left + self.offset.x,
            top + self.offset.y,
            2 * BUTTON_RADIUS,
            2 * BUTTON_RADIUS,
        );
        resources.shaders().ui().draw_manipulation_icon(
            matrix,
            &rect.as_f32(),
            self.action.icon_index(),
            &Color4::ORANGE5,
            &Color4::CYAN4,
            &Color4::PURPLE2
                .mix(Color4::PURPLE4, self.hover_pulse.brightness()),
        );
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        left: i32,
        top: i32,
    ) -> Option<ManipulationAction> {
        let center = Point2::new(left, top)
            + self.offset
            + Vector2::new(BUTTON_RADIUS, BUTTON_RADIUS);
        match event {
            Event::ClockTick(tick) => {
                self.hover_pulse.on_clock_tick(tick, ui);
            }
            Event::MouseDown(mouse) if mouse.left => {
                if button_contains(center, mouse.pt) {
                    self.hover_pulse.on_click(ui);
                    ui.audio().play_sound(Sound::ButtonClick);
                    return Some(self.action);
                }
            }
            Event::MouseMove(mouse) => {
                let hovering = button_contains(center, mouse.pt);
                if self.hover_pulse.set_hovering(hovering, ui) {
                    ui.audio().play_sound(Sound::ButtonHover);
                }
            }
            Event::Unfocus => self.unfocus(),
            _ => {}
        }
        return None;
    }

    fn unfocus(&mut self) {
        self.hover_pulse.unfocus();
    }
}

fn button_contains(center: Point2<i32>, pt: Point2<i32>) -> bool {
    let delta = pt - center;
    delta.x * delta.x + delta.y * delta.y <= BUTTON_RADIUS * BUTTON_RADIUS
}

//===========================================================================//
