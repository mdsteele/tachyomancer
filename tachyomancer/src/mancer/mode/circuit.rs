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

use super::shared::ModeChange;
use crate::mancer::gui::{Event, Music, Window};
use crate::mancer::state::GameState;
use crate::mancer::view::{export_circuit_image, CircuitAction, CircuitView};
use directories::UserDirs;
use png::{self, HasParameters};
use std::fs::File;
use std::path::PathBuf;
use tachy::geom::RectSize;
use tachy::save::{Puzzle, SolutionData};

//===========================================================================//

pub fn run(state: &mut GameState, window: &mut Window) -> ModeChange {
    debug_assert!(state.profile().is_some());
    debug_assert!(state.edit_grid().is_some());
    window
        .ui()
        .audio()
        .play_music(music_for_puzzle(state.edit_grid().unwrap().puzzle()));
    let mut view = {
        let grid = state.edit_grid().unwrap();
        CircuitView::new(window, grid, state.prefs())
    };
    loop {
        match window.next_event() {
            Event::Quit => return ModeChange::Quit,
            Event::Redraw => {
                window.pump_audio();
                view.draw(window.resources(), state.edit_grid().unwrap());
                window.pump_video();
            }
            event => {
                match view.on_event(
                    &event,
                    &mut window.ui(),
                    state.edit_grid_mut_and_prefs().unwrap(),
                ) {
                    Some(CircuitAction::BackToMenu) => match state.save() {
                        Ok(()) => return back_to_menu(state),
                        Err(err) => {
                            view.show_failed_to_save_error(
                                &mut window.ui(),
                                state.prefs(),
                                &err,
                            );
                        }
                    },
                    Some(CircuitAction::BackToMenuWithoutSaving) => {
                        return back_to_menu(state);
                    }
                    Some(CircuitAction::ExportImage(score)) => {
                        export_image(window, &mut view, state, score);
                    }
                    Some(CircuitAction::Victory(solution)) => {
                        record_score(window, &mut view, state, solution);
                    }
                    None => {}
                }
                window.pump_cursor();
                state.maybe_autosave_circuit();
            }
        }
    }
}

fn music_for_puzzle(puzzle: Puzzle) -> Vec<Music> {
    match puzzle {
        Puzzle::AutomateBeacon => {
            vec![Music::PitchBlack, Music::BeyondTheStars]
        }
        Puzzle::AutomateMiningRobot => {
            vec![Music::MorningCruise, Music::EcstaticWave]
        }
        Puzzle::AutomateReactor => {
            vec![Music::AfterlifeCity, Music::InfectedEuphoria]
        }
        Puzzle::AutomateRobotArm => {
            vec![Music::InfectedEuphoria, Music::FireWithin]
        }
        Puzzle::AutomateStorageDepot => {
            vec![Music::AfterlifeCity, Music::FireWithin]
        }
        Puzzle::AutomateTranslator => {
            vec![Music::TheHyperboreanMenace, Music::SettingSail]
        }
        Puzzle::AutomateXUnit => {
            vec![Music::DerelictShip, Music::BeyondTheStars]
        }
        Puzzle::CommandLander => {
            vec![Music::BeyondTheStars, Music::SettingSail]
        }
        Puzzle::CommandShields => {
            vec![Music::LockAndLoad, Music::InfectedEuphoria]
        }
        Puzzle::CommandTurret => {
            vec![Music::LockAndLoad, Music::InfectedEuphoria]
        }
        Puzzle::SandboxBehavior | Puzzle::SandboxEvent => {
            vec![Music::MorningCruise, Music::InfectedEuphoria]
        }
        // TODO: specify music for other puzzles
        _ => vec![Music::EcstaticWave],
    }
}

fn back_to_menu(state: &mut GameState) -> ModeChange {
    state.clear_edit_grid();
    if !state.has_circuit_name(state.circuit_name()) {
        state.set_circuit_name(String::new());
    }
    ModeChange::Next
}

fn export_image(
    window: &mut Window,
    view: &mut CircuitView,
    state: &GameState,
    score: u32,
) {
    debug_assert!(state.edit_grid().is_some());
    let grid = state.edit_grid().unwrap();
    let name = format!("{} {}", grid.puzzle().title(), state.circuit_name());
    let (size, rgb) = export_circuit_image(window.resources(), grid, score);
    match save_png(&name, size, &rgb) {
        Ok(path) => {
            view.show_export_image_success(
                &mut window.ui(),
                state.prefs(),
                &path.to_string_lossy(),
            );
        }
        Err(error) => {
            view.show_export_image_error(
                &mut window.ui(),
                state.prefs(),
                &error,
            );
        }
    }
}

fn is_alphanum_or_period(ch: char) -> bool {
    ch == '.' || ch.is_ascii_alphanumeric()
}

fn save_png(
    name: &str,
    size: RectSize<usize>,
    rgb: &[u8],
) -> Result<PathBuf, String> {
    let user_dirs = UserDirs::new()
        .ok_or_else(|| "No valid home directory found.".to_string())?;
    let downloads_dir = user_dirs
        .download_dir()
        .ok_or_else(|| "No valid downloads directory found.".to_string())?;
    let name = name.replace(|ch| !is_alphanum_or_period(ch), "_");
    let mut png_path = downloads_dir.join(format!("{}.png", name));
    let mut counter: u64 = 0;
    while png_path.exists() {
        counter += 1;
        png_path = downloads_dir.join(format!("{}_{}.png", name, counter));
    }
    let png_file = File::create(&png_path).map_err(|err| err.to_string())?;
    let mut encoder =
        png::Encoder::new(png_file, size.width as u32, size.height as u32);
    encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|err| err.to_string())?;
    writer.write_image_data(rgb).map_err(|err| err.to_string())?;
    Ok(png_path)
}

fn record_score(
    window: &mut Window,
    view: &mut CircuitView,
    state: &mut GameState,
    solution: SolutionData,
) {
    let puzzle = solution.puzzle;
    let area = solution.circuit.size.area();
    let score = solution.score;
    window.submit_solution(solution);
    match state.record_puzzle_score(puzzle, area, score) {
        Ok(()) => {
            view.show_victory_dialog(
                &mut window.ui(),
                state.prefs(),
                puzzle,
                area,
                score,
                state.local_scores(puzzle),
            );
        }
        Err(err) => {
            // TODO: display error to user; don't panic
            panic!("Victory failed: {:?}", err);
        }
    }
}

//===========================================================================//
