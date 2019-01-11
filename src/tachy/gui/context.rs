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

use super::audio::{AudioMixer, AudioQueue};
use sdl2;
use std::sync::{Arc, Mutex};

//===========================================================================//

pub struct GuiContext {
    _sdl_context: sdl2::Sdl,
    pub(super) video_subsystem: sdl2::VideoSubsystem,
    pub(super) event_pump: sdl2::EventPump,
    _audio_subsystem: sdl2::AudioSubsystem,
    _audio_device: sdl2::audio::AudioDevice<AudioMixer>,
    pub(super) audio_queue: Arc<Mutex<AudioQueue>>,
}

impl GuiContext {
    pub fn init(init_sound_volume_percent: i32) -> Result<GuiContext, String> {
        let sdl_context = sdl2::init()?;
        if cfg!(any(target_os = "ios", target_os = "macos")) {
            sdl2::hint::set("SDL_MAC_CTRL_CLICK_EMULATE_RIGHT_CLICK", "1");
        }
        let video_subsystem = sdl_context.video()?;
        let event_pump = sdl_context.event_pump()?;

        let audio_subsystem = sdl_context.audio()?;
        let mut audio_queue = AudioQueue::new();
        audio_queue.set_sound_volume_percent(init_sound_volume_percent);
        let audio_queue = Arc::new(Mutex::new(audio_queue));
        let audio_device = AudioMixer::audio_device(&audio_subsystem,
                                                    audio_queue.clone())?;
        audio_device.resume();

        Ok(GuiContext {
               _sdl_context: sdl_context,
               video_subsystem,
               event_pump,
               _audio_subsystem: audio_subsystem,
               _audio_device: audio_device,
               audio_queue,
           })
    }

    pub fn get_native_resolution(&self) -> Result<(u32, u32), String> {
        let display_mode = self.video_subsystem.desktop_display_mode(0)?;
        Ok((display_mode.w as u32, display_mode.h as u32))
    }
}

//===========================================================================//
