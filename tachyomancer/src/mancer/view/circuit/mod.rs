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

mod bounds;
mod camera;
mod chipdrag;
mod control;
mod export;
mod grid;
mod manip;
mod parts;
mod select;
mod specify;
mod tooltip;
mod tray;
mod tutorial;
mod verify;
mod wiredrag;

use self::control::{ControlsAction, ControlsStatus, ControlsTray};
pub use self::export::export_circuit_image;
use self::grid::{EditGridAction, EditGridView};
use self::parts::{PartsAction, PartsTray};
use self::specify::SpecificationTray;
use self::tooltip::GridTooltipTag;
use self::tutorial::TutorialBubble;
use self::verify::VerificationTray;
use super::dialog::{
    ButtonDialogBox, DialogAction, HotkeyDialogBox, ScoreGraphDialogBox,
    TextDialogBox, WireSizeDialogBox,
};
use super::paragraph::Paragraph;
use super::tooltip::Tooltip;
use crate::mancer::gui::{Event, Keycode, Resources, Sound, Ui, Window};
use crate::mancer::save::Prefs;
use cgmath::{self, vec2, MetricSpace, Point2};
use std::u8;
use tachy::geom::{AsFloat, Coords, Direction, RectSize};
use tachy::save::{
    ChipType, HotkeyCode, Puzzle, ScoreCurve, SolutionData, WireSize,
    MAX_COMMENT_CHARS,
};
use tachy::state::{
    EditGrid, EvalResult, GridChange, PuzzleExt, TutorialBubblePosition,
};

//===========================================================================//

const FAST_FORWARD_SPEEDUP: f64 = 5.0;
const PARTS_CONTROLS_SPACING: i32 = 4;

//===========================================================================//

pub enum CircuitAction {
    BackToMenu,
    BackToMenuWithoutSaving,
    ExportImage(u32),
    Victory(SolutionData),
}

#[derive(Clone, Copy)]
enum ExportImageDialogAction {
    BackToMenu,
    ContinueEditing,
}

#[derive(Clone, Copy)]
enum FailedSaveDialogAction {
    BackToMenuWithoutSaving,
    ContinueEditing,
}

#[derive(Clone, Copy)]
enum VictoryDialogAction {
    BackToMenu,
    ExportImage(u32),
    ContinueEditing,
}

//===========================================================================//

#[derive(Eq, PartialEq)]
enum CircuitTooltipTag {
    Controls(ControlsAction),
    Grid(GridTooltipTag),
    Parts(ChipType),
    Unused(()),
}

impl CircuitTooltipTag {
    fn tooltip_format(&self, grid: &EditGrid) -> String {
        match self {
            CircuitTooltipTag::Controls(action) => {
                action.tooltip_format().to_string()
            }
            CircuitTooltipTag::Grid(tag) => tag.tooltip_format(grid),
            CircuitTooltipTag::Parts(ctype) => ctype.tooltip_format(),
            CircuitTooltipTag::Unused(()) => String::new(),
        }
    }
}

//===========================================================================//

pub struct CircuitView {
    width: f32,
    height: f32,
    edit_grid: EditGridView,
    controls_tray: ControlsTray,
    parts_tray: PartsTray,
    specification_tray: SpecificationTray,
    verification_tray: VerificationTray,
    seconds_since_time_step: f64,
    controls_status: ControlsStatus,
    tooltip: Tooltip<CircuitTooltipTag>,
    edit_button_dialog: Option<(HotkeyDialogBox, Coords)>,
    edit_coerce_dialog: Option<(WireSizeDialogBox, Coords)>,
    edit_comment_dialog: Option<(TextDialogBox, Coords)>,
    edit_const_dialog: Option<(TextDialogBox, Coords)>,
    export_image_dialog: Option<ButtonDialogBox<ExportImageDialogAction>>,
    failed_save_dialog: Option<ButtonDialogBox<FailedSaveDialogAction>>,
    victory_dialog: Option<ScoreGraphDialogBox<VictoryDialogAction>>,
}

