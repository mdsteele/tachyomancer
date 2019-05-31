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
use sdl2;
use sdl2::keyboard::Keycode;

//===========================================================================//

pub struct Ui<'a> {
    audio: &'a mut AudioQueue,
    clipboard: &'a Clipboard,
    cursor: &'a mut NextCursor,
    event_pump: &'a sdl2::EventPump,
    redraw_requested: &'a mut bool,
}

impl<'a> Ui<'a> {
    pub(super) fn new(audio: &'a mut AudioQueue, clipboard: &'a Clipboard,
                      cursor: &'a mut NextCursor,
                      event_pump: &'a sdl2::EventPump,
                      redraw_requested: &'a mut bool)
                      -> Ui<'a> {
        Ui {
            audio,
            clipboard,
            cursor,
            event_pump,
            redraw_requested,
        }
    }

    pub fn audio(&mut self) -> &mut AudioQueue { &mut self.audio }

    pub fn clipboard(&self) -> &Clipboard { self.clipboard }

    pub fn cursor(&mut self) -> &mut NextCursor { &mut self.cursor }

    pub fn keyboard(&self) -> Keyboard {
        Keyboard { state: self.event_pump.keyboard_state() }
    }

    pub fn request_redraw(&mut self) { *self.redraw_requested = true; }
}

//===========================================================================//

pub struct Keyboard<'a> {
    state: sdl2::keyboard::KeyboardState<'a>,
}

impl<'a> Keyboard<'a> {
    pub fn is_held(&self, keycode: Keycode) -> bool {
        if let Some(scancode) =
            sdl2::keyboard::Scancode::from_keycode(keycode)
        {
            self.state.is_scancode_pressed(scancode)
        } else {
            false
        }
    }
}

//===========================================================================//
