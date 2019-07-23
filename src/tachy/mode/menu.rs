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
use tachy::gui::{Event, Window};
use tachy::state::GameState;
use tachy::view::{MenuAction, MenuView};

//===========================================================================//

pub fn run(state: &mut GameState, window: &mut Window) -> ModeChange {
    debug_assert!(state.profile().is_some());
    let mut view = MenuView::new(window, state);
    loop {
        match window.next_event() {
            Event::Quit => return ModeChange::Quit,
            Event::Redraw => {
                window.pump_audio();
                view.draw(window.resources(), state);
                window.pump_video();
            }
            event => {
                match view.on_event(&event, &mut window.ui(), state) {
                    Some(MenuAction::GoToPuzzle(puzzle)) => {
                        match state.unlock_puzzle(puzzle) {
                            Ok(()) => {
                                let mut ui = window.ui();
                                view.update_puzzle_list(&mut ui, state);
                                state.set_current_puzzle(puzzle);
                                view.go_to_current_puzzle(&mut ui, state);
                            }
                            Err(err) => {
                                view.show_error(&mut window.ui(),
                                                state,
                                                "unlock puzzle",
                                                &err);
                            }
                        }
                    }
                    Some(MenuAction::PlayCutscene(cutscene)) => {
                        state.set_cutscene(cutscene.script());
                        return ModeChange::Next;
                    }
                    Some(MenuAction::GoToConversation(conv)) => {
                        state.set_current_conversation(conv);
                        view.go_to_current_conversation(&mut window.ui(),
                                                        state);
                    }
                    Some(MenuAction::CopyCircuit) => {
                        match state.copy_current_circuit() {
                            Ok(()) => {
                                view.update_circuit_list(&mut window.ui(),
                                                         state);
                            }
                            Err(err) => {
                                view.show_error(&mut window.ui(),
                                                state,
                                                "copy circuit",
                                                &err);
                            }
                        }
                    }
                    Some(MenuAction::DeleteCircuit) => {
                        match state.delete_current_circuit() {
                            Ok(()) => {
                                view.update_circuit_list(&mut window.ui(),
                                                         state);
                            }
                            Err(err) => {
                                view.show_error(&mut window.ui(),
                                                state,
                                                "delete circuit",
                                                &err);
                            }
                        }
                    }
                    Some(MenuAction::EditCircuit) => {
                        match state.load_edit_grid() {
                            Ok(()) => return ModeChange::Next,
                            Err(err) => {
                                view.show_error(&mut window.ui(),
                                                state,
                                                "load circuit",
                                                &err);
                            }
                        }
                    }
                    Some(MenuAction::RenameCircuit(name)) => {
                        match state.rename_current_circuit(&name) {
                            Ok(()) => {
                                view.update_circuit_list(&mut window.ui(),
                                                         state);
                            }
                            Err(err) => {
                                view.show_error(&mut window.ui(),
                                                state,
                                                "rename circuit",
                                                &err);
                            }
                        }
                    }
                    Some(MenuAction::RebootWindow(options)) => {
                        return ModeChange::RebootWindow(options);
                    }
                    Some(MenuAction::NewProfile) => {
                        debug_log!("Starting a new profile");
                        match state.clear_profile() {
                            Ok(()) => return ModeChange::Next,
                            Err(err) => {
                                view.show_error(&mut window.ui(),
                                                state,
                                                "switch profile",
                                                &err);
                            }
                        }
                    }
                    Some(MenuAction::SwitchProfile(name)) => {
                        debug_log!("Switching to profile {:?}", name);
                        match state.create_or_load_profile(name) {
                            Ok(()) => {
                                let mut ui = window.ui();
                                view.update_conversation(&mut ui, state);
                                view.update_puzzle_list(&mut ui, state);
                                view.update_circuit_list(&mut ui, state);
                            }
                            Err(err) => {
                                view.show_error(&mut window.ui(),
                                                state,
                                                "switch profile",
                                                &err);
                            }
                        }
                    }
                    Some(MenuAction::DeleteProfile) => {
                        match state.delete_current_profile() {
                            Ok(()) => {
                                if state.profile().is_none() {
                                    return ModeChange::Next;
                                } else {
                                    let mut ui = window.ui();
                                    view.update_profile_list(&mut ui, state);
                                    view.update_conversation(&mut ui, state);
                                    view.update_puzzle_list(&mut ui, state);
                                    view.update_circuit_list(&mut ui, state);
                                }
                            }
                            Err(err) => {
                                view.show_error(&mut window.ui(),
                                                state,
                                                "delete profile",
                                                &err);
                            }
                        }
                    }
                    Some(MenuAction::QuitGame) => return ModeChange::Quit,
                    None => {}
                }
                window.pump_cursor();
            }
        }
    }
}

//===========================================================================//
