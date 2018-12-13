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
use tachy::save::{MenuSection, Profile, Puzzle, SaveDir};

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
        // TODO: save prefs if necessary
        Ok(())
    }

    pub fn savedir(&self) -> &SaveDir { &self.savedir }

    pub fn profile(&self) -> Option<&Profile> { self.profile.as_ref() }

    pub fn create_or_load_profile(&mut self, name: String)
                                  -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
            profile.save()?;
        }
        let profile = self.savedir.create_or_load_profile(name)?;
        self.circuit_name =
            match profile.circuit_names(profile.current_puzzle()).next() {
                Some(name) => name.to_string(),
                None => String::new(),
            };
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
                self.circuit_name =
                    match profile.circuit_names(puzzle).next() {
                        Some(name) => name.to_string(),
                        None => String::new(),
                    }
            }
        }
    }

    pub fn circuit_name(&self) -> &str { &self.circuit_name }

    pub fn set_circuit_name(&mut self, name: String) {
        self.circuit_name = name;
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
