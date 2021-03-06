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

mod dir;
mod encode;
mod hotkey;
mod menu;
mod prefs;
mod profile;
mod progress;
mod score;

pub use self::dir::{ProfileNamesIter, SaveDir};
pub use self::hotkey::{Hotkey, HotkeyCodeExt, HotkeyIter, HOTKEY_CATEGORIES};
pub use self::menu::MenuSection;
pub use self::prefs::Prefs;
pub use self::profile::{Profile, PROFILE_NAME_MAX_CHARS};
pub use self::progress::{CircuitNamesIter, CIRCUIT_NAME_MAX_CHARS};
pub use self::score::GlobalScoresDir;

//===========================================================================//
