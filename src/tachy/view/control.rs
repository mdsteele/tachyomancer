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

use super::paragraph::Paragraph;
use cgmath::Matrix4;
use tachy::geom::{MatrixExt, Rect, RectSize};
use tachy::gui::{Event, Resources};
use tachy::save::{Hotkey, Prefs, Puzzle};

//===========================================================================//

const BUTTON_WIDTH: i32 = 48;
const BUTTON_HEIGHT: i32 = 32;
const BUTTON_SPACING: i32 = 8;

const TOOLTIP_FONT_SIZE: f32 = 20.0;
const TOOLTIP_HOVER_TIME: f64 = 0.5;
const TOOLTIP_LINE_HEIGHT: f32 = 22.0;
const TOOLTIP_INNER_MARGIN: f32 = 10.0;
const TOOLTIP_WIDTH: f32 = 360.0;

const TRAY_MARGIN: i32 = 12;
const TRAY_HEIGHT: i32 = 2 * TRAY_MARGIN + BUTTON_HEIGHT;

const TOOLTIP_RESET: &str = "\
    $*Reset simulation$* $>$P$*$[EvalReset]$*$K$<\n\
    Resets the simulation back to the beginning and returns to edit mode.";
const TOOLTIP_RUN_PAUSE: &str = "\
    $*Run/pause$* $>$P$*$[EvalRunPause]$*$K$<\n\
    Runs or pauses the simulation.";
const TOOLTIP_STEP_SUBCYCLE: &str = "\
    $*Step forward one subcycle$* $>$P$*$[EvalStepSubcycle]$*$K$<\n\
    Runs the simulation forward by a single subcycle, then pauses.  This \
    allows you to see how data is flowing through your circuit, one chip at \
    a time.";
const TOOLTIP_STEP_CYCLE: &str = "\
    $*Step forward one cycle$* $>$P$*$[EvalStepCycle]$*$K$<\n\
    Runs the simulation forward until the end of the current cycle, then \
    pauses.  This allows you to see event loops in your circuit, running \
    one iteration at a time.";
const TOOLTIP_STEP_TIME: &str = "\
    $*Step forward one time step$* $>$P$*$[EvalStepTime]$*$K$<\n\
    Runs the simulation forward until the end of the current time step, \
    then pauses.";

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
    pub fn new(window_size: RectSize<u32>, current_puzzle: Puzzle,
               prefs: &Prefs)
               -> ControlsTray {
        let mut actions =
            vec![
                (ControlsAction::Reset, Hotkey::EvalReset, TOOLTIP_RESET),
                (ControlsAction::RunOrPause,
                 Hotkey::EvalRunPause,
                 TOOLTIP_RUN_PAUSE),
                (ControlsAction::StepSubcycle,
                 Hotkey::EvalStepSubcycle,
                 TOOLTIP_STEP_SUBCYCLE),
            ];
        if current_puzzle.allows_events() {
            actions.push((ControlsAction::StepCycle,
                          Hotkey::EvalStepCycle,
                          TOOLTIP_STEP_CYCLE));
        }
        actions.push((ControlsAction::StepTime,
                      Hotkey::EvalStepTime,
                      TOOLTIP_STEP_TIME));
        let buttons = actions
            .into_iter()
            .enumerate()
            .map(|(index, (action, hotkey, tooltip))| {
                     ControlsButton::new(action,
                                         index as i32,
                                         hotkey,
                                         tooltip,
                                         prefs)
                 })
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
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(matrix, (0.0, 0.5, 0.0), rect);
        let matrix = matrix * Matrix4::trans2(rect.x, rect.y);
        for button in self.buttons.iter() {
            button.draw(resources, &matrix);
        }
        for button in self.buttons.iter() {
            button.draw_tooltip(resources, &matrix);
        }
    }

    pub fn on_event(&mut self, event: &Event, prefs: &Prefs)
                    -> Option<Option<ControlsAction>> {
        for button in self.buttons.iter_mut() {
            let point = self.rect.top_left();
            let opt_action = button.on_event(&event.relative_to(point), prefs);
            if opt_action.is_some() {
                return Some(opt_action);
            }
        }
        match event {
            Event::MouseDown(mouse) if self.rect.contains_point(mouse.pt) => {
                Some(None)
            }
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
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
    hotkey: Hotkey,
    hovering: bool,
    hover_time: f64,
    tooltip: Paragraph,
}

impl ControlsButton {
    pub fn new(action: ControlsAction, index: i32, hotkey: Hotkey,
               tooltip: &str, prefs: &Prefs)
               -> ControlsButton {
        ControlsButton {
            action,
            rect: Rect::new(TRAY_MARGIN +
                                (BUTTON_WIDTH + BUTTON_SPACING) * index,
                            TRAY_MARGIN,
                            BUTTON_WIDTH,
                            BUTTON_HEIGHT),
            hotkey,
            hovering: false,
            hover_time: 0.0,
            tooltip: Paragraph::compile(TOOLTIP_FONT_SIZE,
                                        TOOLTIP_LINE_HEIGHT,
                                        TOOLTIP_WIDTH,
                                        prefs,
                                        tooltip),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = if self.hovering {
            (1.0, 0.2, 0.2)
        } else {
            (0.75, 0.0, 0.0)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(matrix, color, rect);
    }

    pub fn draw_tooltip(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        if self.hovering && self.hover_time >= TOOLTIP_HOVER_TIME {
            let width = self.tooltip.width() + 2.0 * TOOLTIP_INNER_MARGIN;
            let height = self.tooltip.height() + 2.0 * TOOLTIP_INNER_MARGIN;
            let rect = Rect::new((self.rect.x as f32) +
                                     0.5 * (self.rect.width as f32) -
                                     0.5 * width,
                                 (self.rect.y as f32) - (height + 10.0),
                                 width,
                                 height);
            resources
                .shaders()
                .solid()
                .fill_rect(matrix, (0.9, 0.9, 0.9), rect);
            self.tooltip.draw(resources,
                              matrix,
                              (rect.x + TOOLTIP_INNER_MARGIN,
                               rect.y + TOOLTIP_INNER_MARGIN));
        }
    }

    pub fn on_event(&mut self, event: &Event, prefs: &Prefs)
                    -> Option<ControlsAction> {
        match event {
            Event::ClockTick(tick) => {
                if self.hovering {
                    self.hover_time = (self.hover_time + tick.elapsed)
                        .min(TOOLTIP_HOVER_TIME);
                }
            }
            Event::KeyDown(key) => {
                if key.code == prefs.hotkey_code(self.hotkey) {
                    // TODO: play sound
                    return Some(self.action);
                }
            }
            Event::MouseDown(mouse) => {
                if mouse.left && self.rect.contains_point(mouse.pt) {
                    // TODO: play sound
                    return Some(self.action);
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
                if !self.hovering {
                    self.hover_time = 0.0;
                }
            }
            Event::Unfocus => {
                self.hovering = false;
                self.hover_time = 0.0;
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
