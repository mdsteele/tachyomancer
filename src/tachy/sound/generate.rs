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

use std::f32;

//===========================================================================//

const MASTER_VOL: f32 = 0.4;

const SUPERSAMPLE: i32 = 8;

const TWO_PI: f32 = 2.0 * f32::consts::PI;

//===========================================================================//

#[allow(dead_code)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum WaveKind {
    Noise,
    Sawtooth,
    Sine,
    Square,
    Triangle,
    Wobble,
}

impl Default for WaveKind {
    fn default() -> WaveKind { WaveKind::Sine }
}

//===========================================================================//

#[derive(Default)]
pub struct SoundSpec {
    pub wave_kind: WaveKind,
    pub env_attack: f32,
    pub env_sustain: f32,
    pub env_punch: f32,
    pub env_decay: f32,
    pub start_freq: f32,
    pub freq_limit: f32,
    pub freq_slide: f32,
    pub freq_delta_slide: f32,
    pub vibrato_depth: f32,
    pub vibrato_speed: f32,
    pub arp_mod: f32,
    pub arp_speed: f32,
    pub square_duty: f32,
    pub duty_sweep: f32,
    pub repeat_speed: f32,
    pub phaser_offset: f32,
    pub phaser_sweep: f32,
    pub lpf_cutoff: f32,
    pub lpf_ramp: f32,
    pub lpf_resonance: f32,
    pub hpf_cutoff: f32,
    pub hpf_ramp: f32,
    pub volume_adjust: f32, // -1.0 to 1.0
}

impl SoundSpec {
    pub fn new() -> SoundSpec { SoundSpec::default() }

    pub fn generate(&self) -> Vec<f32> { Synth::generate(self) }
}

//===========================================================================//

/// Xorshift RNG (see http://en.wikipedia.org/wiki/Xorshift)
struct Rng {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

impl Rng {
    fn new() -> Rng {
        Rng {
            x: 123456789,
            y: 362436069,
            z: 521288629,
            w: 88675123,
        }
    }

    /// Returns a random f32 from -1 to 1.
    fn next_f32(&mut self) -> f32 {
        let t: u32 = self.x ^ (self.x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        self.w = self.w ^ (self.w >> 19) ^ t ^ (t >> 8);
        (((self.w as f64) * 4.656612874161595e-10) - 1.0) as f32
    }
}

impl Default for Rng {
    fn default() -> Rng { Rng::new() }
}

/*===========================================================================*/
// Synthesizer:

// This section of the file contains a modified version of the sound generation
// code from sfxr (http://www.drpetter.se/project_sfxr.html), written by Tomas
// Pettersson ("DrPetter").  sfxr is made available under the MIT/Expat
// license, reproduced here:

// *********************************************************************
// * sfxr                                                              *
// * Copyright (c) 2007 Tomas Pettersson                               *
// *                                                                   *
// * Permission is hereby granted, free of charge, to any person       *
// * obtaining a copy of this software and associated documentation    *
// * files (the "Software"), to deal in the Software without           *
// * restriction, including without limitation the rights to use,      *
// * copy, modify, merge, publish, distribute, sublicense, and/or sell *
// * copies of the Software, and to permit persons to whom the         *
// * Software is furnished to do so, subject to the following          *
// * conditions:                                                       *
// *                                                                   *
// * The above copyright notice and this permission notice shall be    *
// * included in all copies or substantial portions of the Software.   *
// *                                                                   *
// * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,   *
// * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES   *
// * OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND          *
// * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT       *
// * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,      *
// * WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING      *
// * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR     *
// * OTHER DEALINGS IN THE SOFTWARE.                                   *
// *********************************************************************

// Many thanks to DrPetter for developing sfxr, and for releasing it as Free
// Software.

#[derive(Default)]
struct Synth {
    rng: Rng,
    phase: i32,
    fperiod: f64,
    fmaxperiod: f64,
    fslide: f64,
    fdslide: f64,
    period: i32,
    square_duty: f32,
    square_slide: f32,
    env_stage: usize,
    env_time: i32,
    env_length: [i32; 3],
    env_vol: f32,
    fphase: f32,
    fdphase: f32,
    iphase: i32,
    phaser_buffer: Vec<f32>,
    ipp: i32,
    noise_buffer: Vec<f32>,
    fltp: f32,
    fltdp: f32,
    fltw: f32,
    fltw_d: f32,
    fltdmp: f32,
    fltphp: f32,
    flthp: f32,
    flthp_d: f32,
    vib_phase: f32,
    vib_speed: f32,
    vib_amp: f32,
    rep_time: i32,
    rep_limit: i32,
    arp_time: i32,
    arp_limit: i32,
    arp_mod: f64,
}

impl Synth {
    fn refill_noise_buffer(&mut self) {
        for entry in self.noise_buffer.iter_mut() {
            *entry = self.rng.next_f32();
        }
    }