impl CircuitView {
    pub fn new(
        window: &Window,
        grid: &EditGrid,
        prefs: &Prefs,
    ) -> CircuitView {
        let window_size = window.size();
        let puzzle = grid.puzzle();
        // TODO: Don't show any tutorial bubbles if puzzle is solved.
        let bubbles = puzzle.tutorial_bubbles();
        let bounds_bubbles: Vec<(Direction, TutorialBubble)> = bubbles
            .iter()
            .filter_map(|&(pos, format)| match pos {
                TutorialBubblePosition::Bounds(dir) => {
                    Some((dir, TutorialBubble::new(prefs, format)))
                }
                _ => None,
            })
            .collect();
        let controls_bubble = bubbles
            .iter()
            .find(|&&(pos, _)| pos == TutorialBubblePosition::ControlsTray)
            .map(|&(_, format)| TutorialBubble::new(prefs, format));
        let parts_bubble = bubbles
            .iter()
            .find(|&&(pos, _)| pos == TutorialBubblePosition::PartsTray)
            .map(|&(_, format)| TutorialBubble::new(prefs, format));
        let controls_tray =
            ControlsTray::new(window_size, puzzle, controls_bubble);
        let parts_tray = PartsTray::new(
            window,
            grid.allowed_chips(),
            controls_tray.rect().y - PARTS_CONTROLS_SPACING,
            parts_bubble,
        );
        CircuitView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            edit_grid: EditGridView::new(
                window_size,
                grid.bounds(),
                bounds_bubbles,
            ),
            controls_tray,
            parts_tray,
            specification_tray: SpecificationTray::new(
                window_size,
                puzzle,
                prefs,
            ),
            verification_tray: VerificationTray::new(window_size, puzzle),
            seconds_since_time_step: 0.0,
            controls_status: ControlsStatus::Stopped,
            tooltip: Tooltip::new(window_size),
            edit_button_dialog: None,
            edit_coerce_dialog: None,
            edit_comment_dialog: None,
            edit_const_dialog: None,
            export_image_dialog: None,
            failed_save_dialog: None,
            victory_dialog: None,
        }
    }

    pub fn draw(&self, resources: &Resources, grid: &EditGrid) {
        self.edit_grid.draw_board(resources, grid);
        let projection =
            cgmath::ortho(0.0, self.width, self.height, 0.0, -100.0, 100.0);
        self.verification_tray.draw(resources, &projection, grid.eval());
        self.specification_tray.draw(resources, &projection);
        self.parts_tray.draw(resources, &projection, grid.eval().is_none());
        self.controls_tray.draw(
            resources,
            &projection,
            self.controls_status,
            grid,
        );
        self.edit_grid.draw_dragged(resources);
        self.tooltip.draw(resources, &projection);
        if let Some((ref dialog, _)) = self.edit_button_dialog {
            dialog.draw(resources, &projection);
        } else if let Some((ref dialog, _)) = self.edit_coerce_dialog {
            dialog.draw(resources, &projection);
        } else if let Some((ref dialog, _)) = self.edit_comment_dialog {
            dialog.draw(resources, &projection, |_| true);
        } else if let Some((ref dialog, _)) = self.edit_const_dialog {
            dialog.draw(resources, &projection, is_valid_const);
        } else if let Some(ref dialog) = self.export_image_dialog {
            dialog.draw(resources, &projection);
        } else if let Some(ref dialog) = self.failed_save_dialog {
            dialog.draw(resources, &projection);
        } else if let Some(ref dialog) = self.victory_dialog {
            dialog.draw(resources, &projection);
        }
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        (grid, prefs): (&mut EditGrid, &Prefs),
    ) -> Option<CircuitAction> {
        debug_assert_eq!(
            self.controls_status == ControlsStatus::Stopped,
            grid.eval().is_none()
        );

        self.tooltip
            .on_event(event, ui, prefs, |tag| tag.tooltip_format(grid));

        if let Some((mut dialog, coords)) = self.edit_button_dialog.take() {
            match dialog.on_event(event, ui) {
                Some(DialogAction::Value(opt_code)) => {
                    change_button_chip_hotkey(ui, grid, coords, opt_code);
                }
                Some(DialogAction::Cancel) => {}
                None => self.edit_button_dialog = Some((dialog, coords)),
            }
            return None;
        }

        if let Some((mut dialog, coords)) = self.edit_coerce_dialog.take() {
            match dialog.on_event(event, ui) {
                Some(DialogAction::Value(size)) => {
                    change_coerce_chip_size(ui, grid, coords, size);
                }
                Some(DialogAction::Cancel) => {}
                None => self.edit_coerce_dialog = Some((dialog, coords)),
            }
            return None;
        }

        if let Some((mut dialog, coords)) = self.edit_comment_dialog.take() {
            match dialog.on_event(event, ui, |_| true) {
                Some(DialogAction::Value(text)) => {
                    change_comment_chip_value(ui, grid, coords, &text);
                }
                Some(DialogAction::Cancel) => {}
                None => self.edit_comment_dialog = Some((dialog, coords)),
            }
            return None;
        }

        if let Some((mut dialog, coords)) = self.edit_const_dialog.take() {
            match dialog.on_event(event, ui, is_valid_const) {
                Some(DialogAction::Value(text)) => {
                    if let Ok(new_value) = text.parse::<u8>() {
                        change_const_chip_value(ui, grid, coords, new_value);
                    }
                }
                Some(DialogAction::Cancel) => {}
                None => self.edit_const_dialog = Some((dialog, coords)),
            }
            return None;
        }

        if let Some(mut dialog) = self.export_image_dialog.take() {
            match dialog.on_event(event, ui) {
                Some(ExportImageDialogAction::BackToMenu) => {
                    return Some(CircuitAction::BackToMenu);
                }
                Some(ExportImageDialogAction::ContinueEditing) => {}
                None => self.export_image_dialog = Some(dialog),
            }
            return None;
        }

        if let Some(mut dialog) = self.failed_save_dialog.take() {
            match dialog.on_event(event, ui) {
                Some(FailedSaveDialogAction::BackToMenuWithoutSaving) => {
                    return Some(CircuitAction::BackToMenuWithoutSaving);
                }
                Some(FailedSaveDialogAction::ContinueEditing) => {}
                None => self.failed_save_dialog = Some(dialog),
            }
            return None;
        }

        if let Some(mut dialog) = self.victory_dialog.take() {
            match dialog.on_event(event, ui) {
                Some(VictoryDialogAction::BackToMenu) => {
                    return Some(CircuitAction::BackToMenu);
                }
                Some(VictoryDialogAction::ExportImage(score)) => {
                    return Some(CircuitAction::ExportImage(score));
                }
                Some(VictoryDialogAction::ContinueEditing) => {}
                None => self.victory_dialog = Some(dialog),
            }
            return None;
        }

        let mut action: Option<CircuitAction> = None;
        match event {
            Event::ClockTick(tick) => {
                let mut result = EvalResult::Continue;
                if let Some(eval) = grid.eval_mut() {
                    if self.controls_status == ControlsStatus::FastForwarding
                        || self.controls_status == ControlsStatus::Running
                    {
                        let mut seconds_per_time_step =
                            eval.seconds_per_time_step();
                        if self.controls_status
                            == ControlsStatus::FastForwarding
                        {
                            seconds_per_time_step /= FAST_FORWARD_SPEEDUP;
                        }
                        self.seconds_since_time_step += tick.elapsed;
                        while self.seconds_since_time_step
                            >= seconds_per_time_step
                        {
                            self.seconds_since_time_step -=
                                seconds_per_time_step;
                            result = eval.step_time();
                            ui.request_redraw();
                        }
                    }
                }
                action = self.on_eval_result(result, ui, grid, prefs);
            }
            Event::KeyDown(key) => {
                if key.code == Keycode::Escape {
                    if self.edit_grid.has_interaction() {
                        self.edit_grid.cancel_interaction(ui, grid);
                    } else {
                        return Some(CircuitAction::BackToMenu);
                    }
                }
            }
            _ => {}
        }

        self.edit_grid.request_interaction_cursor(event, ui.cursor());

        if let Some(opt_action) = self.controls_tray.on_event(
            event,
            ui,
            self.controls_status,
            grid,
            self.edit_grid.is_dragging(),
            &mut self.tooltip.sink(CircuitTooltipTag::Controls),
            prefs,
        ) {
            match opt_action {
                None => {}
                Some(ControlsAction::FastForward) => {
                    match self.controls_status {
                        ControlsStatus::Stopped => {
                            debug_assert!(grid.eval().is_none());
                            self.edit_grid.cancel_interaction(ui, grid);
                            ui.audio().play_sound(Sound::Beep);
                            self.seconds_since_time_step = 0.0;
                            self.controls_status =
                                ControlsStatus::FastForwarding;
                            grid.start_eval();
                            ui.request_redraw();
                        }
                        ControlsStatus::Running => {
                            debug_assert!(grid.eval().is_some());
                            self.controls_status =
                                ControlsStatus::FastForwarding;
                            ui.request_redraw();
                        }
                        ControlsStatus::Paused => {
                            debug_assert!(grid.eval().is_some());
                            self.seconds_since_time_step = 0.0;
                            self.controls_status =
                                ControlsStatus::FastForwarding;
                            ui.request_redraw();
                        }
                        ControlsStatus::FastForwarding
                        | ControlsStatus::Finished => {
                            debug_assert!(grid.eval().is_some());
                        }
                    }
                }
                Some(ControlsAction::GoToError) => {
                    self.move_camera_to_grid_error(ui, grid);
                }
                Some(ControlsAction::Reset) => {
                    if grid.eval().is_some() {
                        ui.audio().play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        self.controls_status = ControlsStatus::Stopped;
                        grid.stop_eval();
                        ui.request_redraw();
                    }
                }
                Some(ControlsAction::RunOrPause) => {
                    match self.controls_status {
                        ControlsStatus::Stopped => {
                            debug_assert!(grid.eval().is_none());
                            self.edit_grid.cancel_interaction(ui, grid);
                            ui.audio().play_sound(Sound::Beep);
                            self.seconds_since_time_step = 0.0;
                            self.controls_status = ControlsStatus::Running;
                            grid.start_eval();
                            ui.request_redraw();
                        }
                        ControlsStatus::Running
                        | ControlsStatus::FastForwarding => {
                            debug_assert!(grid.eval().is_some());
                            self.seconds_since_time_step = 0.0;
                            self.controls_status = ControlsStatus::Paused;
                            ui.request_redraw();
                        }
                        ControlsStatus::Paused => {
                            debug_assert!(grid.eval().is_some());
                            self.seconds_since_time_step = 0.0;
                            self.controls_status = ControlsStatus::Running;
                            ui.request_redraw();
                        }
                        ControlsStatus::Finished => {
                            debug_assert!(grid.eval().is_some());
                        }
                    }
                }
                Some(ControlsAction::StepTime) => {
                    if grid.eval().is_none() {
                        self.edit_grid.cancel_interaction(ui, grid);
                        ui.audio().play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        self.controls_status = ControlsStatus::Paused;
                        grid.start_eval();
                        ui.request_redraw();
                    }
                    let mut result = EvalResult::Continue;
                    if let Some(eval) = grid.eval_mut() {
                        result = eval.step_time();
                        ui.request_redraw();
                    }
                    action = self.on_eval_result(result, ui, grid, prefs);
                }
                Some(ControlsAction::StepCycle) => {
                    if grid.eval().is_none() {
                        self.edit_grid.cancel_interaction(ui, grid);
                        ui.audio().play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        self.controls_status = ControlsStatus::Paused;
                        grid.start_eval();
                        ui.request_redraw();
                    }
                    let mut result = EvalResult::Continue;
                    if let Some(eval) = grid.eval_mut() {
                        result = eval.step_cycle();
                        ui.request_redraw();
                    }
                    action = self.on_eval_result(result, ui, grid, prefs);
                }
                Some(ControlsAction::StepSubcycle) => {
                    if grid.eval().is_none() {
                        self.edit_grid.cancel_interaction(ui, grid);
                        ui.audio().play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        self.controls_status = ControlsStatus::Paused;
                        grid.start_eval();
                        ui.request_redraw();
                    }
                    let mut result = EvalResult::Continue;
                    if let Some(eval) = grid.eval_mut() {
                        result = eval.step_subcycle();
                        ui.request_redraw();
                    }
                    action = self.on_eval_result(result, ui, grid, prefs);
                }
            }
            return action;
        }

        let (opt_action, stop) = self.parts_tray.on_event(
            event,
            ui,
            grid.eval().is_none(),
            &mut self.tooltip.sink(CircuitTooltipTag::Parts),
        );
        match opt_action {
            Some(PartsAction::Grab(ctype, pt)) => {
                self.edit_grid.grab_from_parts_tray(pt, ui, ctype);
            }
            Some(PartsAction::Drop) => {
                self.edit_grid.drop_into_parts_tray(ui, grid);
            }
            None => {}
        }
        if stop {
            return action;
        }

        let stop = self.specification_tray.on_event(
            event,
            ui,
            &mut self.tooltip.sink(CircuitTooltipTag::Unused),
        );
        if stop {
            return action;
        }

        let stop = self.verification_tray.on_event(
            event,
            ui,
            &mut self.tooltip.sink(CircuitTooltipTag::Unused),
        );
        if stop {
            return action;
        }

        match self.edit_grid.on_event(
            event,
            ui,
            grid,
            &mut self.tooltip.sink(CircuitTooltipTag::Grid),
            prefs,
        ) {
            Some(EditGridAction::EditButton(coords, code)) => {
                let size =
                    RectSize::new(self.width as i32, self.height as i32);
                let dialog = HotkeyDialogBox::new(
                    size,
                    prefs,
                    "Choose a hotkey for this button:",
                    code,
                );
                self.edit_button_dialog = Some((dialog, coords));
                ui.request_redraw();
            }
            Some(EditGridAction::EditCoerce(coords, wire_size)) => {
                let size =
                    RectSize::new(self.width as i32, self.height as i32);
                let dialog = WireSizeDialogBox::new(
                    size,
                    prefs,
                    "Choose new wire size:",
                    wire_size,
                );
                self.edit_coerce_dialog = Some((dialog, coords));
                ui.request_redraw();
            }
            Some(EditGridAction::EditComment(coords, string)) => {
                let size =
                    RectSize::new(self.width as i32, self.height as i32);
                let dialog = TextDialogBox::new(
                    size,
                    prefs,
                    "Enter comment string:",
                    &string,
                    MAX_COMMENT_CHARS,
                );
                self.edit_comment_dialog = Some((dialog, coords));
                ui.request_redraw();
            }
            Some(EditGridAction::EditConst(coords, value)) => {
                let size =
                    RectSize::new(self.width as i32, self.height as i32);
                let dialog = TextDialogBox::new(
                    size,
                    prefs,
                    "Choose new const value:",
                    &value.to_string(),
                    u8::MAX.to_string().len(),
                );
                self.edit_const_dialog = Some((dialog, coords));
                ui.request_redraw();
            }
            None => {}
        }
        return action;
    }

    fn on_eval_result(
        &mut self,
        result: EvalResult,
        ui: &mut Ui,
        grid: &mut EditGrid,
        prefs: &Prefs,
    ) -> Option<CircuitAction> {
        match result {
            EvalResult::Continue => None,
            EvalResult::Breakpoint(coords_vec) => {
                debug_log!("Breakpoint: {:?}", coords_vec);
                self.seconds_since_time_step = 0.0;
                self.controls_status = ControlsStatus::Paused;
                ui.request_redraw();
                // TODO: If there are multiple simultaneous breakpoints, go to
                //   the one closest to the current camera center.
                if let Some(&coords) = coords_vec.first() {
                    let goal = coords.as_f32() + vec2(0.5, 0.5);
                    self.edit_grid.set_camera_goal(goal);
                }
                None
            }
            EvalResult::Victory(score) => {
                let bounds = grid.bounds();
                let (time_steps, inputs) = {
                    // TODO: It would be nice to not have this unwrap() here.
                    let eval = grid.eval().unwrap();
                    (eval.time_step(), eval.recorded_inputs(bounds.top_left()))
                };
                debug_log!("Victory: area={}, score={}", bounds.area(), score);
                grid.stop_eval();
                self.controls_status = ControlsStatus::Stopped;
                ui.request_redraw();
                Some(CircuitAction::Victory(SolutionData {
                    install_id: prefs.install_id(),
                    puzzle: grid.puzzle(),
                    score,
                    time_steps,
                    circuit: grid.to_circuit_data(),
                    inputs,
                }))
            }
            EvalResult::Failure => {
                debug_log!("Failure!");
                // TODO: If fatal port error, move camera to show port.
                if cfg!(debug_assertions) {
                    for error in grid.eval().unwrap().errors() {
                        debug_log!(
                            "Time step {}: {}",
                            error.time_step,
                            error.message
                        );
                    }
                }
                self.controls_status = ControlsStatus::Finished;
                ui.request_redraw();
                None
            }
        }
    }

    fn move_camera_to_grid_error(&mut self, ui: &mut Ui, grid: &mut EditGrid) {
        self.edit_grid.cancel_interaction(ui, grid);
        let center = self.edit_grid.camera_center();
        let mut nearest: Option<Point2<f32>> = None;
        let mut best_dist = f32::INFINITY;
        for error in grid.errors() {
            for wire_id in error.wire_ids() {
                for ((coords, dir), _) in
                    grid.wire_fragments_for_wire_id(wire_id)
                {
                    let point = coords.as_f32()
                        + vec2(0.5, 0.5)
                        + dir.delta().as_f32() * 0.5;
                    let dist = point.distance2(center);
                    if dist < best_dist {
                        nearest = Some(point);
                        best_dist = dist;
                    }
                }
            }
        }
        if let Some(goal) = nearest {
            self.edit_grid.set_camera_goal(goal);
        }
    }

    pub fn show_export_image_success(
        &mut self,
        ui: &mut Ui,
        prefs: &Prefs,
        path: &str,
    ) {
        let format = format!("Saved image to:\n\n{}", Paragraph::escape(path));
        self.show_export_image_dialog(ui, prefs, &format);
    }

    pub fn show_export_image_error(
        &mut self,
        ui: &mut Ui,
        prefs: &Prefs,
        error: &str,
    ) {
        debug_warn!("Failed to export image: {}", error);
        // TODO: Play sound for error dialog popup.
        let format = format!(
            "$R$*ERROR:$*$D Unable to export image!\n\n{}",
            Paragraph::escape(error)
        );
        self.show_export_image_dialog(ui, prefs, &format);
    }

    fn show_export_image_dialog(
        &mut self,
        ui: &mut Ui,
        prefs: &Prefs,
        format: &str,
    ) {
        let buttons = &[
            (
                "Continue editing",
                ExportImageDialogAction::ContinueEditing,
                Some(Keycode::Escape),
            ),
            (
                "Back to menu",
                ExportImageDialogAction::BackToMenu,
                Some(Keycode::Return),
            ),
        ];
        let size = RectSize::new(self.width as i32, self.height as i32);
        self.export_image_dialog =
            Some(ButtonDialogBox::new(size, prefs, format, buttons));
        ui.request_redraw();
        // TODO: Unfocus other views
    }

    pub fn show_failed_to_save_error(
        &mut self,
        ui: &mut Ui,
        prefs: &Prefs,
        error: &str,
    ) {
        debug_warn!("Failed to save: {}", error);
        // TODO: Play sound for error dialog popup.
        let size = RectSize::new(self.width as i32, self.height as i32);
        let format = format!(
            "$R$*ERROR:$*$D Unable to save!\n\n{}",
            Paragraph::escape(error)
        );
        let buttons = &[
            (
                "Continue editing",
                FailedSaveDialogAction::ContinueEditing,
                Some(Keycode::Escape),
            ),
            (
                "Discard changes",
                FailedSaveDialogAction::BackToMenuWithoutSaving,
                None,
            ),
        ];
        self.failed_save_dialog =
            Some(ButtonDialogBox::new(size, prefs, &format, buttons));
        ui.request_redraw();
        // TODO: Unfocus other views
    }

    pub fn show_victory_dialog(
        &mut self,
        ui: &mut Ui,
        prefs: &Prefs,
        puzzle: Puzzle,
        area: i32,
        score: u32,
        local_scores: &ScoreCurve,
    ) {
        // TODO: Play sound for victory.
        let window_size = RectSize::new(self.width as i32, self.height as i32);
        let format = format!("Task \"{}\" completed!", puzzle.title());
        let buttons = &[
            (
                "Continue editing",
                VictoryDialogAction::ContinueEditing,
                Some(Keycode::Escape),
            ),
            ("Export image", VictoryDialogAction::ExportImage(score), None),
            (
                "Back to menu",
                VictoryDialogAction::BackToMenu,
                Some(Keycode::Return),
            ),
        ];
        self.victory_dialog = Some(ScoreGraphDialogBox::new(
            window_size,
            prefs,
            &format,
            puzzle,
            local_scores,
            (area, score),
            buttons,
        ));
        ui.request_redraw();
        // TODO: Unfocus other views
    }
}

