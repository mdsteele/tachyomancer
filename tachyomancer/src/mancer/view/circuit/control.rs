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
use super::super::tooltip::TooltipSink;
use super::tutorial::TutorialBubble;
use crate::mancer::font::Align;
use crate::mancer::gui::{Cursor, Event, Resources, Sound, Ui};
use crate::mancer::save::{Hotkey, HotkeyCodeExt, Prefs};
use cgmath::{Matrix4, Point2};
use tachy::geom::{AsFloat, Color4, Rect, RectSize};
use tachy::save::Puzzle;
use tachy::state::EditGrid;

//===========================================================================//

const BUTTON_WIDTH: i32 = 48;
const BUTTON_HEIGHT: i32 = 32;
const BUTTON_SPACING: i32 = 8;

const TIMER_FONT_SIZE: f32 = 24.0;
const TIMER_HEIGHT: i32 = 24;

const TRAY_MARGIN: i32 = 12;
const TRAY_HEIGHT: i32 = 3 * TRAY_MARGIN + TIMER_HEIGHT + BUTTON_HEIGHT;

const TOOLTIP_RESET: &str = "$*Reset simulation$* $>$G$*$[EvalReset]$*$D$<\n\
     Resets the simulation back to the beginning and returns to edit mode.";
const TOOLTIP_RUN_PAUSE: &str = "$*Run/pause$* $>$G$*$[EvalRunPause]$*$D$<\n\
                                 Runs or pauses the simulation.";
const TOOLTIP_STEP_SUBCYCLE: &str =
    "$*Step forward one subcycle$* $>$G$*$[EvalStepSubcycle]$*$D$<\n\
     Runs the simulation forward by a single subcycle, then pauses.  This \
     allows you to see how data is flowing through your circuit, one chip at \
     a time.";
const TOOLTIP_STEP_CYCLE: &str =
    "$*Step forward one cycle$* $>$G$*$[EvalStepCycle]$*$D$<\n\
     Runs the simulation forward until the end of the current cycle, then \
     pauses.  This allows you to see event loops in your circuit, running \
     one iteration at a time.";
