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

use crate::tachy::music;
use crate::tachy::sound;
use lewton::inside_ogg::OggStreamReader;
use lewton::samples::InterleavedSamples;
use sdl2;
use sdl2::audio::AudioFormatNum;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

// ========================================================================= //

const DESIRED_AUDIO_RATE: i32 = sound::AUDIO_RATE as i32;
const DESIRED_BUFFER_SIZE: u16 = 1024; // num samples
const DESIRED_NUM_CHANNELS: u8 = 1; // mono

const MUSIC_FADE_OUT_SECONDS: f32 = 0.75;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Music {
    MorningCruise,
}

impl Music {
    fn ogg_data(&self) -> &'static [u8] {
        match *self {
            Music::MorningCruise => music::MORNING_CRUISE_OGG_DATA,
        }
    }
}

//===========================================================================//

pub struct AudioQueue {
    sounds: Vec<Sound>,
    sound_volume: Option<f32>, // 0.0 to 1.0
    music: Option<Music>,
    music_volume: Option<f32>, // 0.0 to 1.0
}

impl AudioQueue {
    pub fn new() -> AudioQueue {
        AudioQueue {
            sounds: Vec::new(),
            sound_volume: None,
            music: None,
            music_volume: None,
        }
    }

    pub fn play_sound(&mut self, sound: Sound) {
        self.sounds.push(sound);
    }

    pub fn play_music(&mut self, music: Music) {
        self.music = Some(music);
    }

    pub fn set_sound_volume_percent(&mut self, percent: i32) {
        self.sound_volume = Some(0.01 * (percent.max(0).min(100) as f32));
    }

    pub fn set_music_volume_percent(&mut self, percent: i32) {
        self.music_volume = Some(0.01 * (percent.max(0).min(100) as f32));
    }

