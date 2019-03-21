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

use sdl2;
use sdl2::clipboard::ClipboardUtil;

//===========================================================================//

pub struct Clipboard {
    util: ClipboardUtil,
}

impl Clipboard {
    pub(super) fn new(video_subsystem: &sdl2::VideoSubsystem) -> Clipboard {
        Clipboard { util: video_subsystem.clipboard() }
    }

    #[allow(dead_code)]
    pub fn get(&self) -> Option<String> {
        match self.util.clipboard_text() {
            Ok(text) => Some(text),
            Err(err) => {
                debug_log!("Cannot get clipboard text: {}", err);
                None
            }
        }
    }

    pub fn set(&self, text: &str) {
        match self.util.set_clipboard_text(text) {
            Ok(()) => {}
            Err(err) => {
                debug_log!("Cannot set clipboard text: {}", err);
            }
        }
    }
}

//===========================================================================//
