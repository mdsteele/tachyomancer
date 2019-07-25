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

use super::texture::{Texture1D, Texture2D, Texture2DMultisample};
use super::uniform::{ShaderUniform, UniformValue};
use gl;
use gl::types::{GLenum, GLint, GLuint};
use std::marker::PhantomData;

//===========================================================================//

pub struct ShaderSampler<T> {
    slot: GLuint,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut T>,
}

impl<T: SamplerValue> ShaderSampler<T> {
    pub(super) fn new(uniform: ShaderUniform<T::UniformType>, slot: GLuint)
                      -> ShaderSampler<T> {
        uniform.set(&T::uniform_value(slot));
        ShaderSampler {
            slot,
            phantom: PhantomData,
        }
    }

    pub fn set(&self, value: &T) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + self.slot);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        value.set_sampler();
    }
}

//===========================================================================//

pub trait SamplerValue {
    type UniformType: UniformValue;
    fn uniform_value(slot: GLuint) -> Self::UniformType;
    fn set_sampler(&self);
}

//===========================================================================//

pub struct Texture1DSlot(GLint);

impl UniformValue for Texture1DSlot {
    fn gl_type() -> GLenum { gl::SAMPLER_1D }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform1i(loc, self.0);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

impl SamplerValue for Texture1D {
    type UniformType = Texture1DSlot;

    fn uniform_value(slot: GLuint) -> Texture1DSlot {
        Texture1DSlot(slot as GLint)
    }

    fn set_sampler(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_1D, self.id());
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//

pub struct Texture2DSlot(GLint);

impl UniformValue for Texture2DSlot {
    fn gl_type() -> GLenum { gl::SAMPLER_2D }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform1i(loc, self.0);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

impl SamplerValue for Texture2D {
    type UniformType = Texture2DSlot;

    fn uniform_value(slot: GLuint) -> Texture2DSlot {
        Texture2DSlot(slot as GLint)
    }

    fn set_sampler(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id());
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//

pub struct Texture2DMultisampleSlot(GLint);

impl UniformValue for Texture2DMultisampleSlot {
    fn gl_type() -> GLenum { gl::SAMPLER_2D_MULTISAMPLE }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform1i(loc, self.0);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

impl SamplerValue for Texture2DMultisample {
    type UniformType = Texture2DMultisampleSlot;

    fn uniform_value(slot: GLuint) -> Texture2DMultisampleSlot {
        Texture2DMultisampleSlot(slot as GLint)
    }

    fn set_sampler(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, self.id());
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//
