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

use gl;
use gl::types::{GLint, GLsizei, GLuint};
use std::os::raw::c_void;

//===========================================================================//

pub struct Texture1D {
    name: GLuint,
}

impl Texture1D {
    /// Creates a new texture.  The length of the data array must be a power of
    /// two, with one byte for each pixel.
    pub fn new_gray(data: &[u8]) -> Result<Texture1D, String> {
        let width = data.len();
        if !width.is_power_of_two() {
            return Err(format!("1D texture has width of {}, \
                                which is not a power of 2",
                               width));
        }
        let mut name: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut name);
            gl::BindTexture(gl::TEXTURE_1D, name);
            gl::TexImage1D(gl::TEXTURE_1D,
                           0,
                           gl::RED as GLint,
                           width as GLsizei,
                           0,
                           gl::RED,
                           gl::UNSIGNED_BYTE,
                           data.as_ptr() as *const c_void);
            gl::TexParameteri(gl::TEXTURE_1D,
                              gl::TEXTURE_MAG_FILTER,
                              gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_1D,
                              gl::TEXTURE_MIN_FILTER,
                              gl::LINEAR_MIPMAP_LINEAR as GLint);
            gl::GenerateMipmap(gl::TEXTURE_1D);
        }
        return Ok(Texture1D { name });
    }

    /// Sets this as the current texture.
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_1D, self.name);
        }
    }
}

/// Deletes the underlying GL texture when dropped.
impl Drop for Texture1D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.name);
        }
    }
}

//===========================================================================//
