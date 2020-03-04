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
use super::tray::TraySlide;
use super::tutorial::TutorialBubble;
use crate::mancer::font::Align;
use crate::mancer::gui::{Cursor, Event, Resources, Sound, Ui};
use crate::mancer::save::{Hotkey, HotkeyCodeExt, Prefs};
use crate::mancer::shader::UiShader;
use cgmath::{Deg, Matrix4, Point2};
use tachy::geom::{AsFloat, Color4, MatrixExt, Rect, RectSize};
use tachy::save::Puzzle;
use tachy::state::EditGrid;

//===========================================================================//

const RUN_BUTTON_SIZE: i32 = 60;
const BUTTON_WIDTH: i32 = 48;
const BUTTON_HEIGHT: i32 = 32;
const BUTTON_SPACING: i32 = 8;

const TIMER_FONT_SIZE: f32 = 24.0;

const TRAY_EXTRA_HIDDEN_HEIGHT: i32 = 20;
const TRAY_FLIP_HORZ: bool = false;
const TRAY_INNER_MARGIN: i32 = 12;
const TRAY_TAB_FONT_SIZE: f32 = 16.0;
const TRAY_TAB_HEIGHT: f32 = 66.0;
const TRAY_TAB_TEXT: &str = "CONTROLS";

const TOOLTIP_FAST_FORWARD: &str =
    "$*Fast-forward$* $>$G$*$[EvalFastForward]$*$D$<\n\
     Runs the simulation at increased speed.";
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
    FastForwarding,
    Paused,
    Finished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlsAction {
    FastForward,
    Reset,
    RunOrPause,
    StepSubcycle,
    StepCycle,
    StepTime,
}

impl ControlsAction {
    fn hotkey(self) -> Hotkey {
        match self {
            ControlsAction::FastForward => Hotkey::EvalFastForward,
            ControlsAction::Reset => Hotkey::EvalReset,
            ControlsAction::RunOrPause => Hotkey::EvalRunPause,
            ControlsAction::StepSubcycle => Hotkey::EvalStepSubcycle,
            ControlsAction::StepCycle => Hotkey::EvalStepCycle,
            ControlsAction::StepTime => Hotkey::EvalStepTime,
        }
    }

    fn icon_index(self, status: ControlsStatus) -> usize {
        match self {
            ControlsAction::FastForward => 3,
            ControlsAction::Reset => 2,
            ControlsAction::RunOrPause => {
                match status {
                    ControlsStatus::Stopped
                    | ControlsStatus::Paused
                    | ControlsStatus::Finished => 0, // Run
                    ControlsStatus::Running
                    | ControlsStatus::FastForwarding => 1, // Pause
                }
            }
            ControlsAction::StepSubcycle => 4,
            ControlsAction::StepCycle => 5,
            ControlsAction::StepTime => 6,
        }
    }

    pub fn tooltip_format(self) -> &'static str {
        match self {
            ControlsAction::FastForward => TOOLTIP_FAST_FORWARD,
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
    show_cycle_count: bool,
    slide: TraySlide,
    tutorial_bubble: Option<TutorialBubble>,
}

impl ControlsTray {
    pub fn new(
        window_size: RectSize<i32>,
        current_puzzle: Puzzle,
        tutorial_bubble: Option<TutorialBubble>,
    ) -> ControlsTray {
        let show_cycle_count = current_puzzle.allows_events();
        let mut buttons = Vec::<ControlsButton>::new();
        let mut button_left = TRAY_INNER_MARGIN;
        buttons.push(ControlsButton::new(
            ControlsAction::RunOrPause,
            Rect::new(
                TRAY_INNER_MARGIN,
                window_size.height - (RUN_BUTTON_SIZE + TRAY_INNER_MARGIN),
                RUN_BUTTON_SIZE,
                RUN_BUTTON_SIZE,
            ),
        ));
        button_left += RUN_BUTTON_SIZE + BUTTON_SPACING;
        let button_top = window_size.height
            - (BUTTON_HEIGHT / 2 + RUN_BUTTON_SIZE / 2 + TRAY_INNER_MARGIN);
        buttons.push(ControlsButton::new(
            ControlsAction::FastForward,
            Rect::new(button_left, button_top, BUTTON_WIDTH, BUTTON_HEIGHT),
        ));
        button_left += BUTTON_WIDTH + BUTTON_SPACING;
        buttons.push(ControlsButton::new(
            ControlsAction::Reset,
            Rect::new(button_left, button_top, BUTTON_WIDTH, BUTTON_HEIGHT),
        ));
        let tray_width = button_left + BUTTON_WIDTH + TRAY_INNER_MARGIN;
        let button_left =
            TRAY_INNER_MARGIN + RUN_BUTTON_SIZE / 2 - BUTTON_WIDTH / 2;
        let mut button_top = window_size.height
            - (BUTTON_HEIGHT
                + BUTTON_SPACING
                + RUN_BUTTON_SIZE
                + TRAY_INNER_MARGIN);
        buttons.push(ControlsButton::new(
            ControlsAction::StepTime,
            Rect::new(button_left, button_top, BUTTON_WIDTH, BUTTON_HEIGHT),
        ));
        if show_cycle_count {
            button_top -= BUTTON_HEIGHT + BUTTON_SPACING;
            buttons.push(ControlsButton::new(
                ControlsAction::StepCycle,
                Rect::new(
                    button_left,
                    button_top,
                    BUTTON_WIDTH,
                    BUTTON_HEIGHT,
                ),
            ));
        }
        button_top -= BUTTON_HEIGHT + BUTTON_SPACING;
        buttons.push(ControlsButton::new(
            ControlsAction::StepSubcycle,
            Rect::new(button_left, button_top, BUTTON_WIDTH, BUTTON_HEIGHT),
        ));
        let tray_top = button_top - TRAY_INNER_MARGIN;
        let rect = Rect::new(
            0,
            tray_top,
            tray_width,
            window_size.height - tray_top + TRAY_EXTRA_HIDDEN_HEIGHT,
        );
        ControlsTray {
            rect,
            buttons,
            show_cycle_count,
            slide: TraySlide::new(rect.width),
            tutorial_bubble,
        }
    }

