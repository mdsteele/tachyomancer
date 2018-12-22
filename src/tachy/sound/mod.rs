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

pub fn drop_chip_data() -> Result<Vec<f32>, String> {
    decode_flac("drop-chip.flac", include_bytes!("drop-chip.flac"))
}

pub fn grab_chip_data() -> Result<Vec<f32>, String> {
    decode_flac("grab-chip.flac", include_bytes!("grab-chip.flac"))
}

//===========================================================================//

fn decode_flac(name: &str, data: &[u8]) -> Result<Vec<f32>, String> {
    let mut reader =
        FlacReader::new(data)
            .map_err(|err| format!("Failed to decode {}: {}", name, err))?;
    let streaminfo = reader.streaminfo();
    if streaminfo.sample_rate != AUDIO_RATE {
        return Err(format!("Sample rate of {} is {}, but expected {}",
                           name,
                           streaminfo.sample_rate,
                           AUDIO_RATE));
    }
    if streaminfo.channels != 1 {
        return Err(format!("Found {} channels in {}, but expected mono",
                           streaminfo.channels,
                           name));
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
        let sample =
            sample
                .map_err(|err| format!("Failed to decode {}: {}", name, err))?;
        samples.push(scale * (sample as f32));
    }
    if samples.len() != num_samples {
        return Err(format!("Wrong number of samples for {} \
                            (expected {}, but was {})",
                           name,
                           num_samples,
                           samples.len()));
    }
    Ok(samples)
}

//===========================================================================//
