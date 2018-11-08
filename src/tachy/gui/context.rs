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

//===========================================================================//

pub struct GuiContext {
    _sdl_context: sdl2::Sdl,
    pub(super) video_subsystem: sdl2::VideoSubsystem,
    pub(super) event_pump: sdl2::EventPump,
}

impl GuiContext {
    pub fn init() -> Result<GuiContext, String> {
        let sdl_context = sdl2::init()?;
        if cfg!(any(target_os = "ios", target_os = "macos")) {
            sdl2::hint::set("SDL_MAC_CTRL_CLICK_EMULATE_RIGHT_CLICK", "1");
        }
        let video_subsystem = sdl_context.video()?;
        let event_pump = sdl_context.event_pump()?;
        Ok(GuiContext {
               _sdl_context: sdl_context,
               video_subsystem,
               event_pump,
           })
    }

    pub fn get_native_resolution(&self) -> Result<(u32, u32), String> {
        let display_mode = self.video_subsystem.desktop_display_mode(0)?;
        Ok((display_mode.w as u32, display_mode.h as u32))
    }
}

//===========================================================================//
