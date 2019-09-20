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

mod generate;

use self::generate::{SoundSpec, WaveKind};
use claxon::FlacReader;

//===========================================================================//

pub const AUDIO_RATE: u32 = 44100; // samples/second

//===========================================================================//

pub fn beep_data() -> Result<Vec<f32>, String> {
    let beep_num_samples = (AUDIO_RATE / 4) as usize;
    let mut beep_data = Vec::with_capacity(beep_num_samples);
    for idx in 0..beep_num_samples {
        let cycle = 440.0 * (idx as f32) / (AUDIO_RATE as f32);
        let sample = if cycle.fract() < 0.5 { -0.25 } else { 0.25 };
        beep_data.push(sample);
    }
    Ok(beep_data)
}

pub fn button_click_data() -> Result<Vec<f32>, String> {
    let mut spec = SoundSpec::new();
    spec.wave_kind = WaveKind::Triangle;
    spec.env_attack = 0.0891609;
    spec.env_sustain = 0.155;
    spec.env_punch = 0.0656827;
    spec.env_decay = 0.23;
    spec.start_freq = 0.19;
    spec.freq_slide = 0.00708617;
    spec.freq_delta_slide = -0.0759178;
    spec.vibrato_depth = 0.00914664;
    spec.vibrato_speed = 0.881547;
    spec.arp_mod = -0.67998;
    spec.arp_speed = 0.284788;
    spec.repeat_speed = 0.431267;
    spec.phaser_offset = -0.0525242;
    spec.phaser_sweep = -0.0142853;
    spec.lpf_cutoff = 0.103765;
    spec.lpf_ramp = 0.240105;
    spec.lpf_resonance = 0.173952;
    Ok(spec.generate())
}

pub fn button_hover_data() -> Result<Vec<f32>, String> {
    let mut spec = SoundSpec::new();
    spec.wave_kind = WaveKind::Triangle;
    spec.env_attack = 0.0891609;
    spec.env_sustain = 0.155;
    spec.env_punch = 0.0656827;
    spec.env_decay = 0.23;
    spec.start_freq = 0.215;
    spec.freq_slide = 0.00708617;
    spec.freq_delta_slide = -0.0759178;
    spec.vibrato_depth = 0.00914664;
    spec.vibrato_speed = 0.881547;
    spec.arp_mod = -0.67998;
    spec.arp_speed = 0.284788;
    spec.repeat_speed = 0.431267;
    spec.phaser_offset = -0.0525242;
    spec.phaser_sweep = -0.0142853;
    spec.lpf_cutoff = 0.103765;
    spec.lpf_ramp = 0.240105;
    spec.lpf_resonance = 0.173952;
    Ok(spec.generate())
}

pub fn change_bounds_data() -> Result<Vec<f32>, String> {
    let mut spec = SoundSpec::new();
    spec.wave_kind = WaveKind::Noise;
    spec.env_sustain = 0.1;
    spec.start_freq = 0.3;
    spec.freq_slide = 0.6;
    spec.lpf_cutoff = 0.735;
    spec.volume_adjust = -0.;
    Ok(spec.generate())
}

pub fn drag_wire_data() -> Result<Vec<f32>, String> {
    let mut spec = SoundSpec::new();
    spec.wave_kind = WaveKind::Noise;
    spec.env_decay = 0.115;
    spec.start_freq = 0.825;
    spec.freq_slide = -0.3;
    spec.phaser_offset = -0.7;
    spec.phaser_sweep = -0.04;
    spec.volume_adjust = -0.5;
    Ok(spec.generate())
}

pub fn drop_chip_data() -> Result<Vec<f32>, String> {
    decode_flac("drop-chip.flac", include_bytes!("drop-chip.flac"))
}

pub fn grab_chip_data() -> Result<Vec<f32>, String> {
    decode_flac("grab-chip.flac", include_bytes!("grab-chip.flac"))
}

pub fn type_key_data() -> Result<Vec<f32>, String> {
    decode_flac("type-key.flac", include_bytes!("type-key.flac"))
}

//===========================================================================//

fn decode_flac(name: &str, data: &[u8]) -> Result<Vec<f32>, String> {
    let mut reader = FlacReader::new(data)
        .map_err(|err| format!("Failed to decode {}: {}", name, err))?;
    let streaminfo = reader.streaminfo();
    if streaminfo.sample_rate != AUDIO_RATE {
        return Err(format!(
            "Sample rate of {} is {}, but expected {}",
            name, streaminfo.sample_rate, AUDIO_RATE
        ));
    }
    if streaminfo.channels != 1 {
        return Err(format!(
            "Found {} channels in {}, but expected mono",
            streaminfo.channels, name
        ));
    }
    let num_samples = match streaminfo.samples {
        Some(num) => num as usize,
        None => {
            return Err(format!("Number of samples not found for {}", name))
        }
    };
    let scale = 1.0 / ((1 << (streaminfo.bits_per_sample - 1)) as f32);
    let mut samples = Vec::<f32>::with_capacity(num_samples);
    for sample in reader.samples() {
        let sample = sample
            .map_err(|err| format!("Failed to decode {}: {}", name, err))?;
        samples.push(scale * (sample as f32));
    }
    if samples.len() != num_samples {
        return Err(format!(
            "Wrong number of samples for {} (expected {}, but was {})",
            name,
            num_samples,
            samples.len()
        ));
    }
    Ok(samples)
}

//===========================================================================//
