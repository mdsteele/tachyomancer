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

use super::types::ConversationBuilder;
use crate::mancer::save::Profile;
use tachy::save::Puzzle;

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn memory(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.esra("Time to learn about RAM.");
    builder.puzzle(profile, Puzzle::TutorialRam)?;
    builder.esra("Good job.");
    Ok(())
}

//===========================================================================//
