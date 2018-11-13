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

use cgmath::{Matrix4, vec3};
use tachy::gui::{Event, Resources};
use tachy::state::{Rect, RectSize};

//===========================================================================//

const BUTTON_WIDTH: i32 = 48;
const BUTTON_HEIGHT: i32 = 32;
const BUTTON_SPACING: i32 = 8;
const TRAY_MARGIN: i32 = 12;
const TRAY_HEIGHT: i32 = 2 * TRAY_MARGIN + BUTTON_HEIGHT;

//===========================================================================//

#[derive(Clone, Copy, Debug)]
pub enum ControlsAction {
    Reset,
    RunOrPause,
    StepSubcycle,
    StepCycle,
    StepTime,
}

//===========================================================================//

pub struct ControlsTray {
    buttons: Vec<ControlsButton>,
    rect: Rect<i32>,
}

impl ControlsTray {
    pub fn new(window_size: RectSize<u32>, allow_step_cycle: bool)
               -> ControlsTray {
        let mut actions = vec![
            ControlsAction::Reset,
            ControlsAction::RunOrPause,
            ControlsAction::StepSubcycle,
        ];
        if allow_step_cycle {
            actions.push(ControlsAction::StepCycle);
        }
        actions.push(ControlsAction::StepTime);
        let buttons = actions
            .into_iter()
            .enumerate()
            .map(|(index, action)| ControlsButton::new(action, index as i32))
            .collect::<Vec<ControlsButton>>();
        let width = 2 * TRAY_MARGIN +
            (buttons.len() as i32) * (BUTTON_WIDTH + BUTTON_SPACING) -
            BUTTON_SPACING;
        let rect = Rect::new(((window_size.width as i32) - width) / 2,
                             (window_size.height as i32) - TRAY_HEIGHT,
                             width,
                             TRAY_HEIGHT);
        ControlsTray { buttons, rect }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let x = self.rect.x() as f32;
        let y = self.rect.y() as f32;
        let w = self.rect.width() as f32;
        let h = self.rect.height() as f32;
        resources
            .shaders()
            .solid()
            .fill_rect(matrix, (0.0, 0.5, 0.0), (x, y, w, h));
        let matrix = matrix * Matrix4::from_translation(vec3(x, y, 0.0));
        for button in self.buttons.iter() {
            button.draw(resources, &matrix);
        }
    }

    pub fn handle_event(&mut self, event: &Event)
                        -> Option<Option<ControlsAction>> {
        for button in self.buttons.iter_mut() {
            let point = self.rect.top_left();
            let opt_action = button.handle_event(&event.relative_to(point));
            if opt_action.is_some() {
                return Some(opt_action);
            }
        }
        match event {
            Event::MouseDown(mouse) if self.rect.contains_point(mouse.pt) => {
                Some(None)
            }
            _ => None,
        }
    }
}

//===========================================================================//

struct ControlsButton {
    action: ControlsAction,
    rect: Rect<i32>,
}

impl ControlsButton {
    pub fn new(action: ControlsAction, index: i32) -> ControlsButton {
        ControlsButton {
            action,
            rect: Rect::new(TRAY_MARGIN +
                                (BUTTON_WIDTH + BUTTON_SPACING) * index,
                            TRAY_MARGIN,
                            BUTTON_WIDTH,
                            BUTTON_HEIGHT),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let x = self.rect.x() as f32;
        let y = self.rect.y() as f32;
        let w = self.rect.width() as f32;
        let h = self.rect.height() as f32;
        resources
            .shaders()
            .solid()
            .fill_rect(matrix, (0.75, 0.0, 0.0), (x, y, w, h));
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<ControlsAction> {
        match event {
            Event::MouseDown(mouse) => {
                if mouse.left && self.rect.contains_point(mouse.pt) {
                    return Some(self.action);
                }
            }
            // TODO: support hotkeys
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
