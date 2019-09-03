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
use sdl2::audio::AudioFormatNum;
use std::sync::{Arc, Mutex};
use tachy::sound;

// ========================================================================= //

const DESIRED_AUDIO_RATE: i32 = sound::AUDIO_RATE as i32;
const DESIRED_BUFFER_SIZE: u16 = 1024; // num samples
const DESIRED_NUM_CHANNELS: u8 = 1; // mono

//===========================================================================//

#[derive(Clone, Copy, Debug)]
pub enum Sound {
    Beep = 0,
    ButtonClick,
    ButtonHover,
    ChangeBounds,
    DragWire,
    DropChip,
    GrabChip,
    TypeKey,
}

//===========================================================================//

pub struct AudioQueue {
    sounds: Vec<Sound>,
    sound_volume: Option<f32>, // 0.0 to 1.0
}

impl AudioQueue {
    pub fn new() -> AudioQueue {
        AudioQueue {
            sounds: Vec::new(),
            sound_volume: None,
        }
    }

    pub fn play_sound(&mut self, sound: Sound) { self.sounds.push(sound); }

    pub fn set_sound_volume_percent(&mut self, percent: i32) {
        self.sound_volume = Some(0.01 * (percent.max(0).min(100) as f32));
    }

    pub(super) fn merge(&mut self, other: AudioQueue) {
        self.sounds.extend(other.sounds);
        self.sound_volume = other.sound_volume.or(self.sound_volume);
    }
}

//===========================================================================//

struct AudioData {
    sound_data: Vec<Vec<f32>>,
}

impl AudioData {
    fn new() -> Result<AudioData, String> {
        let mut sound_data = Vec::<Vec<f32>>::new();
        sound_data.push(sound::beep_data()?);
        sound_data.push(sound::button_click_data()?);
        sound_data.push(sound::button_hover_data()?);
        sound_data.push(sound::change_bounds_data()?);
        sound_data.push(sound::drag_wire_data()?);
        sound_data.push(sound::drop_chip_data()?);
        sound_data.push(sound::grab_chip_data()?);
        sound_data.push(sound::type_key_data()?);
        Ok(AudioData { sound_data })
    }

    fn sound_data(&self, sound: Sound) -> &[f32] {
        &self.sound_data[sound as usize]
    }
}

//===========================================================================//

pub struct AudioMixer {
    audio_data: AudioData,
    audio_queue: Arc<Mutex<AudioQueue>>,
    active_sounds: Vec<(Sound, usize)>,
    sound_volume: f32, // 0.0 to 1.0
}

impl AudioMixer {
    fn new(audio_queue: Arc<Mutex<AudioQueue>>, audio_data: AudioData)
           -> AudioMixer {
        let mut mixer = AudioMixer {
            audio_queue,
            audio_data,
            active_sounds: Vec::new(),
            sound_volume: 0.0,
        };
        mixer.drain_queue();
        mixer
    }

    pub fn audio_device(
        audio_subsystem: &sdl2::AudioSubsystem,
        audio_queue: Arc<Mutex<AudioQueue>>)
        -> Result<sdl2::audio::AudioDevice<AudioMixer>, String> {
        let audio_data = AudioData::new()?;
        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(DESIRED_AUDIO_RATE),
            channels: Some(DESIRED_NUM_CHANNELS),
            samples: Some(DESIRED_BUFFER_SIZE),
        };
        let device = audio_subsystem
            .open_playback(None, &desired_spec, |_| {
                AudioMixer::new(audio_queue, audio_data)
            })?;
        {
            let actual_spec = device.spec();
            if actual_spec.freq != DESIRED_AUDIO_RATE ||
                actual_spec.format != f32::audio_format() ||
                actual_spec.channels != DESIRED_NUM_CHANNELS
            {
                return Err(format!("Could not initialize a compatible audio \
                                    device (desired: {{ freq: {}, \
                                    format: {:?}, channels: {}, \
                                    samples: {} }}, actual: {:?})",
                                   DESIRED_AUDIO_RATE,
                                   f32::audio_format(),
                                   DESIRED_NUM_CHANNELS,
                                   DESIRED_BUFFER_SIZE,
                                   actual_spec));
            }
        }
        return Ok(device);
    }

    fn drain_queue(&mut self) {
        let mut audio_queue = self.audio_queue.lock().unwrap();
        for sound in audio_queue.sounds.drain(..) {
            self.active_sounds.push((sound, 0));
        }
        if let Some(volume) = audio_queue.sound_volume.take() {
            self.sound_volume = volume;
        }
    }
}

impl sdl2::audio::AudioCallback for AudioMixer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.drain_queue();
        for sample in out.iter_mut() {
            *sample = 0.0;
        }
        let mut remaining_sounds = Vec::new();
        for (sound, start) in self.active_sounds.drain(..) {
            let data = self.audio_data.sound_data(sound);
            let len = out.len().min(data.len() - start);
            let end = start + len;
            let slice = &data[start..end];
            for index in 0..len {
                out[index] += slice[index];
            }
            if end < data.len() {
                remaining_sounds.push((sound, end));
            }
        }
        for sample in out.iter_mut() {
            *sample *= self.sound_volume;
        }
        self.active_sounds = remaining_sounds;
    }
}

//===========================================================================//
