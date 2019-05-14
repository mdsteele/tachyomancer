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
use super::clipboard::Clipboard;
use super::cursor::Cursors;
use super::debug::StdinReader;
use sdl2;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tachy::geom::RectSize;

//===========================================================================//

pub struct GuiContext {
    pub(super) sdl_context: sdl2::Sdl,
    pub(super) video_subsystem: sdl2::VideoSubsystem,
    pub(super) clipboard: Clipboard,
    pub(super) event_pump: sdl2::EventPump,
    _audio_subsystem: sdl2::AudioSubsystem,
    _audio_device: sdl2::audio::AudioDevice<AudioMixer>,
    pub(super) audio_queue: Arc<Mutex<AudioQueue>>,
    pub(super) cursors: Cursors,
    pub(super) stdin_reader: StdinReader,
}

impl GuiContext {
    pub fn init(init_sound_volume_percent: i32) -> Result<GuiContext, String> {
        let sdl_context = sdl2::init()?;
        if cfg!(any(target_os = "ios", target_os = "macos")) {
            sdl2::hint::set("SDL_MAC_CTRL_CLICK_EMULATE_RIGHT_CLICK", "1");
        }
        let video_subsystem = sdl_context.video()?;
        let clipboard = Clipboard::new(&video_subsystem);
        let event_pump = sdl_context.event_pump()?;
        let cursors = Cursors::new()?;

        let audio_subsystem = sdl_context.audio()?;
        let mut audio_queue = AudioQueue::new();
        audio_queue.set_sound_volume_percent(init_sound_volume_percent);
        let audio_queue = Arc::new(Mutex::new(audio_queue));
        let audio_device = AudioMixer::audio_device(&audio_subsystem,
                                                    audio_queue.clone())?;
        audio_device.resume();

        Ok(GuiContext {
               sdl_context,
               video_subsystem,
               clipboard,
               event_pump,
               _audio_subsystem: audio_subsystem,
               _audio_device: audio_device,
               audio_queue,
               cursors,
               stdin_reader: StdinReader::start(),
           })
    }

    pub fn get_native_resolution(&self) -> Result<RectSize<i32>, String> {
        let display_mode = self.video_subsystem.desktop_display_mode(0)?;
        Ok(RectSize::new(display_mode.w, display_mode.h))
    }

    pub fn get_possible_resolutions(&self)
                                    -> Result<Vec<RectSize<i32>>, String> {
        let num_modes = self.video_subsystem.num_display_modes(0)?;
        let mut resolutions = HashSet::<RectSize<i32>>::new();
        for index in 0..num_modes {
            let mode = self.video_subsystem.display_mode(0, index)?;
            resolutions.insert(RectSize::new(mode.w, mode.h));
        }
        let mut resolutions: Vec<RectSize<i32>> =
            resolutions.into_iter().collect();
        resolutions.sort_by(|r1, r2| {
                                r2.width
                                    .cmp(&r1.width)
                                    .then(r2.height.cmp(&r1.height))
                            });
        Ok(resolutions)
    }
}

//===========================================================================//
