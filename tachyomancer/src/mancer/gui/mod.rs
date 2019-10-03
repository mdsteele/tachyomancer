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

mod audio;
mod clipboard;
mod context;
mod cursor;
mod debug;
mod event;
mod resource;
mod score;
mod ui;
mod window;

pub use self::audio::{AudioQueue, Music, Sound};
pub use self::clipboard::Clipboard;
pub use self::context::GuiContext;
pub use self::cursor::{Cursor, Cursors, NextCursor};
pub use self::event::{
    ClockEventData, Event, KeyEventData, Keycode, MouseEventData,
    MultitouchEventData, ScrollEventData,
};
pub use self::resource::Resources;
pub use self::score::GlobalScores;
pub use self::ui::{Keyboard, Ui};
pub use self::window::{Window, WindowOptions};

//===========================================================================//
