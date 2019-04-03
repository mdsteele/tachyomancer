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

use tachy::gl::{Texture1D, Texture2D};

//===========================================================================//

const BRUSHED_METAL_JPEG_DATA: &[u8] = include_bytes!("brushed_metal.jpeg");

#[cfg_attr(rustfmt, rustfmt_skip)]
const CHIP_ICONS_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/texture/chip_icons.png"));

#[cfg_attr(rustfmt, rustfmt_skip)]
const WIRE_TEXTURE1D_DATA: &[u8; 256] = &[
    // 0-bit (4 pixels):
    192, 128, 0, 0, 0, 0, 128, 192,
    // 1-bit (6 pixels):
    128, 128, 0, 0, 255, 255, 255, 255, 0, 0, 128, 128,
    // 2-bit (10 pixels):
    128, 128, 0, 0, 192, 255, 255, 192, 0, 0,
    0, 0, 192, 255, 255, 192, 0, 0, 128, 128,
    // 4-bit (16 pixels):
    128, 0, 0, 0, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 0, 0,
    0, 0, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 0, 0, 0, 128,
    // 8-bit (22 pixels):
    0, 0, 0, 255, 255, 255, 0, 0, 255, 255, 255,
    0, 0, 255, 255, 255, 0, 0, 255, 255, 255, 0,
    0, 255, 255, 255, 0, 0, 255, 255, 255, 0, 0,
    255, 255, 255, 0, 0, 255, 255, 255, 0, 0, 0,
    // 16-bit (30 pixels):
    128, 128, 0, 0, 128, 144, 160, 176, 192, 192, 176, 160, 144, 128, 0,
    0, 0, 0, 128, 144, 160, 176, 192, 192, 176, 160, 144, 128, 0, 0,
    0, 0, 128, 144, 160, 176, 192, 192, 176, 160, 144, 128, 0, 0, 0,
    0, 128, 144, 160, 176, 192, 192, 176, 160, 144, 128, 0, 0, 128, 128,
    // 32-bit (40 pixels):
    128, 128, 0, 0, 0, 0,
    132, 136, 140, 144, 148, 152, 156, 160, 164, 168, 172,
    174, 176, 180, 184, 188, 192, 188, 184, 180, 176, 172,
    168, 164, 160, 156, 152, 148, 144, 140, 136, 132,
    0, 0, 0, 0,
    132, 136, 140, 144, 148, 152, 156, 160, 164, 168, 172,
    174, 176, 180, 184, 188, 192, 188, 184, 180, 176, 172,
    168, 164, 160, 156, 152, 148, 144, 140, 136, 132,
    0, 0, 0, 0, 128, 128,
];

//===========================================================================//

pub struct Textures {
    brushed_metal: Texture2D,
    chip_icons: Texture2D,
    wire: Texture1D,
}

impl Textures {
    pub fn new() -> Result<Textures, String> {
        let brushed_metal = Texture2D::from_jpeg("brushed_metal",
                                                 BRUSHED_METAL_JPEG_DATA)?;
        let chip_icons = Texture2D::from_png("texture/chip_icons",
                                             CHIP_ICONS_PNG_DATA)?;
        let wire = Texture1D::new_red(WIRE_TEXTURE1D_DATA)?;
        Ok(Textures {
               brushed_metal,
               chip_icons,
               wire,
           })
    }

    pub fn brushed_metal(&self) -> &Texture2D { &self.brushed_metal }

    pub fn chip_icons(&self) -> &Texture2D { &self.chip_icons }

    pub fn wire(&self) -> &Texture1D { &self.wire }
}

//===========================================================================//
