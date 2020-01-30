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

use tachy::geom::RectSize;

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
const GALACTICO_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/font/galactico_64.png"));
#[cfg_attr(rustfmt, rustfmt_skip)]
const GALACTICO_METRICS: (u32, u32, u32, u32) =
    include!(concat!(env!("OUT_DIR"), "/font/galactico_64_metrics.rs"));

#[cfg_attr(rustfmt, rustfmt_skip)]
const INCONSOLATA_BOLD_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/font/inconsolata-bold_64.png"));
#[cfg_attr(rustfmt, rustfmt_skip)]
const INCONSOLATA_BOLD_METRICS: (u32, u32, u32, u32) =
    include!(concat!(env!("OUT_DIR"), "/font/inconsolata-bold_64_metrics.rs"));

#[cfg_attr(rustfmt, rustfmt_skip)]
const INCONSOLATA_REGULAR_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/font/inconsolata-regular_64.png"));
#[cfg_attr(rustfmt, rustfmt_skip)]
const INCONSOLATA_REGULAR_METRICS: (u32, u32, u32, u32) =
    include!(concat!(env!("OUT_DIR"), "/font/inconsolata-regular_64_metrics.rs"));

#[cfg_attr(rustfmt, rustfmt_skip)]
const SEGMENT7_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/font/segment7_64.png"));
#[cfg_attr(rustfmt, rustfmt_skip)]
const SEGMENT7_METRICS: (u32, u32, u32, u32) =
    include!(concat!(env!("OUT_DIR"), "/font/segment7_64_metrics.rs"));

//===========================================================================//

#[derive(Clone, Copy)]
pub enum Align {
    TopLeft,
    MidLeft,
    TopCenter,
    MidCenter,
    BottomCenter,
    TopRight,
    MidRight,
    BottomRight,
}

//===========================================================================//

#[derive(Clone, Copy, Debug, EnumString, Eq, PartialEq)]
pub enum Font {
    Alien,
    Bold,
    Led,
    Roman,
}

impl Font {
    pub fn ratio(self) -> f32 {
        let (char_width, char_height, _, _) = self.metrics();
        (char_width as f32) / (char_height as f32)
    }

    pub fn str_width(&self, height: f32, text: &str) -> f32 {
        Font::str_width_for_ratio(self.ratio(), height, text)
    }

    pub(super) fn str_width_for_ratio(
        ratio: f32,
        height: f32,
        text: &str,
    ) -> f32 {
        ratio * height * (text.chars().count() as f32)
    }

    pub(super) fn png_name_and_data(self) -> (&'static str, &'static [u8]) {
        match self {
            Font::Alien => ("font/galactico", GALACTICO_PNG_DATA),
            Font::Bold => ("font/inconsolata-bold", INCONSOLATA_BOLD_PNG_DATA),
            Font::Led => ("font/segment7", SEGMENT7_PNG_DATA),
            Font::Roman => {
                ("font/inconsolata-regular", INCONSOLATA_REGULAR_PNG_DATA)
            }
        }
    }

    pub(super) fn char_tex_size(self) -> RectSize<f32> {
        let (char_width, char_height, tex_width, tex_height) = self.metrics();
        RectSize::new(
            (char_width as f32) / (tex_width as f32),
            (char_height as f32) / (tex_height as f32),
        )
    }

    pub fn metrics(self) -> (u32, u32, u32, u32) {
        match self {
            Font::Alien => GALACTICO_METRICS,
            Font::Bold => INCONSOLATA_BOLD_METRICS,
            Font::Led => SEGMENT7_METRICS,
            Font::Roman => INCONSOLATA_REGULAR_METRICS,
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::Font;
    use std::str::FromStr;

    #[test]
    fn font_from_str_round_trip() {
        for &font in &[Font::Alien, Font::Bold, Font::Roman] {
            assert_eq!(Font::from_str(&format!("{:?}", font)), Ok(font));
        }
    }
}

//===========================================================================//
