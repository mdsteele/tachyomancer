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
pub use sdl2::keyboard::Keycode;

//===========================================================================//

pub enum Event {
    Quit,
    KeyDown(KeyEventData),
}

impl Event {
    pub(super) fn from_sdl_event(sdl_event: sdl2::event::Event)
                                 -> Option<Event> {
        match sdl_event {
            sdl2::event::Event::Quit { .. } => Some(Event::Quit),
            sdl2::event::Event::KeyDown { keycode, keymod, .. } => {
                if let Some(code) = keycode {
                    Some(Event::KeyDown(KeyEventData::new(code, keymod)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

//===========================================================================//

pub struct KeyEventData {
    pub code: Keycode,
    pub command: bool,
    pub shift: bool,
}

impl KeyEventData {
    fn new(keycode: Keycode, keymod: sdl2::keyboard::Mod) -> KeyEventData {
        let shift = sdl2::keyboard::LSHIFTMOD | sdl2::keyboard::RSHIFTMOD;
        let command = if cfg!(any(target_os = "ios", target_os = "macos")) {
            sdl2::keyboard::LGUIMOD | sdl2::keyboard::RGUIMOD
        } else {
            sdl2::keyboard::LCTRLMOD | sdl2::keyboard::RCTRLMOD
        };
        KeyEventData {
            code: keycode,
            shift: keymod.intersects(shift),
            command: keymod.intersects(command),
        }
    }
}

//===========================================================================//
