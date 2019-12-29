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
use crate::mancer::view::{CircuitAction, CircuitView};
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
                    Some(CircuitAction::Victory(solution)) => {
                        record_score(state, window, solution);
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

fn record_score(
    state: &mut GameState,
    window: &Window,
    solution: SolutionData,
) {
    let score = solution.score;
    let area = solution.circuit.size.area();
    window.submit_solution(solution);
    match state.record_current_puzzle_score(area, score) {
        Ok(()) => {}
        Err(err) => {
            // TODO: display error to user; don't panic
            panic!("Victory failed: {:?}", err);
        }
    }
}

//===========================================================================//
