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
const LIST_ICONS_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/texture/list_icons.png"));

#[cfg_attr(rustfmt, rustfmt_skip)]
const PORTRAITS_PNG_DATA: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/texture/portraits.png"));

const RED_PLANET_JPEG_DATA: &[u8] = include_bytes!("scene/red_planet.jpeg");

const STARFIELD_JPEG_DATA: &[u8] = include_bytes!("scene/starfield.jpeg");

#[cfg_attr(rustfmt, rustfmt_skip)]
const VALLEY_HEIGHTMAP_PNG_DATA: &[u8] =
    include_bytes!("scene/valley_heightmap.png");

#[cfg_attr(rustfmt, rustfmt_skip)]
const WIRE_TEXTURE1D_DATA: &[u8; 1024] = &[
    // 0-bit (6 + 4 pixels):
    0, 0, 255, 32,
    0, 0, 255, 64,
    0, 0, 255, 96,
    0, 0, 255, 128,
    0, 0, 255, 160,
    0, 0, 255, 192,
    192, 255, 0, 0,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    192, 255, 0, 0,
    0, 0, 255, 192,
    0, 0, 255, 160,
    0, 0, 255, 128,
    0, 0, 255, 96,
    0, 0, 255, 64,
    0, 0, 255, 32,
    // 1-bit (6 + 6 pixels):
    0, 0, 255, 32,
    0, 0, 255, 64,
    0, 0, 255, 96,
    0, 0, 255, 128,
    0, 0, 255, 160,
    0, 0, 255, 192,
    128, 255, 0, 0,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    128, 255, 0, 0,
    0, 0, 255, 192,
    0, 0, 255, 160,
    0, 0, 255, 128,
    0, 0, 255, 96,
    0, 0, 255, 64,
    0, 0, 255, 32,
    // 2-bit (6 + 10 pixels):
    0, 0, 255, 32,
    0, 0, 255, 64,
    0, 0, 255, 96,
    0, 0, 255, 128,
    0, 0, 255, 160,
    0, 0, 255, 192,
    128, 255, 0, 0,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    192, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    192, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    192, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    192, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    128, 255, 0, 0,
    0, 0, 255, 192,
    0, 0, 255, 160,
    0, 0, 255, 128,
    0, 0, 255, 96,
    0, 0, 255, 64,
    0, 0, 255, 32,
    // 4-bit (6 + 16 pixels):
    0, 0, 255, 32,
    0, 0, 255, 64,
    0, 0, 255, 96,
    0, 0, 255, 128,
    0, 0, 255, 160,
    0, 0, 255, 192,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    0, 0, 255, 192,
    0, 0, 255, 160,
    0, 0, 255, 128,
    0, 0, 255, 96,
    0, 0, 255, 64,
    0, 0, 255, 32,
    // 8-bit (6 + 22 pixels):
    0, 0, 255, 32,
    0, 0, 255, 64,
    0, 0, 255, 96,
    0, 0, 255, 128,
    0, 0, 255, 160,
    0, 0, 255, 192,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    255, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 0, 255, 192,
    0, 0, 255, 160,
    0, 0, 255, 128,
    0, 0, 255, 96,
    0, 0, 255, 64,
    0, 0, 255, 32,
    // 16-bit (6 + 30 pixels):
    0, 0, 255, 32,
    0, 0, 255, 64,
    0, 0, 255, 96,
    0, 0, 255, 128,
    0, 0, 255, 160,
    0, 0, 255, 192,
    128, 255, 0, 0,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    144, 255, 0, 0,
    160, 255, 0, 0,
    176, 255, 0, 0,
    192, 255, 0, 0,
    192, 255, 0, 0,
    176, 255, 0, 0,
    160, 255, 0, 0,
    144, 255, 0, 0,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    144, 255, 0, 0,
    160, 255, 0, 0,
    176, 255, 0, 0,
    192, 255, 0, 0,
    192, 255, 0, 0,
    176, 255, 0, 0,
    160, 255, 0, 0,
    144, 255, 0, 0,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    144, 255, 0, 0,
    160, 255, 0, 0,
    176, 255, 0, 0,
    192, 255, 0, 0,
    192, 255, 0, 0,
    176, 255, 0, 0,
    160, 255, 0, 0,
    144, 255, 0, 0,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    144, 255, 0, 0,
    160, 255, 0, 0,
    176, 255, 0, 0,
    192, 255, 0, 0,
    192, 255, 0, 0,
    176, 255, 0, 0,
    160, 255, 0, 0,
    144, 255, 0, 0,
    128, 255, 0, 0,
    0, 255, 0, 0,
    0, 255, 0, 0,
    128, 255, 0, 0,
    128, 255, 0, 0,
    0, 0, 255, 192,
    0, 0, 255, 160,
    0, 0, 255, 128,
    0, 0, 255, 96,
    0, 0, 255, 64,
    0, 0, 255, 32,
    // Padding:
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

//===========================================================================//

pub struct Textures {
    brushed_metal: Texture2D,
    chip_icons: Texture2D,
    list_icons: Texture2D,
    portraits: Texture2D,
    red_planet: Texture2D,
    starfield: Texture2D,
    valley_heightmap: Texture2D,
    white: Texture2D,
    wire: Texture1D,
}

impl Textures {
    pub fn new() -> Result<Textures, String> {
        let brushed_metal = Texture2D::from_jpeg("brushed_metal",
                                                 BRUSHED_METAL_JPEG_DATA)?;
        let chip_icons = Texture2D::from_png("texture/chip_icons",
                                             CHIP_ICONS_PNG_DATA)?;
        let list_icons = Texture2D::from_png("texture/list_icons",
                                             LIST_ICONS_PNG_DATA)?;
        let portraits = Texture2D::from_png("texture/portraits",
                                            PORTRAITS_PNG_DATA)?;
        let red_planet = Texture2D::from_jpeg("red_planet",
                                              RED_PLANET_JPEG_DATA)?;
        let starfield = Texture2D::from_jpeg("starfield",
                                             STARFIELD_JPEG_DATA)?;
        let valley_heightmap = Texture2D::from_png("valley_heightmap",
                                                   VALLEY_HEIGHTMAP_PNG_DATA)?;
        let white = Texture2D::new_rgba(1, 1, &[255, 255, 255, 255])?;
        let wire = Texture1D::new_rgba(WIRE_TEXTURE1D_DATA)?;
        let textures = Textures {
            brushed_metal,
            chip_icons,
            list_icons,
            portraits,
            red_planet,
            starfield,
            valley_heightmap,
            white,
            wire,
        };
        Ok(textures)
    }

    pub fn brushed_metal(&self) -> &Texture2D { &self.brushed_metal }

    pub fn chip_icons(&self) -> &Texture2D { &self.chip_icons }

    pub fn list_icons(&self) -> &Texture2D { &self.list_icons }

    pub fn portraits(&self) -> &Texture2D { &self.portraits }

    pub fn red_planet(&self) -> &Texture2D { &self.red_planet }

    pub fn starfield(&self) -> &Texture2D { &self.starfield }

    pub fn valley_heightmap(&self) -> &Texture2D { &self.valley_heightmap }

    pub fn white(&self) -> &Texture2D { &self.white }

    pub fn wire(&self) -> &Texture1D { &self.wire }
}

//===========================================================================//
