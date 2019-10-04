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
mod chipdrag;
mod control;
mod grid;
mod parts;
mod select;
mod specify;
mod tooltip;
mod tray;
mod tutorial;
mod verify;
mod wiredrag;

use self::control::{ControlsAction, ControlsStatus, ControlsTray};
use self::grid::{EditGridAction, EditGridView};
use self::parts::{PartsAction, PartsTray};
use self::specify::SpecificationTray;
use self::tutorial::TutorialBubble;
use self::verify::VerificationTray;
use super::dialog::{ButtonDialogBox, TextDialogBox};
use crate::mancer::gui::{Event, Keycode, Resources, Sound, Ui, Window};
use crate::mancer::save::Prefs;
use cgmath;
use std::u16;
use tachy::geom::{Coords, Direction, RectSize};
use tachy::save::{ChipType, SolutionData};
use tachy::state::{
    EditGrid, EvalResult, EvalScore, GridChange, PuzzleExt,
    TutorialBubblePosition,
};

//===========================================================================//

pub enum CircuitAction {
    BackToMenu,
    Victory(SolutionData),
}

#[derive(Clone, Copy)]
enum VictoryDialogAction {
    BackToMenu,
    ContinueEditing,
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
    edit_const_dialog: Option<(TextDialogBox, Coords)>,
    victory_dialog: Option<ButtonDialogBox<VictoryDialogAction>>,
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
        CircuitView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            edit_grid: EditGridView::new(
                window_size,
                grid.bounds(),
                bounds_bubbles,
            ),
            controls_tray: ControlsTray::new(
                window_size,
                puzzle,
                controls_bubble,
            ),
            parts_tray: PartsTray::new(
                window,
                grid.allowed_chips(),
                parts_bubble,
            ),
            specification_tray: SpecificationTray::new(
                window_size,
                puzzle,
                prefs,
            ),
            verification_tray: VerificationTray::new(window_size, puzzle),
            seconds_since_time_step: 0.0,
            controls_status: ControlsStatus::Stopped,
            edit_const_dialog: None,
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
            grid.has_errors(),
        );
        self.edit_grid.draw_dragged(resources);
        self.edit_grid.draw_tooltip(resources, &projection);
        if let Some((ref dialog, _)) = self.edit_const_dialog {
            dialog.draw(resources, &projection, is_valid_const);
        }
        if let Some(ref dialog) = self.victory_dialog {
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

        if let Some((mut dialog, coords)) = self.edit_const_dialog.take() {
            match dialog.on_event(event, ui, is_valid_const) {
                Some(Some(text)) => {
                    if let Ok(new_value) = text.parse::<u16>() {
                        change_const_chip_value(ui, grid, coords, new_value);
                    }
                }
                Some(None) => {}
                None => self.edit_const_dialog = Some((dialog, coords)),
            }
            return None;
        }

        if let Some(mut dialog) = self.victory_dialog.take() {
            match dialog.on_event(event, ui) {
                Some(VictoryDialogAction::BackToMenu) => {
                    return Some(CircuitAction::BackToMenu);
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
                    if self.controls_status == ControlsStatus::Running {
                        let seconds_per_time_step =
                            eval.seconds_per_time_step();
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
                    return Some(CircuitAction::BackToMenu);
                }
            }
            _ => {}
        }

        self.edit_grid.request_interaction_cursor(event, ui.cursor());

        if let Some(opt_action) = self.controls_tray.on_event(
            event,
            ui,
            self.controls_status,
            grid.has_errors(),
            prefs,
        ) {
            match opt_action {
                None => {}
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
                            ui.audio().play_sound(Sound::Beep);
                            self.seconds_since_time_step = 0.0;
                            self.controls_status = ControlsStatus::Running;
                            grid.start_eval();
                            ui.request_redraw();
                        }
                        ControlsStatus::Running => {
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

        let (opt_action, stop) =
            self.parts_tray.on_event(event, ui, grid.eval().is_none(), prefs);
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

        let stop = self.specification_tray.on_event(event, ui);
        if stop {
            return action;
        }

        let stop = self.verification_tray.on_event(event, ui);
        if stop {
            return action;
        }

        match self.edit_grid.on_event(event, ui, grid, prefs) {
            Some(EditGridAction::EditConst(coords, value)) => {
                let size =
                    RectSize::new(self.width as i32, self.height as i32);
                let dialog = TextDialogBox::new(
                    size,
                    prefs,
                    "Choose new const value:",
                    &value.to_string(),
                    u16::MAX.to_string().len(),
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
                None
            }
            EvalResult::Victory(score) => {
                let area = grid.bounds().area();
                let score = match score {
                    EvalScore::Value(value) => value as u32,
                    EvalScore::WireLength => {
                        grid.wire_fragments().len() as u32
                    }
                };
                // TODO: It would be nice to not have this unwrap() here.
                let time_steps = grid.eval().unwrap().time_step();
                debug_log!("Victory!  area={}, score={}", area, score);
                grid.stop_eval();
                let size =
                    RectSize::new(self.width as i32, self.height as i32);
                // TODO: The dialog box should show the optimization graph
                //   (with this point plotted on it).
                let format =
                    format!("Victory!\nArea: {}\nScore: {}", area, score);
                let buttons = &[
                    (
                        "Continue editing",
                        VictoryDialogAction::ContinueEditing,
                        Some(Keycode::Escape),
                    ),
                    (
                        "Back to menu",
                        VictoryDialogAction::BackToMenu,
                        Some(Keycode::Return),
                    ),
                ];
                self.victory_dialog =
                    Some(ButtonDialogBox::new(size, prefs, &format, buttons));
                // TODO: Unfocus other views
                self.controls_status = ControlsStatus::Stopped;
                ui.request_redraw();
                Some(CircuitAction::Victory(SolutionData {
                    install_id: prefs.install_id(),
                    puzzle: grid.puzzle(),
                    score,
                    time_steps,
                    circuit: grid.to_circuit_data(),
                }))
            }
            EvalResult::Failure => {
                debug_log!("Failure!");
                self.controls_status = ControlsStatus::Finished;
                ui.request_redraw();
                None
            }
        }
    }
}

fn is_valid_const(text: &str) -> bool {
    text.parse::<u16>().is_ok()
}

fn change_const_chip_value(
    ui: &mut Ui,
    grid: &mut EditGrid,
    coords: Coords,
    new_value: u16,
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
