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

use super::texture::{Texture2D, Texture2DMultisample};
use gl;
use gl::types::{GLsizei, GLuint};
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use tachy::geom::RectSize;

//===========================================================================//

pub struct FrameBuffer {
    id: GLuint,
    size: RectSize<usize>,
    _texture: Texture2D,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

assert_not_impl_any!(FrameBuffer: Send, Sync);

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> FrameBuffer {
        let texture_width = width.next_power_of_two();
        let texture_height = height.next_power_of_two();
        let color_texture = Texture2D::new(
            texture_width,
            texture_height,
            ptr::null(),
            (gl::RGB, gl::RGB8),
        );
        let mut id: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                color_texture.id(),
                0,
            );
            debug_assert_eq!(
                gl::CheckFramebufferStatus(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // unbind
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        FrameBuffer {
            id,
            size: RectSize::new(width, height),
            _texture: color_texture,
            phantom: PhantomData,
        }
    }

    pub fn bind(&mut self, window_size: RectSize<i32>) -> FrameBufferBinding {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            gl::Viewport(
                0,
                0,
                self.size.width as GLsizei,
                self.size.height as GLsizei,
            );
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        FrameBufferBinding { window_size, fbo: self }
    }
}

/// Deletes the underlying GL framebuffer when dropped.
impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//

pub struct FrameBufferBinding<'a> {
    window_size: RectSize<i32>,
    fbo: &'a FrameBuffer,
}

impl<'a> FrameBufferBinding<'a> {
    pub fn read_rgb_data(&self) -> Vec<u8> {
        let num_bytes = 3 * self.fbo.size.width * self.fbo.size.height;
        let mut data = Vec::with_capacity(num_bytes);
        unsafe {
            gl::ReadPixels(
                0,
                0,
                self.fbo.size.width as GLsizei,
                self.fbo.size.height as GLsizei,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr() as *mut c_void,
            );
            data.set_len(num_bytes);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        data
    }

    pub fn unbind(self) {}
}

impl<'a> Drop for FrameBufferBinding<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(
                0,
                0,
                self.window_size.width,
                self.window_size.height,
            );
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//

pub struct FrameBufferMultisample {
    id: GLuint,
    size: RectSize<usize>,
    texture_size: RectSize<usize>,
    texture: Texture2DMultisample,
    _depth_texture: Option<Texture2DMultisample>,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

assert_not_impl_any!(FrameBufferMultisample: Send, Sync);

impl FrameBufferMultisample {
    pub fn new(
        width: usize,
        height: usize,
        depth: bool,
    ) -> FrameBufferMultisample {
        let texture_width = width.next_power_of_two();
        let texture_height = height.next_power_of_two();
        let color_texture = Texture2DMultisample::new(
            texture_width,
            texture_height,
            gl::RGBA8,
        );
        let depth_texture = if depth {
            Some(Texture2DMultisample::new(
                texture_width,
                texture_height,
                gl::DEPTH_COMPONENT,
            ))
        } else {
            None
        };
        let mut id: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D_MULTISAMPLE,
                color_texture.id(),
                0,
            );
            if let Some(ref texture) = depth_texture {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_ATTACHMENT,
                    gl::TEXTURE_2D_MULTISAMPLE,
                    texture.id(),
                    0,
                );
            }
            debug_assert_eq!(
                gl::CheckFramebufferStatus(gl::FRAMEBUFFER),
                gl::FRAMEBUFFER_COMPLETE
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // unbind
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        FrameBufferMultisample {
            id,
            size: RectSize::new(width, height),
            texture_size: RectSize::new(texture_width, texture_height),
            texture: color_texture,
            _depth_texture: depth_texture,
            phantom: PhantomData,
        }
    }

    pub fn size(&self) -> RectSize<usize> {
        self.size
    }

    pub fn texture_size(&self) -> RectSize<usize> {
        self.texture_size
    }

    pub fn texture(&self) -> &Texture2DMultisample {
        &self.texture
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            gl::Viewport(
                0,
                0,
                self.size.width as GLsizei,
                self.size.height as GLsizei,
            );
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }

    pub fn unbind(&self, window_size: RectSize<i32>) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, window_size.width, window_size.height);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

/// Deletes the underlying GL framebuffer when dropped.
impl Drop for FrameBufferMultisample {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//