    pub fn rect(&self) -> Rect<i32> {
        self.rect
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        status: ControlsStatus,
        grid: &EditGrid,
    ) {
        let matrix =
            matrix * Matrix4::trans2(-self.slide.distance() as f32, 0.0);
        self.draw_box(resources, &matrix);
        self.draw_timers(resources, &matrix, grid);
        for button in self.buttons.iter() {
            button.draw(resources, &matrix, status, grid.has_errors());
        }
        if let Some(ref bubble) = self.tutorial_bubble {
            let left = self.rect.right() + 26;
            let top = self.rect.bottom()
                - TRAY_EXTRA_HIDDEN_HEIGHT
                - 8
                - bubble.height();
            bubble.draw(resources, &matrix, Point2::new(left, top));
        }
    }

    fn draw_box(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let rect = self.rect.as_f32();
        let tab_rect =
            UiShader::tray_tab_rect(rect, TRAY_TAB_HEIGHT, TRAY_FLIP_HORZ);

        resources.shaders().ui().draw_tray(
            matrix,
            &rect,
            TRAY_TAB_HEIGHT,
            TRAY_FLIP_HORZ,
            &Color4::ORANGE2,
            &Color4::CYAN2,
            &Color4::PURPLE0_TRANSLUCENT,
        );

        let tab_matrix = matrix
            * Matrix4::trans2(
                tab_rect.x + 0.5 * tab_rect.width,
                tab_rect.y + 0.5 * tab_rect.height,
            )
            * Matrix4::from_angle_z(Deg(-90.0));
        let font = resources.fonts().roman();
        font.draw(
            &tab_matrix,
            TRAY_TAB_FONT_SIZE,
            Align::MidCenter,
            (0.0, -2.0),
            TRAY_TAB_TEXT,
        );
    }

    fn draw_timers(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        grid: &EditGrid,
    ) {
        let (time_step, cycle, subcycle) = if let Some(eval) = grid.eval() {
            (eval.time_step(), eval.cycle(), eval.subcycle())
        } else {
            (0, 0, 0)
        };
        let timer_right = (self.rect.right() - TRAY_INNER_MARGIN) as f32;
        let mut timer_mid =
            self.rect.y + TRAY_INNER_MARGIN + BUTTON_HEIGHT / 2;
        resources.fonts().roman().draw(
            matrix,
            TIMER_FONT_SIZE,
            Align::MidRight,
            (timer_right, timer_mid as f32),
            &format!("Sub:{}", subcycle),
        );
        if self.show_cycle_count {
            timer_mid += BUTTON_HEIGHT + BUTTON_SPACING;
            resources.fonts().roman().draw(
                matrix,
                TIMER_FONT_SIZE,
                Align::MidRight,
                (timer_right, timer_mid as f32),
                &format!("Cycle:{}", cycle),
            );
        }
        timer_mid += BUTTON_HEIGHT + BUTTON_SPACING;
        resources.fonts().roman().draw(
            matrix,
            TIMER_FONT_SIZE,
            Align::MidRight,
            (timer_right, timer_mid as f32),
            &format!("Time:{}", time_step),
        );
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
        let rel_event =
            event.relative_to(Point2::new(-self.slide.distance(), 0));
        for button in self.buttons.iter_mut() {
            let opt_action = button.on_event(
                &rel_event,
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
        match &rel_event {
            Event::ClockTick(tick) => self.slide.on_clock_tick(tick, ui),
            Event::MouseDown(mouse) => {
                let tab_rect = UiShader::tray_tab_rect(
                    self.rect.as_f32(),
                    TRAY_TAB_HEIGHT,
                    TRAY_FLIP_HORZ,
                );
                if tab_rect.contains_point(mouse.pt.as_f32()) {
                    self.slide.toggle();
                    return Some(None);
                } else if self.rect.contains_point(mouse.pt) {
                    return Some(None);
                }
            }
            Event::MouseMove(mouse) | Event::MouseUp(mouse) => {
                let tab_rect = UiShader::tray_tab_rect(
                    self.rect.as_f32(),
                    TRAY_TAB_HEIGHT,
                    TRAY_FLIP_HORZ,
                );
                if self.rect.contains_point(mouse.pt)
                    || tab_rect.contains_point(mouse.pt.as_f32())
                {
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
    hover_pulse: HoverPulse,
}

impl ControlsButton {
    pub fn new(action: ControlsAction, rect: Rect<i32>) -> ControlsButton {
        ControlsButton { action, rect, hover_pulse: HoverPulse::new() }
    }

    fn is_enabled(&self, status: ControlsStatus) -> bool {
        match self.action {
            ControlsAction::FastForward => {
                status != ControlsStatus::Finished
                    && status != ControlsStatus::FastForwarding
            }
            ControlsAction::Reset => status != ControlsStatus::Stopped,
            ControlsAction::RunOrPause => status != ControlsStatus::Finished,
            _ => {
                status == ControlsStatus::Stopped
                    || status == ControlsStatus::Paused
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
                    && key.code
                        == prefs.hotkey_code(self.action.hotkey()).to_keycode()
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
