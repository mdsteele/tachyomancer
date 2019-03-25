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

use super::super::tooltip::Tooltip;
use cgmath::Matrix4;
use tachy::geom::{AsFloat, Color4, Rect, RectSize};
use tachy::gui::{Event, Resources};
use tachy::save::{Hotkey, Prefs, Puzzle};

//===========================================================================//

const BUTTON_WIDTH: i32 = 48;
const BUTTON_HEIGHT: i32 = 32;
const BUTTON_SPACING: i32 = 8;

const TRAY_MARGIN: i32 = 12;
const TRAY_HEIGHT: i32 = 2 * TRAY_MARGIN + BUTTON_HEIGHT;

const TOOLTIP_RESET: &str = "\
    $*Reset simulation$* $>$G$*$[EvalReset]$*$D$<\n\
    Resets the simulation back to the beginning and returns to edit mode.";
const TOOLTIP_RUN_PAUSE: &str = "\
    $*Run/pause$* $>$G$*$[EvalRunPause]$*$D$<\n\
    Runs or pauses the simulation.";
const TOOLTIP_STEP_SUBCYCLE: &str = "\
    $*Step forward one subcycle$* $>$G$*$[EvalStepSubcycle]$*$D$<\n\
    Runs the simulation forward by a single subcycle, then pauses.  This \
    allows you to see how data is flowing through your circuit, one chip at \
    a time.";
const TOOLTIP_STEP_CYCLE: &str = "\
    $*Step forward one cycle$* $>$G$*$[EvalStepCycle]$*$D$<\n\
    Runs the simulation forward until the end of the current cycle, then \
    pauses.  This allows you to see event loops in your circuit, running \
    one iteration at a time.";
const TOOLTIP_STEP_TIME: &str = "\
    $*Step forward one time step$* $>$G$*$[EvalStepTime]$*$D$<\n\
    Runs the simulation forward until the end of the current time step, \
    then pauses.";

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlsAction {
    Reset,
    RunOrPause,
    StepSubcycle,
    StepCycle,
    StepTime,
}

impl ControlsAction {
    fn tooltip_format(self) -> &'static str {
        match self {
            ControlsAction::Reset => TOOLTIP_RESET,
            ControlsAction::RunOrPause => TOOLTIP_RUN_PAUSE,
            ControlsAction::StepSubcycle => TOOLTIP_STEP_SUBCYCLE,
            ControlsAction::StepCycle => TOOLTIP_STEP_CYCLE,
            ControlsAction::StepTime => TOOLTIP_STEP_TIME,
        }
    }
}

//===========================================================================//

pub struct ControlsTray {
    rect: Rect<i32>,
    buttons: Vec<ControlsButton>,
    tooltip: Tooltip<ControlsAction>,
}

impl ControlsTray {
    pub fn new(window_size: RectSize<i32>, current_puzzle: Puzzle)
               -> ControlsTray {
        let mut actions =
            vec![
                (ControlsAction::Reset, Hotkey::EvalReset),
                (ControlsAction::RunOrPause, Hotkey::EvalRunPause),
                (ControlsAction::StepSubcycle, Hotkey::EvalStepSubcycle),
            ];
        if current_puzzle.allows_events() {
            actions.push((ControlsAction::StepCycle, Hotkey::EvalStepCycle));
        }
        actions.push((ControlsAction::StepTime, Hotkey::EvalStepTime));
        let width = 2 * TRAY_MARGIN +
            (actions.len() as i32) * (BUTTON_WIDTH + BUTTON_SPACING) -
            BUTTON_SPACING;
        let rect = Rect::new((window_size.width - width) / 2,
                             window_size.height - TRAY_HEIGHT,
                             width,
                             TRAY_HEIGHT);
        let buttons = actions
            .into_iter()
            .enumerate()
            .map(|(index, (action, hotkey))| {
                let rect = Rect::new(rect.x + TRAY_MARGIN +
                                         (BUTTON_WIDTH + BUTTON_SPACING) *
                                             (index as i32),
                                     rect.y + TRAY_MARGIN,
                                     BUTTON_WIDTH,
                                     BUTTON_HEIGHT);
                ControlsButton::new(action, rect, hotkey)
            })
            .collect::<Vec<ControlsButton>>();
        let tooltip = Tooltip::new(window_size);
        ControlsTray {
            rect,
            buttons,
            tooltip,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let ui = resources.shaders().ui();
        ui.draw_box2(matrix,
                     &self.rect.as_f32(),
                     &Color4::ORANGE2,
                     &Color4::CYAN2,
                     &Color4::PURPLE0.with_alpha(0.8));
        for button in self.buttons.iter() {
            button.draw(resources, matrix);
        }
        self.tooltip.draw(resources, matrix);
    }

    pub fn on_event(&mut self, event: &Event, prefs: &Prefs)
                    -> Option<Option<ControlsAction>> {
        for button in self.buttons.iter_mut() {
            let opt_action = button.on_event(event, prefs, &mut self.tooltip);
            if opt_action.is_some() {
                return Some(opt_action);
            }
        }
        match event {
            Event::ClockTick(tick) => {
                self.tooltip.tick(tick, prefs, |action| {
                    action.tooltip_format().to_string()
                });
                None
            }
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
}

impl ControlsButton {
    pub fn new(action: ControlsAction, rect: Rect<i32>, hotkey: Hotkey)
               -> ControlsButton {
        ControlsButton {
            action,
            rect,
            hotkey,
            hovering: false,
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

    pub fn on_event(&mut self, event: &Event, prefs: &Prefs,
                    tooltip: &mut Tooltip<ControlsAction>)
                    -> Option<ControlsAction> {
        match event {
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
                if self.hovering {
                    tooltip.start_hover(self.action, mouse.pt);
                } else {
                    tooltip.stop_hover(&self.action);
                }
            }
            Event::Unfocus => {
                self.hovering = false;
                tooltip.stop_hover(&self.action);
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
