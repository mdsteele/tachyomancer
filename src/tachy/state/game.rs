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
use tachy::save::{Puzzle, SaveDir};

//===========================================================================//

pub struct GameState {
    savedir: SaveDir,
    current_puzzle: Puzzle,
    edit_grid: Option<EditGrid>,
}

impl GameState {
    pub fn new(savedir: SaveDir) -> Result<GameState, String> {
        // TODO: Load current profile.
        Ok(GameState {
               savedir,
               current_puzzle: Puzzle::SandboxEvent,
               edit_grid: Some(EditGrid::new()),
           })
    }

    pub fn savedir(&self) -> &SaveDir { &self.savedir }

    pub fn current_puzzle(&self) -> Puzzle { self.current_puzzle }

    pub fn edit_grid_mut(&mut self) -> Option<&mut EditGrid> {
        self.edit_grid.as_mut()
    }
}

//===========================================================================//
