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

use super::texture::Texture2DMultisample;
use crate::tachy::geom::RectSize;
use gl;
use gl::types::GLuint;
use std::marker::PhantomData;

//===========================================================================//

pub struct FrameBuffer {
    id: GLuint,
    size: RectSize<usize>,
    texture_size: RectSize<usize>,
    texture: Texture2DMultisample,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> FrameBuffer {
        let texture_width = width.next_power_of_two();
        let texture_height = height.next_power_of_two();
        let texture = Texture2DMultisample::new(
            texture_width,
            texture_height,
            gl::RGBA8,
        );
        let mut id: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D_MULTISAMPLE,
                texture.id(),
                0,
            );
            assert_eq!(
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
            texture_size: RectSize::new(texture_width, texture_height),
            texture,
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
                self.size.width as i32,
                self.size.height as i32,
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
impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//
