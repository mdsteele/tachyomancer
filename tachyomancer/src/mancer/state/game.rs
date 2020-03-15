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

use super::cutscene::CutsceneScript;
use crate::mancer::save::{
    MenuSection, Prefs, Profile, ProfileNamesIter, SaveDir,
};
use std::time::Duration;
use tachy::save::{Chapter, Conversation, Puzzle, ScoreCurve};
use tachy::state::EditGrid;
use unicase;

//===========================================================================//

const AUTOSAVE_DURATION: Duration = Duration::from_secs(60);

//===========================================================================//

pub struct GameState {
    savedir: SaveDir,
    menu_section: MenuSection,
    profile: Option<Profile>,
    circuit_name: String,
    edit_grid: Option<EditGrid>,
    cutscene: Option<CutsceneScript>,
}

impl GameState {
    pub fn new(savedir: SaveDir) -> Result<GameState, String> {
        let opt_profile = savedir.load_current_profile_if_any()?;
        let menu_section = MenuSection::Navigation;
        let mut circuit_name = String::new();
        if let Some(ref profile) = opt_profile {
            if let Some(name) = profile.last_circuit_name_for_current_puzzle()
            {
                circuit_name = name;
            }
        }
        Ok(GameState {
            savedir,
            menu_section,
            profile: opt_profile,
            circuit_name,
            edit_grid: None,
            cutscene: None,
        })
    }

