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
mod tray;
mod verify;
mod wiredrag;

use self::control::{ControlsAction, ControlsStatus, ControlsTray};
use self::grid::{EditGridAction, EditGridView};
use self::parts::{PartsAction, PartsTray};
use self::specify::SpecificationTray;
use self::verify::VerificationTray;
use super::dialog::{ButtonDialogBox, TextDialogBox};
use cgmath;
use std::u32;
use tachy::geom::{Coords, RectSize};
use tachy::gui::{Event, Keycode, Resources, Sound, Ui};
use tachy::save::{ChipType, Prefs, Puzzle};
use tachy::state::{EditGrid, EvalResult, EvalScore, GridChange};

//===========================================================================//

const SECONDS_PER_TIME_STEP: f64 = 0.1;

//===========================================================================//

#[derive(Clone, Copy, Debug)]
pub enum CircuitAction {
    BackToMenu,
    Victory(i32, i32),
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
    victory_dialog: Option<ButtonDialogBox<Option<CircuitAction>>>,
}

impl CircuitView {
    pub fn new(window_size: RectSize<i32>, current_puzzle: Puzzle,
               prefs: &Prefs)
               -> CircuitView {
        CircuitView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            edit_grid: EditGridView::new(window_size),
            controls_tray: ControlsTray::new(window_size, current_puzzle),
            parts_tray: PartsTray::new(window_size, current_puzzle),
            specification_tray: SpecificationTray::new(window_size,
                                                       current_puzzle,
                                                       prefs),
            verification_tray: VerificationTray::new(window_size,
                                                     current_puzzle),
            seconds_since_time_step: 0.0,
            controls_status: ControlsStatus::Stopped,
            edit_const_dialog: None,
            victory_dialog: None,
        }
    }

    pub fn draw(&self, resources: &Resources, grid: &EditGrid) {
        self.edit_grid.draw_board(resources, grid);
        let projection =
            cgmath::ortho(0.0, self.width, self.height, 0.0, -1.0, 1.0);
        self.verification_tray.draw(resources, &projection, grid.eval());
        self.specification_tray.draw(resources, &projection);
        self.parts_tray.draw(resources, &projection);
        self.controls_tray.draw(resources, &projection, self.controls_status);
        self.edit_grid.draw_dragged(resources);
        self.edit_grid.draw_tooltip(resources, &projection);
        if let Some((ref dialog, _)) = self.edit_const_dialog {
            dialog.draw(resources, &projection, is_valid_const);
        }
        if let Some(ref dialog) = self.victory_dialog {
            dialog.draw(resources, &projection);
        }
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui,
                    (grid, prefs): (&mut EditGrid, &Prefs))
                    -> Option<CircuitAction> {
        debug_assert_eq!(self.controls_status == ControlsStatus::Stopped,
                         grid.eval().is_none());

        if let Some((mut dialog, coords)) = self.edit_const_dialog.take() {
            match dialog.on_event(event, ui, is_valid_const) {
                Some(Some(text)) => {
                    if let Ok(new_value) = text.parse::<u32>() {
                        change_const_chip_value(grid, coords, new_value);
                    }
                }
                Some(None) => {}
                None => self.edit_const_dialog = Some((dialog, coords)),
            }
            return None;
        }

        if let Some(mut dialog) = self.victory_dialog.take() {
            match dialog.on_event(event, ui) {
                Some(Some(action)) => return Some(action),
                Some(None) => {}
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
                        self.seconds_since_time_step += tick.elapsed;
                        while self.seconds_since_time_step >=
                            SECONDS_PER_TIME_STEP
                        {
                            self.seconds_since_time_step -=
                                SECONDS_PER_TIME_STEP;
                            result = eval.step_time();
                        }
                    }
                }
                action = self.on_eval_result(result, grid, prefs);
            }
            Event::KeyDown(key) => {
                if key.code == Keycode::Escape {
                    return Some(CircuitAction::BackToMenu);
                }
            }
            _ => {}
        }

        self.edit_grid.request_interaction_cursor(event, ui.cursor());

        if let Some(opt_action) =
            self.controls_tray.on_event(event, ui, self.controls_status, prefs)
        {
            match opt_action {
                None => {}
                Some(ControlsAction::Reset) => {
                    if grid.eval().is_some() {
                        ui.audio().play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        self.controls_status = ControlsStatus::Stopped;
                        grid.stop_eval();
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
                        }
                        ControlsStatus::Running => {
                            debug_assert!(grid.eval().is_some());
                            self.seconds_since_time_step = 0.0;
                            self.controls_status = ControlsStatus::Paused;
                        }
                        ControlsStatus::Paused => {
                            debug_assert!(grid.eval().is_some());
                            self.seconds_since_time_step = 0.0;
                            self.controls_status = ControlsStatus::Running;
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
                    }
                    let mut result = EvalResult::Continue;
                    if let Some(eval) = grid.eval_mut() {
                        result = eval.step_time();
                    }
                    action = self.on_eval_result(result, grid, prefs);
                }
                Some(ControlsAction::StepCycle) => {
                    if grid.eval().is_none() {
                        ui.audio().play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        self.controls_status = ControlsStatus::Paused;
                        grid.start_eval();
                    }
                    let mut result = EvalResult::Continue;
                    if let Some(eval) = grid.eval_mut() {
                        result = eval.step_cycle();
                    }
                    action = self.on_eval_result(result, grid, prefs);
                }
                Some(ControlsAction::StepSubcycle) => {
                    if grid.eval().is_none() {
                        ui.audio().play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        self.controls_status = ControlsStatus::Paused;
                        grid.start_eval();
                    }
                    let mut result = EvalResult::Continue;
                    if let Some(eval) = grid.eval_mut() {
                        result = eval.step_subcycle();
                    }
                    action = self.on_eval_result(result, grid, prefs);
                }
            }
            return action;
        }

        let (opt_action, stop) = self.parts_tray.on_event(event);
        match opt_action {
            Some(PartsAction::Grab(ctype, pt)) => {
                self.edit_grid.grab_from_parts_tray(ctype, pt);
                ui.audio().play_sound(Sound::GrabChip);
            }
            Some(PartsAction::Drop) => {
                self.edit_grid.drop_into_parts_tray(grid);
                // TODO: Only play sound if we were actually holding a chip
                ui.audio().play_sound(Sound::DropChip);
            }
            None => {}
        }
        if stop {
            return action;
        }

        let stop = self.specification_tray.on_event(event);
        if stop {
            return action;
        }

        let stop = self.verification_tray.on_event(event);
        if stop {
            return action;
        }

        match self.edit_grid.on_event(event, ui, grid, prefs) {
            Some(EditGridAction::EditConst(coords, value)) => {
                let size = RectSize::new(self.width as i32,
                                         self.height as i32);
                let dialog = TextDialogBox::new(size,
                                                prefs,
                                                "Choose new const value:",
                                                &value.to_string(),
                                                u32::MAX.to_string().len());
                self.edit_const_dialog = Some((dialog, coords));
            }
            None => {}
        }
        return action;
    }

    fn on_eval_result(&mut self, result: EvalResult, grid: &mut EditGrid,
                      prefs: &Prefs)
                      -> Option<CircuitAction> {
        match result {
            EvalResult::Continue => None,
            EvalResult::Breakpoint(coords_vec) => {
                debug_log!("Breakpoint: {:?}", coords_vec);
                self.seconds_since_time_step = 0.0;
                self.controls_status = ControlsStatus::Paused;
                None
            }
            EvalResult::Victory(score) => {
                let area = grid.bounds().area();
                let score = match score {
                    EvalScore::Value(value) => value,
                    EvalScore::WireLength => {
                        grid.wire_fragments().len() as i32
                    }
                };
                debug_log!("Victory!  area={}, score={}", area, score);
                grid.stop_eval();
                let size = RectSize::new(self.width as i32,
                                         self.height as i32);
                // TODO: The dialog box should show the optimization graph
                //   (with this point plotted on it).
                let format =
                    format!("Victory!\nArea: {}\nScore: {}", area, score);
                let buttons =
                    &[
                        ("Continue editing", None, Some(Keycode::Escape)),
                        ("Back to menu",
                         Some(CircuitAction::BackToMenu),
                         Some(Keycode::Return)),
                    ];
                self.victory_dialog =
                    Some(ButtonDialogBox::new(size, prefs, &format, buttons));
                // TODO: Unfocus other views
                self.controls_status = ControlsStatus::Stopped;
                Some(CircuitAction::Victory(area, score))
            }
            EvalResult::Failure => {
                debug_log!("Failure!");
                self.controls_status = ControlsStatus::Finished;
                None
            }
        }
    }
}

fn is_valid_const(text: &str) -> bool { text.parse::<u32>().is_ok() }

fn change_const_chip_value(grid: &mut EditGrid, coords: Coords,
                           new_value: u32) {
    if let Some((coords, ChipType::Const(old_value), orient)) =
        grid.chip_at(coords)
    {
        let changes = vec![
            GridChange::RemoveChip(coords,
                                   ChipType::Const(old_value),
                                   orient),
            GridChange::AddChip(coords,
                                ChipType::Const(new_value),
                                orient),
        ];
        if !grid.try_mutate(changes) {
            debug_log!("WARNING: change_const_chip_value mutation failed");
        }
    }
}

//===========================================================================//
