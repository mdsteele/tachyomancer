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

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
const GALACTICO_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/font/galactico.png"));
#[cfg_attr(rustfmt, rustfmt_skip)]
const INCONSOLATA_BOLD_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/font/inconsolata-bold.png"));
#[cfg_attr(rustfmt, rustfmt_skip)]
const INCONSOLATA_REGULAR_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/font/inconsolata-regular.png"));

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
}

//===========================================================================//

#[derive(Clone, Copy, Debug, EnumString, Eq, PartialEq)]
pub enum Font {
    Alien,
    Bold,
    Roman,
}

impl Font {
    pub fn ratio(self) -> f32 {
        match self {
            Font::Alien => 0.7,
            Font::Bold => 0.5,
            Font::Roman => 0.5,
        }
    }

    pub fn str_width(&self, height: f32, text: &str) -> f32 {
        Font::str_width_for_ratio(self.ratio(), height, text)
    }

    pub(super) fn str_width_for_ratio(ratio: f32, height: f32, text: &str)
                                      -> f32 {
        ratio * height * (text.chars().count() as f32)
    }

    pub(super) fn png_name_and_data(self) -> (&'static str, &'static [u8]) {
        match self {
            Font::Alien => ("font/galactico", GALACTICO_PNG_DATA),
            Font::Bold => ("font/inconsolata-bold", INCONSOLATA_BOLD_PNG_DATA),
            Font::Roman => {
                ("font/inconsolata-regular", INCONSOLATA_REGULAR_PNG_DATA)
            }
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
