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

use super::edit::EditGrid;
use tachy::save::{CIRCUIT_NAME_MAX_WIDTH, Conversation, MenuSection, Prefs,
                  Profile, ProfileNamesIter, Puzzle, SaveDir};
use unicase;
use unicode_width::UnicodeWidthStr;

//===========================================================================//

pub struct GameState {
    savedir: SaveDir,
    menu_section: MenuSection,
    profile: Option<Profile>,
    circuit_name: String,
    edit_grid: Option<EditGrid>,
}

impl GameState {
    pub fn new(savedir: SaveDir) -> Result<GameState, String> {
        let opt_profile = savedir.load_current_profile_if_any()?;
        let menu_section = MenuSection::Puzzles;
        let mut circuit_name = String::new();
        if let Some(ref profile) = opt_profile {
            let puzzle = profile.current_puzzle();
            if let Some(name) = profile.circuit_names(puzzle).next() {
                circuit_name = name.to_string();
            }
        }
        let state = GameState {
            savedir,
            menu_section,
            profile: opt_profile,
            circuit_name,
            edit_grid: None,
        };
        Ok(state)
    }

    pub fn save(&mut self) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            if let Some(ref mut grid) = self.edit_grid {
                if grid.is_modified() {
                    let puzzle = profile.current_puzzle();
                    let circuit_data = grid.to_circuit_data();
                    profile
                        .save_circuit(puzzle,
                                      &self.circuit_name,
                                      &circuit_data)?;
                    grid.mark_unmodified();
                }
            }
            profile.save()?;
        }
        self.savedir.save()?;
        Ok(())
    }

    pub fn prefs(&self) -> &Prefs { self.savedir.prefs() }

    pub fn prefs_mut(&mut self) -> &mut Prefs { self.savedir.prefs_mut() }

    pub fn profile_names(&self) -> ProfileNamesIter {
        self.savedir.profile_names()
    }

    pub fn has_profile(&self, name: &str) -> bool {
        self.savedir.has_profile(name)
    }

    pub fn profile(&self) -> Option<&Profile> { self.profile.as_ref() }

    pub fn create_or_load_profile(&mut self, name: String)
                                  -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            profile.save()?;
        }
        let profile = self.savedir.create_or_load_profile(name)?;
        self.circuit_name = profile
            .first_circuit_name_for_current_puzzle()
            .unwrap_or_else(String::new);
        self.profile = Some(profile);
        Ok(())
    }

    pub fn clear_profile(&mut self) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            profile.save()?;
        }
        self.profile = None;
        Ok(())
    }

    pub fn menu_section(&self) -> MenuSection { self.menu_section }

    pub fn set_menu_section(&mut self, section: MenuSection) {
        self.menu_section = section;
    }

    pub fn current_conversation(&self) -> Conversation {
        if let Some(ref profile) = self.profile {
            profile.current_conversation()
        } else {
            Conversation::first()
        }
    }

    pub fn set_current_conversation(&mut self, conv: Conversation) {
        if let Some(ref mut profile) = self.profile {
            profile.set_current_conversation(conv);
        }
    }

    pub fn increment_current_conversation_progress(&mut self) {
        if let Some(ref mut profile) = self.profile {
            let conv = profile.current_conversation();
            profile.increment_conversation_progress(conv);
        }
    }

    pub fn is_conversation_unlocked(&self, conv: Conversation) -> bool {
        match self.profile.as_ref() {
            Some(profile) => profile.is_conversation_unlocked(conv),
            None => conv == Conversation::first(),
        }
    }

    pub fn is_conversation_complete(&self, conv: Conversation) -> bool {
        self.profile
            .as_ref()
            .map_or(false, |profile| profile.is_conversation_complete(conv))
    }

    pub fn mark_current_conversation_complete(&mut self) {
        if let Some(ref mut profile) = self.profile {
            let conv = profile.current_conversation();
            profile.mark_conversation_complete(conv);
        }
    }

    pub fn set_current_conversation_choice(&mut self, key: String,
                                           value: String) {
        if let Some(ref mut profile) = self.profile {
            let conv = profile.current_conversation();
            debug_log!("Making conversation {:?} choice {:?} = {:?}",
                       conv,
                       key,
                       value);
            profile.set_conversation_choice(conv, key, value);
        }
    }

    pub fn current_puzzle(&self) -> Puzzle {
        if let Some(ref profile) = self.profile {
            profile.current_puzzle()
        } else {
            Puzzle::first()
        }
    }

    pub fn set_current_puzzle(&mut self, puzzle: Puzzle) {
        if let Some(ref mut profile) = self.profile {
            if profile.current_puzzle() != puzzle {
                profile.set_current_puzzle(puzzle);
                self.circuit_name = profile
                    .first_circuit_name_for_current_puzzle()
                    .unwrap_or_else(String::new);
            }
        }
    }

    pub fn is_puzzle_unlocked(&self, puzzle: Puzzle) -> bool {
        match self.profile.as_ref() {
            Some(profile) => profile.is_puzzle_unlocked(puzzle),
            None => puzzle == Puzzle::first(),
        }
    }

    pub fn unlock_puzzle(&mut self, puzzle: Puzzle) -> Result<(), String> {
        match self.profile.as_mut() {
            Some(profile) => profile.unlock_puzzle(puzzle),
            None => Err("No profile loaded".to_string()),
        }
    }

    pub fn is_puzzle_solved(&self, puzzle: Puzzle) -> bool {
        self.profile
            .as_ref()
            .map_or(false, |profile| profile.is_puzzle_solved(puzzle))
    }

    pub fn puzzle_scores(&self, puzzle: Puzzle) -> &[(i32, i32)] {
        if let Some(ref profile) = self.profile {
            profile.puzzle_scores(puzzle)
        } else {
            &[]
        }
    }

    pub fn record_current_puzzle_score(&mut self, area: i32, score: i32)
                                       -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            let puzzle = profile.current_puzzle();
            debug_log!("Recording {:?} score (area={}, score={})",
                       puzzle,
                       area,
                       score);
            profile.record_puzzle_score(puzzle, area, score)?;
            debug_assert!(profile.is_puzzle_solved(puzzle));
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }

    pub fn circuit_name(&self) -> &str { &self.circuit_name }

    pub fn set_circuit_name(&mut self, name: String) {
        self.circuit_name = name;
    }

    pub fn is_valid_circuit_rename(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        if unicase::eq(name, &self.circuit_name) {
            return true;
        }
        if let Some(ref profile) = self.profile {
            if profile.has_circuit_name(profile.current_puzzle(), name) {
                return false;
            }
        }
        return true;
    }

    pub fn copy_current_circuit(&mut self) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            let puzzle = profile.current_puzzle();
            let mut new_name;
            let mut num: u64 = 1;
            loop {
                new_name = format!("{}.{}", self.circuit_name, num);
                if !profile.has_circuit_name(puzzle, &new_name) {
                    break;
                }
                num += 1;
            }
            if new_name.width() > CIRCUIT_NAME_MAX_WIDTH {
                num = 1;
                loop {
                    new_name = format!("Version {}", num);
                    if !profile.has_circuit_name(puzzle, &new_name) {
                        break;
                    }
                    num += 1;
                }
            }
            profile.copy_circuit(puzzle, &self.circuit_name, &new_name)?;
            self.circuit_name = new_name;
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }

    pub fn delete_current_circuit(&mut self) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            let puzzle = profile.current_puzzle();
            profile.delete_circuit(puzzle, &self.circuit_name)?;
            self.circuit_name = profile
                .first_circuit_name_for_current_puzzle()
                .unwrap_or_else(String::new);
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }

    pub fn rename_current_circuit(&mut self, new_name: String)
                                  -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            let puzzle = profile.current_puzzle();
            profile.rename_circuit(puzzle, &self.circuit_name, &new_name)?;
            self.circuit_name = new_name;
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }

    pub fn edit_grid(&self) -> Option<&EditGrid> { self.edit_grid.as_ref() }

    pub fn edit_grid_mut(&mut self) -> Option<&mut EditGrid> {
        self.edit_grid.as_mut()
    }

    pub fn clear_edit_grid(&mut self) { self.edit_grid = None; }

    pub fn new_edit_grid(&mut self) {
        let puzzle = self.current_puzzle();
        if let Some(ref profile) = self.profile {
            let mut num: u64 = 1;
            loop {
                self.circuit_name = format!("Version {}", num);
                if !profile.has_circuit_name(puzzle, &self.circuit_name) {
                    break;
                }
                num += 1;
            }
        }
        debug_log!("Creating new circuit {:?}", self.circuit_name);
        self.edit_grid = Some(EditGrid::new(puzzle));
    }

    pub fn load_edit_grid(&mut self) -> Result<(), String> {
        if let Some(ref profile) = self.profile {
            let puzzle = profile.current_puzzle();
            let data = profile.load_circuit(puzzle, &self.circuit_name)?;
            self.edit_grid = Some(EditGrid::from_circuit_data(puzzle, &data));
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }
}

//===========================================================================//