fn is_valid_const(text: &str) -> bool {
    text.parse::<u8>().is_ok()
}

fn change_button_chip_hotkey(
    ui: &mut Ui,
    grid: &mut EditGrid,
    coords: Coords,
    new_code: Option<HotkeyCode>,
) {
    if let Some((coords, ChipType::Button(old_code), orient)) =
        grid.chip_at(coords)
    {
        let changes = vec![
            GridChange::RemoveChip(coords, ChipType::Button(old_code), orient),
            GridChange::AddChip(coords, ChipType::Button(new_code), orient),
        ];
        if grid.try_mutate(changes) {
            ui.request_redraw();
        } else {
            debug_warn!("change_button_chip_hotkey mutation failed");
        }
    }
}

fn change_coerce_chip_size(
    ui: &mut Ui,
    grid: &mut EditGrid,
    coords: Coords,
    new_size: WireSize,
) {
    if let Some((coords, ChipType::Coerce(old_size), orient)) =
        grid.chip_at(coords)
    {
        let changes = vec![
            GridChange::RemoveChip(coords, ChipType::Coerce(old_size), orient),
            GridChange::AddChip(coords, ChipType::Coerce(new_size), orient),
        ];
        if grid.try_mutate(changes) {
            ui.request_redraw();
        } else {
            debug_warn!("change_coerce_chip_size mutation failed");
        }
    }
}