    pub fn save(&mut self) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            if let Some(ref mut grid) = self.edit_grid {
                if grid.is_modified() {
                    let puzzle = profile.current_puzzle();
                    let circuit_data = grid.to_circuit_data();
                    profile.save_circuit(
                        puzzle,
                        &self.circuit_name,
                        &circuit_data,
                    )?;
                    grid.mark_unmodified();
                }
            }
            profile.save()?;
        }
        self.savedir.save()?;
        Ok(())
    }

    pub fn maybe_autosave_circuit(&mut self) {
        if let Some(ref mut grid) = self.edit_grid {
            if grid.has_been_modified_for_at_least(AUTOSAVE_DURATION)
                && !grid.has_provisional_changes()
            {
                grid.mark_unmodified();
                if let Some(ref mut profile) = self.profile {
                    let puzzle = profile.current_puzzle();
                    let circuit_data = grid.to_circuit_data();
                    match profile.save_circuit(
                        puzzle,
                        &self.circuit_name,
                        &circuit_data,
                    ) {
                        Ok(()) => return,
                        Err(err) => debug_log!("Failed to autosave: {}", err),
                    }
                } else {
                    debug_log!("Failed to autosave: no profile!");
                }
                grid.mark_modified();
            }
        }
    }

    pub fn prefs(&self) -> &Prefs {
        self.savedir.prefs()
    }

    pub fn prefs_mut(&mut self) -> &mut Prefs {
        self.savedir.prefs_mut()
    }

    pub fn cutscene(&self) -> Option<&CutsceneScript> {
        self.cutscene.as_ref()
    }

    pub fn cutscene_mut_and_prefs(
        &mut self,
    ) -> Option<(&mut CutsceneScript, &Prefs)> {
        if let Some(ref mut cutscene) = self.cutscene {
            Some((cutscene, self.savedir.prefs()))
        } else {
            None
        }
    }

    pub fn set_cutscene(&mut self, cutscene: CutsceneScript) {
        self.cutscene = Some(cutscene);
    }

    pub fn clear_cutscene(&mut self) {
        self.cutscene = None;
    }

    pub fn profile_names(&self) -> ProfileNamesIter {
        self.savedir.profile_names()
    }

    pub fn has_profile(&self, name: &str) -> bool {
        self.savedir.has_profile(name)
    }

    pub fn current_profile_is(&self, name: &str) -> bool {
        self.savedir.current_profile_is(name)
    }

    pub fn profile(&self) -> Option<&Profile> {
        self.profile.as_ref()
    }

    pub fn load_profile(&self, name: &str) -> Result<Profile, String> {
        self.savedir.load_profile(name)
    }

    pub fn create_or_load_and_set_profile(
        &mut self,
        name: &str,
    ) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            profile.save()?;
        }
        let profile = self.savedir.create_or_load_and_set_profile(name)?;
        self.circuit_name = profile
            .last_circuit_name_for_current_puzzle()
            .unwrap_or_else(String::new);
        self.profile = Some(profile);
        Ok(())
    }

    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        let is_current: bool = if let Some(ref profile) = self.profile {
            profile.name() == name
        } else {
            false
        };
        self.savedir.delete_profile(name)?;
        if is_current {
            self.profile = self.savedir.load_current_profile_if_any()?;
        }
        Ok(())
    }

    pub fn clear_profile(&mut self) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            profile.save()?;
        }
        self.profile = None;
        Ok(())
    }

    pub fn menu_section(&self) -> MenuSection {
        self.menu_section
    }

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

    pub fn set_current_conversation_progress(&mut self, progress: usize) {
        if let Some(ref mut profile) = self.profile {
            let conv = profile.current_conversation();
            profile.set_conversation_progress(conv, progress);
        }
    }

    pub fn reset_current_conversation_progress(&mut self) {
        if let Some(ref mut profile) = self.profile {
            let conv = profile.current_conversation();
            profile.reset_conversation_progress(conv);
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

    pub fn set_current_conversation_choice(
        &mut self,
        key: String,
        value: String,
    ) {
        if let Some(ref mut profile) = self.profile {
            let conv = profile.current_conversation();
            debug_log!(
                "Making conversation {:?} choice {:?} = {:?}",
                conv,
                key,
                value
            );
            profile.set_conversation_choice(conv, key, value);
        }
    }

    pub fn unlocked_chapters(&self) -> Vec<Chapter> {
        if let Some(ref profile) = self.profile {
            profile.unlocked_chapters()
        } else {
            vec![Chapter::first()]
        }
    }

    pub fn latest_chapter(&self) -> Chapter {
        if let Some(ref profile) = self.profile {
            profile.latest_chapter()
        } else {
            Chapter::first()
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
                    .last_circuit_name_for_current_puzzle()
                    .unwrap_or_else(String::new);
            }
        }
    }

    pub fn are_any_puzzles_unlocked(&self) -> bool {
        match self.profile.as_ref() {
            Some(profile) => profile.are_any_puzzles_unlocked(),
            None => false,
        }
    }

    pub fn is_puzzle_unlocked(&self, puzzle: Puzzle) -> bool {
        match self.profile.as_ref() {
            Some(profile) => profile.is_puzzle_unlocked(puzzle),
            None => false,
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

    pub fn local_scores(&self, puzzle: Puzzle) -> &ScoreCurve {
        if let Some(ref profile) = self.profile {
            profile.local_scores(puzzle)
        } else {
            ScoreCurve::EMPTY
        }
    }

    pub fn reset_local_scores(&mut self, puzzle: Puzzle) {
        if let Some(ref mut profile) = self.profile {
            profile.reset_local_scores(puzzle);
        }
    }

    pub fn record_puzzle_score(
        &mut self,
        puzzle: Puzzle,
        area: i32,
        score: u32,
    ) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            debug_log!(
                "Recording {:?} score (area={}, score={})",
                puzzle,
                area,
                score
            );
            profile.record_puzzle_score(puzzle, area, score)?;
            debug_assert!(profile.is_puzzle_solved(puzzle));
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }

    pub fn circuit_name(&self) -> &str {
        &self.circuit_name
    }

    pub fn set_circuit_name(&mut self, name: String) {
        self.circuit_name = name;
    }

    pub fn has_circuit_name(&self, name: &str) -> bool {
        if let Some(ref profile) = self.profile {
            profile.has_circuit_name(profile.current_puzzle(), name)
        } else {
            false
        }
    }

    pub fn is_valid_circuit_rename(&self, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() {
            return false;
        }
        if unicase::eq(name, &self.circuit_name) {
            return true;
        }
        return !self.has_circuit_name(name);
    }

    pub fn copy_current_circuit(&mut self) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            let prefix = format!("{}.", self.circuit_name);
            let new_name = profile.choose_new_circuit_name(&prefix);
            let puzzle = profile.current_puzzle();
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
                .last_circuit_name_for_current_puzzle()
                .unwrap_or_else(String::new);
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }

    /// Renames the current circuit to the given new name.  If there is no
    /// current circuit, creates a new circuit with the given name.
    pub fn rename_current_circuit(
        &mut self,
        new_name: &str,
    ) -> Result<(), String> {
        let new_name = new_name.trim();
        if !self.is_valid_circuit_rename(new_name) {
            Err(format!("Invalid rename: {:?}", new_name))
        } else if let Some(ref mut profile) = self.profile {
            let puzzle = profile.current_puzzle();
            if self.circuit_name.is_empty() {
                let solved = profile.solved_puzzles();
                let edit_grid = EditGrid::new(puzzle, &solved);
                let circuit_data = edit_grid.to_circuit_data();
                profile.save_circuit(puzzle, new_name, &circuit_data)?;
            } else {
                profile.rename_circuit(
                    puzzle,
                    &self.circuit_name,
                    new_name,
                )?;
            }
            self.circuit_name = new_name.to_string();
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }

    pub fn edit_grid(&self) -> Option<&EditGrid> {
        self.edit_grid.as_ref()
    }

    pub fn edit_grid_mut_and_prefs(
        &mut self,
    ) -> Option<(&mut EditGrid, &Prefs)> {
        if let Some(ref mut grid) = self.edit_grid {
            Some((grid, self.savedir.prefs()))
        } else {
            None
        }
    }

    pub fn clear_edit_grid(&mut self) {
        self.edit_grid = None;
    }

    pub fn load_and_set_edit_grid(&mut self) -> Result<(), String> {
        if let Some(ref profile) = self.profile {
            let puzzle = profile.current_puzzle();
            let solved = profile.solved_puzzles();
            if self.circuit_name.is_empty() {
                self.circuit_name =
                    profile.choose_new_circuit_name("Version ");
                debug_log!("Creating new circuit {:?}", self.circuit_name);
                self.edit_grid = Some(EditGrid::new(puzzle, &solved));
            } else {
                let data = profile.load_circuit(puzzle, &self.circuit_name)?;
                self.edit_grid =
                    Some(EditGrid::from_circuit_data(puzzle, &solved, &data));
            }
            Ok(())
        } else {
            Err("No profile loaded".to_string())
        }
    }

    pub fn load_edit_grid(
        &self,
        puzzle: Puzzle,
        circuit_name: &str,
    ) -> Result<EditGrid, String> {
        if let Some(ref profile) = self.profile {
            let solved = profile.solved_puzzles();
            if circuit_name.is_empty() {
                Ok(EditGrid::new(puzzle, &solved))
            } else {
                let data = profile.load_circuit(puzzle, circuit_name)?;
                Ok(EditGrid::from_circuit_data(puzzle, &solved, &data))
            }
        } else {
            Err("No profile loaded".to_string())
        }
    }
}

//===========================================================================//
