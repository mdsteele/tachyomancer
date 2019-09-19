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
use gl::types::{GLenum, GLint, GLsizei, GLuint};
use jpeg_decoder;
use png;
use std::marker::PhantomData;
use std::os::raw::c_void;

//===========================================================================//

pub struct Texture1D {
    id: GLuint,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

impl Texture1D {
    /// Creates a new RGBA texture.  The length of the data array must be a
    /// power of two, with four bytes for each pixel.
    pub fn new_rgba(data: &[u8]) -> Result<Texture1D, String> {
        if data.len() % 4 != 0 {
            return Err(format!(
                "RGBA texture data has length of {}, \
                 which is not a multiple of 4",
                data.len()
            ));
        }
        let width = data.len() / 4;
        if !width.is_power_of_two() {
            return Err(format!(
                "1D texture has width of {}, \
                 which is not a power of 2",
                width
            ));
        }
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_1D, id);
            gl::TexImage1D(
                gl::TEXTURE_1D,
                0,
                gl::RGBA8 as GLint,
                width as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const c_void,
            );
            gl::TexParameteri(
                gl::TEXTURE_1D,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_1D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as GLint,
            );
            gl::GenerateMipmap(gl::TEXTURE_1D);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        return Ok(Texture1D { id, phantom: PhantomData });
    }

    pub(super) fn id(&self) -> GLuint {
        self.id
    }
}

/// Deletes the underlying GL texture when dropped.
impl Drop for Texture1D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//

pub struct Texture2D {
    id: GLuint,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

impl Texture2D {
    pub fn new_rgba(
        width: usize,
        height: usize,
        data: &[u8],
    ) -> Result<Texture2D, String> {
        if !width.is_power_of_two() || !height.is_power_of_two() {
            return Err(format!(
                "2D texture has size of {}x{}, \
                 which are not both powers of 2",
                width, height
            ));
        }
        let expected_data_len = width * height * 4;
        if data.len() != expected_data_len {
            return Err(format!(
                "RGBA texture data has length of {}, \
                 but should be {} for a {}x{} image",
                data.len(),
                expected_data_len,
                width,
                height
            ));
        }
        Ok(Texture2D::new(width, height, data.as_ptr(), (gl::RGBA, gl::RGBA8)))
    }

    pub fn from_jpeg(
        jpeg_name: &str,
        jpeg_data: &[u8],
    ) -> Result<Texture2D, String> {
        let mut decoder = jpeg_decoder::Decoder::new(jpeg_data);
        if let Err(err) = decoder.read_info() {
            return Err(format!(
                "Failed to read JPEG header for {}: {}",
                jpeg_name, err
            ));
        }
        let info = decoder.info().unwrap();
        let format = match info.pixel_format {
            jpeg_decoder::PixelFormat::L8 => (gl::RED, gl::R8),
            jpeg_decoder::PixelFormat::RGB24 => (gl::RGB, gl::RGB8),
            _ => {
                return Err(format!(
                    "JPEG {} has unsupported format: {:?}",
                    jpeg_name, info.pixel_format
                ));
            }
        };
        let width: usize = info.width.into();
        let height: usize = info.height.into();
        if !width.is_power_of_two() || !height.is_power_of_two() {
            return Err(format!(
                "Texture JPEG {} has size of {}x{}, \
                 which are not both powers of 2",
                jpeg_name, width, height
            ));
        }
        let data = decoder.decode().map_err(|err| {
            format!("Failed to decode JPEG data for {}: {}", jpeg_name, err)
        })?;
        return Ok(Texture2D::new(width, height, data.as_ptr(), format));
    }

    /// Creates a new texture from PNG data.  The width and height must each be
    /// a power of two.
    pub fn from_png(
        png_name: &str,
        png_data: &[u8],
    ) -> Result<Texture2D, String> {
        let decoder = png::Decoder::new(png_data);
        let (info, mut reader) = decoder.read_info().map_err(|err| {
            format!("Failed to read PNG header for {}: {}", png_name, err)
        })?;
        if info.bit_depth != png::BitDepth::Eight {
            return Err(format!(
                "PNG {} has unsupported bit depth: {:?}",
                png_name, info.bit_depth
            ));
        }
        let format = match info.color_type {
            png::ColorType::Grayscale => (gl::RED, gl::R8),
            png::ColorType::RGB => (gl::RGB, gl::RGB8),
            png::ColorType::RGBA => (gl::RGBA, gl::RGBA8),
            _ => {
                return Err(format!(
                    "PNG {} has unsupported color type: {:?}",
                    png_name, info.color_type
                ));
            }
        };
        let width = info.width as usize;
        let height = info.height as usize;
        if !width.is_power_of_two() || !height.is_power_of_two() {
            return Err(format!(
                "Texture PNG {} has size of {}x{}, \
                 which are not both powers of 2",
                png_name, width, height
            ));
        }
        let mut data = vec![0u8; info.color_type.samples() * width * height];
        reader.next_frame(&mut data).map_err(|err| {
            format!("Failed to decode PNG data for {}: {}", png_name, err)
        })?;
        return Ok(Texture2D::new(width, height, data.as_ptr(), format));
    }

    pub(super) fn new(
        width: usize,
        height: usize,
        data: *const u8,
        (format, internal_format): (GLenum, GLenum),
    ) -> Texture2D {
        debug_assert!(width.is_power_of_two());
        debug_assert!(height.is_power_of_two());
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as GLint,
                width as GLsizei,
                height as GLsizei,
                0,
                format,
                gl::UNSIGNED_BYTE,
                data as *const c_void,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as GLint,
            );
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        Texture2D { id, phantom: PhantomData }
    }

    pub(super) fn id(&self) -> GLuint {
        self.id
    }
}

/// Deletes the underlying GL texture when dropped.
impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//

pub struct Texture2DMultisample {
    id: GLuint,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

impl Texture2DMultisample {
    pub(super) fn new(
        width: usize,
        height: usize,
        internal_format: GLenum,
    ) -> Texture2DMultisample {
        debug_assert!(width.is_power_of_two());
        debug_assert!(height.is_power_of_two());
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, id);
            gl::TexImage2DMultisample(
                gl::TEXTURE_2D_MULTISAMPLE,
                4,
                internal_format,
                width as GLsizei,
                height as GLsizei,
                gl::FALSE,
            );
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        Texture2DMultisample { id, phantom: PhantomData }
    }

    pub(super) fn id(&self) -> GLuint {
        self.id
    }
}

/// Deletes the underlying GL texture when dropped.
impl Drop for Texture2DMultisample {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//
