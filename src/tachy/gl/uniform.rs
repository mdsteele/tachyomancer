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

use cgmath::{Matrix4, Vector2, Vector3, Vector4};
use gl;
use gl::types::{GLenum, GLint};
use std::marker::PhantomData;
use tachy::geom::{Color4, Rect};

//===========================================================================//

pub struct ShaderUniform<T> {
    loc: GLint,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut T>,
}

impl<T: UniformValue> ShaderUniform<T> {
    pub(super) fn new(loc: GLint) -> ShaderUniform<T> {
        ShaderUniform {
            loc,
            phantom: PhantomData,
        }
    }

    pub fn set(&self, value: &T) { value.set_uniform(self.loc) }
}

//===========================================================================//

pub trait UniformValue {
    fn gl_type() -> GLenum;
    fn array_size() -> GLint { 1 }
    fn set_uniform(&self, loc: GLint);
}

impl UniformValue for Color4 {
    fn gl_type() -> GLenum { gl::FLOAT_VEC4 }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform4f(loc, self.r, self.g, self.b, self.a);
        }
    }
}

impl UniformValue for f32 {
    fn gl_type() -> GLenum { gl::FLOAT }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform1f(loc, *self);
        }
    }
}

impl UniformValue for Matrix4<f32> {
    fn gl_type() -> GLenum { gl::FLOAT_MAT4 }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, &self[0][0]);
        }
    }
}

impl UniformValue for Rect<f32> {
    fn gl_type() -> GLenum { gl::FLOAT_VEC4 }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform4f(loc, self.x, self.y, self.width, self.height);
        }
    }
}

impl UniformValue for Vector2<u32> {
    fn gl_type() -> GLenum { gl::UNSIGNED_INT_VEC2 }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform2ui(loc, self.x, self.y);
        }
    }
}

impl UniformValue for Vector3<f32> {
    fn gl_type() -> GLenum { gl::FLOAT_VEC3 }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform3f(loc, self.x, self.y, self.z);
        }
    }
}

impl UniformValue for Vector4<f32> {
    fn gl_type() -> GLenum { gl::FLOAT_VEC4 }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform4f(loc, self.x, self.y, self.z, self.w);
        }
    }
}

impl UniformValue for [u32; 64] {
    fn gl_type() -> GLenum { gl::UNSIGNED_INT }
    fn array_size() -> GLint { 64 }
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform1uiv(loc, 64, self.as_ptr());
        }
    }
}

//===========================================================================//
