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

use super::audio::AudioQueue;
use super::clipboard::Clipboard;
use super::cursor::NextCursor;

//===========================================================================//

pub struct Ui<'a> {
    audio: &'a mut AudioQueue,
    clipboard: &'a Clipboard,
    cursor: &'a mut NextCursor,
}

impl<'a> Ui<'a> {
    pub(super) fn new(audio: &'a mut AudioQueue, clipboard: &'a Clipboard,
                      cursor: &'a mut NextCursor)
                      -> Ui<'a> {
        Ui {
            audio,
            clipboard,
            cursor,
        }
    }

    pub fn audio(&mut self) -> &mut AudioQueue { &mut self.audio }

    pub fn clipboard(&self) -> &Clipboard { self.clipboard }

    pub fn cursor(&mut self) -> &mut NextCursor { &mut self.cursor }

    // TODO: Allow getting keyboard state from the event pump
}

//===========================================================================//