    /// Resets the synth.
    fn reset(&mut self, spec: &SoundSpec, restart: bool) {
        // This code is taken directly from sfxr, with only minor changes.
        if !restart {
            self.phase = 0;
            self.phaser_buffer = vec![0.0; 1024];
            self.noise_buffer = vec![0.0; 32];
        }
        self.fperiod = 100.0 / ((spec.start_freq as f64).powi(2) + 0.001);
        self.period = self.fperiod as i32;
        self.fmaxperiod = 100.0 / ((spec.freq_limit as f64).powi(2) + 0.001);
        self.fslide = 1.0 - (spec.freq_slide as f64).powi(3) * 0.01;
        self.fdslide = -(spec.freq_delta_slide as f64).powi(3) * 0.000001;
        self.square_duty = 0.5 - spec.square_duty * 0.5;
        self.square_slide = -spec.duty_sweep * 0.00005;
        if spec.arp_mod >= 0.0 {
            self.arp_mod = 1.0 - (spec.arp_mod as f64).powi(2) * 0.9;
        } else {
            self.arp_mod = 1.0 + (spec.arp_mod as f64).powi(2) * 10.0;
        }
        self.arp_time = 0;
        self.arp_limit = ((1.0 - spec.arp_speed).powi(2) * 20000.0 + 32.0) as
            i32;
        if spec.arp_speed == 1.0 {
            self.arp_limit = 0;
        }
        if !restart {
            // Reset filter:
            self.fltp = 0.0;
            self.fltdp = 0.0;
            self.fltw = (1.0 - spec.lpf_cutoff).powi(3) * 0.1;
            self.fltw_d = 1.0 + spec.lpf_ramp * 0.0001;
            self.fltdmp = 5.0 / (1.0 + spec.lpf_resonance.powi(2) * 20.0) *
                (0.01 + self.fltw);
            if self.fltdmp > 0.8 {
                self.fltdmp = 0.8;
            }
            self.fltphp = 0.0;
            self.flthp = spec.hpf_cutoff.powi(2) * 0.1;
            self.flthp_d = 1.0 + spec.hpf_ramp * 0.0003;
            // Reset vibrato:
            self.vib_phase = 0.0;
            self.vib_speed = spec.vibrato_speed.powi(2) * 0.01;
            self.vib_amp = spec.vibrato_depth * 0.5;
            // Reset envelope:
            self.env_vol = 0.0;
            self.env_stage = 0;
            self.env_time = 0;
            self.env_length[0] =
                (spec.env_attack * spec.env_attack * 100000.0) as i32;
            self.env_length[1] =
                (spec.env_sustain * spec.env_sustain * 100000.0) as i32;
            self.env_length[2] =
                (spec.env_decay * spec.env_decay * 100000.0) as i32;
            // Reset phaser:
            self.fphase = spec.phaser_offset.powi(2) * 1020.0;
            if spec.phaser_offset < 0.0 {
                self.fphase = -self.fphase;
            }
            self.fdphase = spec.phaser_sweep.powi(2);
            if spec.phaser_sweep < 0.0 {
                self.fdphase = -self.fdphase;
            }
            self.iphase = (self.fphase as i32).abs();
            self.ipp = 0;
            for entry in self.phaser_buffer.iter_mut() {
                *entry = 0.0;
            }
            // Refill noise buffer:
            self.refill_noise_buffer();
            // Reset repeat:
            self.rep_time = 0;
            self.rep_limit = ((1.0 - spec.repeat_speed).powi(2) * 20000.0 +
                                  32.0) as i32;
            if spec.repeat_speed == 0.0 {
                self.rep_limit = 0;
            }
        }
    }