const TOOLTIP_STEP_TIME: &str =
    "$*Step forward one time step$* $>$G$*$[EvalStepTime]$*$D$<\n\
     Runs the simulation forward until the end of the current time step, \
     then pauses.";

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlsStatus {
    Stopped,
    Running,
    Paused,
    Finished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlsAction {
    Reset,
    RunOrPause,
    StepSubcycle,
    StepCycle,
    StepTime,
}

impl ControlsAction {
    fn icon_index(self, status: ControlsStatus) -> usize {
        match self {
            ControlsAction::Reset => 2,
            ControlsAction::RunOrPause => {
                if status == ControlsStatus::Running {
                    1
                } else {
                    0
                }
            }
            ControlsAction::StepSubcycle => 3,
            ControlsAction::StepCycle => 4,
            ControlsAction::StepTime => 5,
        }
    }

    pub fn tooltip_format(self) -> &'static str {
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
    tutorial_bubble: Option<TutorialBubble>,
    show_cycle_count: bool,
}

impl ControlsTray {
    pub fn new(
        window_size: RectSize<i32>,
        current_puzzle: Puzzle,
        tutorial_bubble: Option<TutorialBubble>,
    ) -> ControlsTray {
        let show_cycle_count = current_puzzle.allows_events();
        let mut actions = vec![
            (ControlsAction::Reset, Hotkey::EvalReset),
            (ControlsAction::RunOrPause, Hotkey::EvalRunPause),
            (ControlsAction::StepSubcycle, Hotkey::EvalStepSubcycle),
        ];
        if show_cycle_count {
            actions.push((ControlsAction::StepCycle, Hotkey::EvalStepCycle));
        }
        actions.push((ControlsAction::StepTime, Hotkey::EvalStepTime));
        let width = 2 * TRAY_MARGIN
            + (actions.len() as i32) * (BUTTON_WIDTH + BUTTON_SPACING)
            - BUTTON_SPACING;
        let rect = Rect::new(
            (window_size.width - width) / 2,
            window_size.height - TRAY_HEIGHT,
            width,
            TRAY_HEIGHT,
        );
        let buttons = actions
            .into_iter()
            .enumerate()
            .map(|(index, (action, hotkey))| {
                let rect = Rect::new(
                    rect.x
                        + TRAY_MARGIN
                        + (BUTTON_WIDTH + BUTTON_SPACING) * (index as i32),
                    rect.bottom() - (BUTTON_HEIGHT + TRAY_MARGIN),
                    BUTTON_WIDTH,
                    BUTTON_HEIGHT,
                );
                ControlsButton::new(action, rect, hotkey)
            })
            .collect::<Vec<ControlsButton>>();
        ControlsTray { rect, buttons, tutorial_bubble, show_cycle_count }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        status: ControlsStatus,
        grid: &EditGrid,
    ) {
        let ui = resources.shaders().ui();
        ui.draw_box2(
            matrix,
            &self.rect.as_f32(),
            &Color4::ORANGE2,
            &Color4::CYAN2,
            &Color4::PURPLE0_TRANSLUCENT,
        );

        // Timers:
        let (time_step, cycle, subcycle) = if let Some(eval) = grid.eval() {
            (eval.time_step(), eval.cycle(), eval.subcycle())
        } else {
            (0, 0, 0)
        };
        let timer_top = (self.rect.y + TRAY_MARGIN) as f32;
        resources.fonts().roman().draw(
            matrix,
            TIMER_FONT_SIZE,
            Align::TopLeft,
            ((self.rect.x + TRAY_MARGIN) as f32, timer_top),
            &format!("Sub:{}", subcycle),
        );
        if self.show_cycle_count {
            resources.fonts().roman().draw(
                matrix,
                TIMER_FONT_SIZE,
                Align::TopRight,
                ((self.rect.right() - TRAY_MARGIN - 100) as f32, timer_top),
                &format!("Cycle:{}", cycle),
            );
        }
        resources.fonts().roman().draw(
            matrix,
            TIMER_FONT_SIZE,
            Align::TopRight,
            ((self.rect.right() - TRAY_MARGIN) as f32, timer_top),
            &format!("Time:{}", time_step),
        );

        // Buttons:
        for button in self.buttons.iter() {
            button.draw(resources, matrix, status, grid.has_errors());
        }
        if let Some(ref bubble) = self.tutorial_bubble {
            let topleft = Point2::new(self.rect.x - 230, self.rect.y - 24);
            bubble.draw(resources, matrix, topleft);
        }
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        status: ControlsStatus,
        has_errors: bool,
        is_interacting: bool,
        tooltip: &mut dyn TooltipSink<ControlsAction>,
        prefs: &Prefs,
    ) -> Option<Option<ControlsAction>> {
        for button in self.buttons.iter_mut() {
            let opt_action = button.on_event(
                event,
                ui,
                status,
                has_errors || is_interacting,
                tooltip,
                prefs,
            );
            if opt_action.is_some() {
                return Some(opt_action);
            }
        }
        match event {
            Event::MouseDown(mouse) if self.rect.contains_point(mouse.pt) => {
                return Some(None);
            }
            Event::MouseMove(mouse) | Event::MouseUp(mouse) => {
                if self.rect.contains_point(mouse.pt) {
                    ui.cursor().request(Cursor::default());
                    tooltip.hover_none(ui);
                }
            }
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                return Some(None);
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//

struct ControlsButton {
    action: ControlsAction,
    rect: Rect<i32>,
    hotkey: Hotkey,
    hover_pulse: HoverPulse,
}

impl ControlsButton {
    pub fn new(
        action: ControlsAction,
        rect: Rect<i32>,
        hotkey: Hotkey,
    ) -> ControlsButton {
        ControlsButton { action, rect, hotkey, hover_pulse: HoverPulse::new() }
    }

    fn is_enabled(&self, status: ControlsStatus) -> bool {
        match self.action {
            ControlsAction::Reset => status != ControlsStatus::Stopped,
            ControlsAction::RunOrPause => status != ControlsStatus::Finished,
            _ => {
                status != ControlsStatus::Running
                    && status != ControlsStatus::Finished
            }
        }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        status: ControlsStatus,
        has_errors: bool,
    ) {
        let ui = resources.shaders().ui();
        let enabled = !has_errors && self.is_enabled(status);

        let rect = self.rect.as_f32();
        let bg_color = if !enabled {
            Color4::new(1.0, 1.0, 1.0, 0.1)
        } else {
            Color4::PURPLE0_TRANSLUCENT.mix(
                Color4::PURPLE3_TRANSLUCENT,
                self.hover_pulse.brightness(),
            )
        };
        ui.draw_box4(
            matrix,
            &rect,
            &Color4::ORANGE4,
            &Color4::CYAN3,
            &bg_color,
        );

        let icon_rect = Rect::new(
            rect.x + 0.5 * (rect.width - rect.height),
            rect.y,
            rect.height,
            rect.height,
        );
        let icon_index = self.action.icon_index(status);
        if enabled {
            ui.draw_controls_icon(
                matrix,
                &icon_rect,
                icon_index,
                &Color4::ORANGE4,
                &Color4::ORANGE3,
                &Color4::ORANGE2,
            );
        } else {
            ui.draw_controls_icon(
                matrix,
                &icon_rect,
                icon_index,
                &Color4::new(0.8, 0.8, 0.8, 1.0),
                &Color4::new(0.6, 0.6, 0.6, 1.0),
                &Color4::new(0.4, 0.4, 0.4, 1.0),
            );
        }
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        status: ControlsStatus,
        controls_disabled: bool,
        tooltip: &mut dyn TooltipSink<ControlsAction>,
        prefs: &Prefs,
    ) -> Option<ControlsAction> {
        match event {
            Event::ClockTick(tick) => {
                self.hover_pulse.on_clock_tick(tick, ui);
            }
            Event::KeyDown(key) => {
                if !controls_disabled
                    && self.is_enabled(status)
                    && key.code == prefs.hotkey_code(self.hotkey).to_keycode()
                {
                    self.hover_pulse.on_click(ui);
                    ui.audio().play_sound(Sound::ButtonClick);
                    return Some(self.action);
                }
            }
            Event::MouseDown(mouse) if mouse.left => {
                if !controls_disabled
                    && self.is_enabled(status)
                    && self.rect.contains_point(mouse.pt)
                {
                    self.hover_pulse.on_click(ui);
                    ui.audio().play_sound(Sound::ButtonClick);
                    return Some(self.action);
                }
            }
            Event::MouseMove(mouse) => {
                let hovering = self.rect.contains_point(mouse.pt);
                if hovering {
                    tooltip.hover_tag(mouse.pt, ui, self.action);
                }
                if self.hover_pulse.set_hovering(hovering, ui)
                    && !controls_disabled
                    && self.is_enabled(status)
                {
                    ui.audio().play_sound(Sound::ButtonHover);
                }
            }
            Event::Unfocus => {
                self.hover_pulse.unfocus();
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
