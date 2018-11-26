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
use png;
use std::marker::PhantomData;
use std::os::raw::c_void;

//===========================================================================//

pub struct Texture1D {
    name: GLuint,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

impl Texture1D {
    /// Creates a new one-color texture.  The length of the data array must be
    /// a power of two, with one byte for each pixel.
    pub fn new_red(data: &[u8]) -> Result<Texture1D, String> {
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
                           gl::R8 as GLint,
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
        return Ok(Texture1D {
                      name,
                      phantom: PhantomData,
                  });
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

pub struct Texture2D {
    name: GLuint,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

impl Texture2D {
    /// Creates a new texture from PNG data.  The width and height must each be
    /// a power of two.
    pub fn from_png(png_name: &str, png_data: &[u8])
                    -> Result<Texture2D, String> {
        let decoder = png::Decoder::new(png_data);
        let (info, mut reader) = decoder
            .read_info()
            .map_err(|err| {
                         format!("Failed to read PNG header for {}: {}",
                                 png_name,
                                 err)
                     })?;
        if info.bit_depth != png::BitDepth::Eight {
            return Err(format!("PNG {} has unsupported bit depth: {:?}",
                               png_name,
                               info.bit_depth));
        }
        let (format, internal_format) = match info.color_type {
            png::ColorType::Grayscale => (gl::RED, gl::R8),
            png::ColorType::RGB => (gl::RGB, gl::RGB8),
            png::ColorType::RGBA => (gl::RGBA, gl::RGBA8),
            _ => {
                return Err(format!("PNG {} has unsupported color type: {:?}",
                                   png_name,
                                   info.color_type));
            }

        };
        let width = info.width as usize;
        let height = info.height as usize;
        if !width.is_power_of_two() || !height.is_power_of_two() {
            return Err(format!("Texture PNG {} has size of {}x{}, \
                                which are not both powers of 2",
                               png_name,
                               width,
                               height));
        }
        let mut data = vec![0u8; info.color_type.samples() * width * height];
        reader
            .next_frame(&mut data)
            .map_err(|err| {
                         format!("Failed to decode PNG data for {}: {}",
                                 png_name,
                                 err)
                     })?;
        let mut name: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut name);
            gl::BindTexture(gl::TEXTURE_2D, name);
            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           internal_format as GLint,
                           width as GLsizei,
                           height as GLsizei,
                           0,
                           format,
                           gl::UNSIGNED_BYTE,
                           data.as_ptr() as *const c_void);
            gl::TexParameteri(gl::TEXTURE_2D,
                              gl::TEXTURE_MAG_FILTER,
                              gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D,
                              gl::TEXTURE_MIN_FILTER,
                              gl::LINEAR as GLint);
        }
        return Ok(Texture2D {
                      name,
                      phantom: PhantomData,
                  });
    }

    /// Sets this as the current texture.
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.name);
        }
    }
}

/// Deletes the underlying GL texture when dropped.
impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.name);
        }
    }
}

//===========================================================================//
