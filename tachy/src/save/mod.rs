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

mod chip;
mod circuit;
mod converse;
mod hotkey;
mod puzzle;
mod score;
mod set;
mod size;
mod solution;
mod wire;

pub use self::chip::{ChipSet, ChipType, CHIP_CATEGORIES, MAX_COMMENT_CHARS};
pub use self::circuit::CircuitData;
pub use self::converse::{
    Chapter, Conversation, ConversationIter, ConversationProgress, Prereq,
};
pub use self::hotkey::HotkeyCode;
pub use self::puzzle::{Puzzle, PuzzleIter, PuzzleKind, ScoreUnits};
pub use self::score::{ScoreCurve, ScoreCurveMap};
pub use self::set::PuzzleSet;
pub use self::size::{WireSize, WireSizeInterval};
pub use self::solution::{InputsData, SolutionData};
pub use self::wire::WireShape;

//===========================================================================//