    fn generate(spec: &SoundSpec) -> Vec<f32> {
        let mut synth = Synth::default();
        synth.reset(spec, false);
        let mut finished = false;

        let mut samples = Vec::<f32>::new();
        while !finished {
            synth.rep_time += 1;
            if synth.rep_limit != 0 && synth.rep_time >= synth.rep_limit {
                synth.rep_time = 0;
                synth.reset(spec, true);
            }

            // frequency envelopes/arpeggios
            synth.arp_time += 1;
            if synth.arp_limit != 0 && synth.arp_time >= synth.arp_limit {
                synth.arp_limit = 0;
                synth.fperiod *= synth.arp_mod;
            }
            synth.fslide += synth.fdslide;
            synth.fperiod *= synth.fslide;
            if synth.fperiod > synth.fmaxperiod {
                synth.fperiod = synth.fmaxperiod;
                if spec.freq_limit > 0.0 {
                    finished = true;
                }
            }
            let mut rfperiod: f32 = synth.fperiod as f32;
            if synth.vib_amp > 0.0 {
                synth.vib_phase += synth.vib_speed;
                rfperiod = (synth.fperiod as f32) *
                    (1.0 + synth.vib_phase.sin() * synth.vib_amp);
            }
            synth.period = rfperiod as i32;
            if synth.period < 8 {
                synth.period = 8;
            }
            synth.square_duty += synth.square_slide;
            if synth.square_duty < 0.0 {
                synth.square_duty = 0.0;
            }
            if synth.square_duty > 0.5 {
                synth.square_duty = 0.5;
            }
            // volume envelope
            synth.env_time += 1;
            if synth.env_time > synth.env_length[synth.env_stage] {
                synth.env_time = 0;
                synth.env_stage += 1;
                if synth.env_stage == 3 {
                    finished = true;
                }
            }
            if synth.env_stage == 0 {
                debug_assert!(synth.env_length[0] > 0);
                synth.env_vol = (synth.env_time as f32) /
                    (synth.env_length[0] as f32);
            }
            if synth.env_stage == 1 {
                synth.env_vol = 1.0;
                if synth.env_length[1] > 0 {
                    synth.env_vol += (1.0 -
                                          (synth.env_time as f32) /
                                              (synth.env_length[1] as f32)) *
                        2.0 *
                        spec.env_punch;
                }
            }
            if synth.env_stage == 2 {
                synth.env_vol = if synth.env_length[2] > 0 {
                    1.0 -
                        (synth.env_time as f32) / (synth.env_length[2] as f32)
                } else {
                    1.0
                };
            }

            // phaser step
            synth.fphase += synth.fdphase;
            synth.iphase = (synth.fphase as i32).abs();
            if synth.iphase > 1023 {
                synth.iphase = 1023;
            }
            if synth.flthp_d != 0.0 {
                synth.flthp *= synth.flthp_d;
                if synth.flthp < 0.00001 {
                    synth.flthp = 0.00001;
                }
                if synth.flthp > 0.1 {
                    synth.flthp = 0.1;
                }
            }

            let mut ssample: f32 = 0.0;
            for _ in 0..SUPERSAMPLE {
                synth.phase += 1;
                if synth.phase >= synth.period {
                    synth.phase %= synth.period;
                    if spec.wave_kind == WaveKind::Noise {
                        synth.refill_noise_buffer();
                    }
                }
                // base waveform
                debug_assert!(synth.period > 0);
                let mut fp: f32 = (synth.phase as f32) / (synth.period as f32);
                let mut sample: f32 = match spec.wave_kind {
                    WaveKind::Noise => {
                        synth.noise_buffer[(synth.phase * 32 / synth.period) as
                                               usize]
                    }
                    WaveKind::Sawtooth => 1.0 - fp * 2.0,
                    WaveKind::Sine => (fp * TWO_PI).sin(),
                    WaveKind::Square => {
                        if fp < synth.square_duty { 0.5 } else { -0.5 }
                    }
                    WaveKind::Triangle => 4.0 * (fp - 0.5).abs() - 1.0,
                    WaveKind::Wobble => {
                        0.5 * ((fp * TWO_PI).cos() + (2.0 * fp * TWO_PI).sin())
                    }
                };
                // lp filter
                let mut pp: f32 = synth.fltp;
                synth.fltw *= synth.fltw_d;
                if synth.fltw < 0.0 {
                    synth.fltw = 0.0;
                }
                if synth.fltw > 0.1 {
                    synth.fltw = 0.1;
                }
                if spec.lpf_cutoff != 0.0 {
                    synth.fltdp += (sample - synth.fltp) * synth.fltw;
                    synth.fltdp -= synth.fltdp * synth.fltdmp;
                } else {
                    synth.fltp = sample;
                    synth.fltdp = 0.0;
                }
                synth.fltp += synth.fltdp;
                // hp filter
                synth.fltphp += synth.fltp - pp;
                synth.fltphp -= synth.fltphp * synth.flthp;
                sample = synth.fltphp;
                // phaser
                debug_assert_eq!(synth.phaser_buffer.len(), 1024);
                synth.phaser_buffer[(synth.ipp as usize) & 1023] = sample;
                let phaser_index = (synth.ipp - synth.iphase + 1024) & 1023;
                sample += synth.phaser_buffer[phaser_index as usize];
                synth.ipp = (synth.ipp + 1) & 1023;
                // final accumulation and envelope application
                ssample += sample * synth.env_vol;
            }
            ssample *= MASTER_VOL / (SUPERSAMPLE as f32);
            ssample *= 1.0 + spec.volume_adjust;
            samples.push(ssample.max(-1.0).min(1.0));
        }
        samples
    }
}

/*===========================================================================*/
