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

use tachy::gl::Texture1D;

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
const WIRE_TEXTURE1D_DATA: &[u8; 128] = &[
    // 0-bit (4 pixels):
    255, 255, 0, 0,
    // 1-bit (6 pixels):
    255, 255, 0, 0, 192, 192,
    // 2-bit (10 pixels):
    255, 255, 0, 0, 0, 192, 192, 192, 0, 0,
    // 4-bit (16 pixels):
    255, 255, 0, 0, 192, 192, 192, 0, 0, 0, 0, 192, 192, 192, 0, 0,
    // 8-bit (22 pixels):
    255, 255, 0, 192, 192, 192, 0, 0, 192, 192, 192,
    0, 0, 192, 192, 192, 0, 0, 192, 192, 192, 0,
    // 16-bit (30 pixels):
    255, 255, 0, 192, 192, 0, 0, 192, 192, 0, 0, 192, 192, 0, 0,
    192, 192, 0, 0, 192, 192, 0, 0, 192, 192, 0, 0, 192, 192, 0,
    // 32-bit (40 pixels):
    255, 0, 192, 0, 192, 0, 192, 0, 192, 0,
    192, 0, 192, 0, 192, 0, 192, 0, 192, 0,
    192, 0, 192, 0, 192, 0, 192, 0, 192, 0,
    192, 0, 192, 0, 192, 0, 192, 0, 192, 0,
];

//===========================================================================//

pub struct Textures {
    wire: Texture1D,
}

impl Textures {
    pub fn new() -> Result<Textures, String> {
        let wire = Texture1D::new_red(WIRE_TEXTURE1D_DATA)?;
        Ok(Textures { wire })
    }

    pub fn wire(&self) -> &Texture1D { &self.wire }
}

//===========================================================================//