fn change_comment_chip_value(
    ui: &mut Ui,
    grid: &mut EditGrid,
    coords: Coords,
    new_string: &str,
) {
    if let Some((coords, ChipType::Comment(old_bytes), orient)) =
        grid.chip_at(coords)
    {
        let mut new_bytes = [b' '; MAX_COMMENT_CHARS];
        for (index, byte) in new_string
            .chars()
            .map(|chr| chr as u8)
            .take(MAX_COMMENT_CHARS)
            .enumerate()
        {
            new_bytes[index] = byte;
        }
        let changes = vec![
            GridChange::RemoveChip(
                coords,
                ChipType::Comment(old_bytes),
                orient,
            ),
            GridChange::AddChip(coords, ChipType::Comment(new_bytes), orient),
        ];
        if grid.try_mutate(changes) {
            ui.request_redraw();
        } else {
            debug_warn!("change_comment_chip_value mutation failed");
        }
    }
}

fn change_const_chip_value(
    ui: &mut Ui,
    grid: &mut EditGrid,
    coords: Coords,
    new_value: u8,
) {
    if let Some((coords, ChipType::Const(old_value), orient)) =
        grid.chip_at(coords)
    {
        let changes = vec![
            GridChange::RemoveChip(coords, ChipType::Const(old_value), orient),
            GridChange::AddChip(coords, ChipType::Const(new_value), orient),
        ];
        if grid.try_mutate(changes) {
            ui.request_redraw();
        } else {
            debug_warn!("change_const_chip_value mutation failed");
        }
    }
}

//===========================================================================//
