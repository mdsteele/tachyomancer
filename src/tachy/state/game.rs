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
    current_puzzle: Puzzle, // TODO: move this to profile
    profile: Option<Profile>,
    edit_grid: Option<EditGrid>,
}

impl GameState {
    pub fn new(savedir: SaveDir) -> Result<GameState, String> {
        let profile = savedir.load_current_profile_if_any()?;
        // TODO: Get current puzzle from profile.
        let menu_section = MenuSection::Puzzles;
        let current_puzzle = Puzzle::SandboxEvent;
        let state = GameState {
            savedir,
            menu_section,
            current_puzzle,
            profile,
            edit_grid: None,
        };
        Ok(state)
    }

    pub fn savedir(&self) -> &SaveDir { &self.savedir }

    pub fn profile(&self) -> Option<&Profile> { self.profile.as_ref() }

    pub fn create_or_load_profile(&mut self, name: String)
                                  -> Result<(), String> {
        self.profile = Some(self.savedir.create_or_load_profile(name)?);
        Ok(())
    }

    pub fn clear_profile(&mut self) { self.profile = None; }

    pub fn menu_section(&self) -> MenuSection { self.menu_section }

    pub fn set_menu_section(&mut self, section: MenuSection) {
        self.menu_section = section;
    }

    pub fn current_puzzle(&self) -> Puzzle { self.current_puzzle }

    pub fn set_current_puzzle(&mut self, puzzle: Puzzle) {
        self.current_puzzle = puzzle;
    }

    pub fn edit_grid_mut(&mut self) -> Option<&mut EditGrid> {
        self.edit_grid.as_mut()
    }

    pub fn clear_edit_grid(&mut self) { self.edit_grid = None; }

    pub fn new_edit_grid(&mut self) {
        self.edit_grid = Some(EditGrid::new(self.current_puzzle));
    }
}

//===========================================================================//
