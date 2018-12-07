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
    edit_grid: Option<EditGrid>,
}

impl GameState {
    pub fn new(savedir: SaveDir) -> Result<GameState, String> {
        let profile = savedir.load_current_profile_if_any()?;
        let menu_section = MenuSection::Navigation;
        let state = GameState {
            savedir,
            menu_section,
            profile,
            edit_grid: None,
        };
        Ok(state)
    }

    pub fn save(&mut self) -> Result<(), String> {
        if let Some(ref mut profile) = self.profile {
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
        self.profile = Some(self.savedir.create_or_load_profile(name)?);
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
            profile.set_current_puzzle(puzzle);
        }
    }

    pub fn edit_grid_mut(&mut self) -> Option<&mut EditGrid> {
        self.edit_grid.as_mut()
    }

    pub fn clear_edit_grid(&mut self) { self.edit_grid = None; }

    pub fn new_edit_grid(&mut self) {
        self.edit_grid = Some(EditGrid::new(self.current_puzzle()));
    }
}

//===========================================================================//