    pub(super) fn merge(&mut self, other: AudioQueue) {
        self.sounds.extend(other.sounds);
        self.sound_volume = other.sound_volume.or(self.sound_volume);
        self.music = other.music.or(self.music);
        self.music_volume = other.music_volume.or(self.music_volume);
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

struct MusicStream {
    music: Music,
    samples: Vec<f32>,
    reader: OggStreamReader<Cursor<&'static [u8]>>,
}

impl MusicStream {
    pub fn new(music: Music) -> Result<MusicStream, String> {
        let cursor = Cursor::new(music.ogg_data());
        match OggStreamReader::new(cursor) {
            Ok(reader) => {
                if reader.ident_hdr.audio_sample_rate != sound::AUDIO_RATE {
                    return Err(format!(
                        "Sample rate of {:?} is {}, but \
                         expected {}",
                        music,
                        reader.ident_hdr.audio_sample_rate,
                        sound::AUDIO_RATE
                    ));
                }
                if reader.ident_hdr.audio_channels != DESIRED_NUM_CHANNELS {
                    return Err(format!(
                        "Found {} channels in {:?}, but \
                         expected {}",
                        reader.ident_hdr.audio_channels,
                        music,
                        DESIRED_NUM_CHANNELS
                    ));
                }
                let stream =
                    MusicStream { music, samples: Vec::new(), reader };
                return Ok(stream);
            }
            Err(error) => {
                return Err(format!(
                    "Failed to decode {:?} header: {:?}",
                    music, error
                ));
            }
        }
    }

    pub fn restart(&mut self) -> Result<(), String> {
        *self = MusicStream::new(self.music)?;
        Ok(())
    }

    pub fn read(&mut self, out: &mut [f32]) -> Result<usize, String> {
        while self.samples.is_empty() {
            match self
                .reader
                .read_dec_packet_generic::<InterleavedSamples<f32>>()
            {
                Ok(Some(interleaved)) => {
                    debug_assert_eq!(
                        interleaved.channel_count,
                        DESIRED_NUM_CHANNELS.into()
                    );
                    self.samples = interleaved.samples;
                }
                Ok(None) => return Ok(0),
                Err(error) => {
                    return Err(format!(
                        "Failed to decode {:?} samples: {:?}",
                        self.music, error
                    ));
                }
            }
        }
        let len = self.samples.len().min(out.len());
        for index in 0..len {
            out[index] = self.samples[index];
        }
        self.samples.drain(..len);
        return Ok(len);
    }
}

//===========================================================================//

pub struct AudioMixer {
    audio_data: AudioData,
    audio_queue: Arc<Mutex<AudioQueue>>,
    active_sounds: Vec<(Sound, usize)>,
    sound_volume: f32, // 0.0 to 1.0
    current_music: Option<MusicStream>,
    next_music: Option<(Music, f32)>,
    music_volume: f32, // 0.0 to 1.0
}

impl AudioMixer {
    fn new(
        audio_queue: Arc<Mutex<AudioQueue>>,
        audio_data: AudioData,
    ) -> AudioMixer {
        let mut mixer = AudioMixer {
            audio_queue,
            audio_data,
            active_sounds: Vec::new(),
            sound_volume: 0.0,
            current_music: None,
            next_music: None,
            music_volume: 0.0,
        };
        mixer.drain_queue();
        mixer
    }

    pub fn audio_device(
        audio_subsystem: &sdl2::AudioSubsystem,
        audio_queue: Arc<Mutex<AudioQueue>>,
    ) -> Result<sdl2::audio::AudioDevice<AudioMixer>, String> {
        let audio_data = AudioData::new()?;
        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(DESIRED_AUDIO_RATE),
            channels: Some(DESIRED_NUM_CHANNELS),
            samples: Some(DESIRED_BUFFER_SIZE),
        };
        let device =
            audio_subsystem.open_playback(None, &desired_spec, |_| {
                AudioMixer::new(audio_queue, audio_data)
            })?;
        {
            let actual_spec = device.spec();
            if actual_spec.freq != DESIRED_AUDIO_RATE
                || actual_spec.format != f32::audio_format()
                || actual_spec.channels != DESIRED_NUM_CHANNELS
            {
                return Err(format!(
                    "Could not initialize a compatible audio \
                     device (desired: {{ freq: {}, \
                     format: {:?}, channels: {}, \
                     samples: {} }}, actual: {:?})",
                    DESIRED_AUDIO_RATE,
                    f32::audio_format(),
                    DESIRED_NUM_CHANNELS,
                    DESIRED_BUFFER_SIZE,
                    actual_spec
                ));
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
        if let Some(volume) = audio_queue.music_volume.take() {
            self.music_volume = volume;
        }
        if let Some(new_music) = audio_queue.music.take() {
            if let Some(ref music_stream) = self.current_music {
                if music_stream.music != new_music {
                    match self.next_music {
                        Some((next_music, _)) if next_music == new_music => {}
                        Some((_, fade)) => {
                            self.next_music = Some((new_music, fade));
                        }
                        None => self.next_music = Some((new_music, 1.0)),
                    }
                }
            } else {
                match MusicStream::new(new_music) {
                    Ok(stream) => self.current_music = Some(stream),
                    Err(error) => {
                        debug_warn!(
                            "Failed to start {:?} music: {}",
                            new_music,
                            error
                        );
                    }
                }
            }
        }
    }
}

impl sdl2::audio::AudioCallback for AudioMixer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.drain_queue();

        // Music:
        if self.music_volume <= 0.0 {
            for sample in out.iter_mut() {
                *sample = 0.0;
            }
        } else {
            let mut start: usize = 0;
            if let Some(ref mut music_stream) = self.current_music {
                while start < out.len() {
                    match music_stream.read(&mut out[start..]) {
                        Ok(0) => match music_stream.restart() {
                            Ok(()) => {}
                            Err(error) => {
                                debug_warn!(
                                    "Failed to restart music: {}",
                                    error
                                );
                                self.current_music = None;
                                self.next_music = None;
                                break;
                            }
                        },
                        Ok(num_samples) => {
                            start += num_samples;
                        }
                        Err(error) => {
                            debug_warn!("Failed to stream music: {}", error);
                            self.current_music = None;
                            self.next_music = None;
                            break;
                        }
                    }
                }
            } else {
                debug_assert!(self.next_music.is_none());
            }
            for sample in out[start..].iter_mut() {
                *sample = 0.0;
            }
        }
        let fade = if let Some((music, old_fade)) = self.next_music {
            let new_fade = old_fade
                - (out.len() as f32)
                    / ((DESIRED_AUDIO_RATE as f32) * MUSIC_FADE_OUT_SECONDS);
            if new_fade > 0.0 {
                self.next_music = Some((music, new_fade));
            } else {
                self.next_music = None;
                match MusicStream::new(music) {
                    Ok(music_stream) => {
                        self.current_music = Some(music_stream);
                    }
                    Err(error) => {
                        debug_warn!(
                            "Failed to start {:?} music: {}",
                            music,
                            error
                        );
                        self.current_music = None;
                    }
                }
            }
            old_fade
        } else {
            1.0
        };
        let music_volume = fade * self.music_volume;
        for sample in out.iter_mut() {
            *sample *= music_volume;
        }

        // Sounds:
        let mut remaining_sounds = Vec::new();
        for (sound, start) in self.active_sounds.drain(..) {
            let data = self.audio_data.sound_data(sound);
            let len = out.len().min(data.len() - start);
            let end = start + len;
            let slice = &data[start..end];
            for index in 0..len {
                out[index] += slice[index] * self.sound_volume;
            }
            if end < data.len() {
                remaining_sounds.push((sound, end));
            }
        }
        self.active_sounds = remaining_sounds;
    }
}

//===========================================================================//
