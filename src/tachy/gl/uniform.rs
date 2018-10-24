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

use cgmath::Matrix4;
use gl;
use gl::types::GLint;
use std::marker::PhantomData;

//===========================================================================//

pub struct ShaderUniform<T> {
    phantom: PhantomData<T>,
    loc: GLint,
}

impl<T: UniformValue> ShaderUniform<T> {
    pub(super) fn new(loc: GLint) -> ShaderUniform<T> {
        ShaderUniform {
            phantom: PhantomData,
            loc,
        }
    }

    pub fn set(&self, value: &T) { value.set_uniform(self.loc) }
}

//===========================================================================//

pub trait UniformValue {
    fn set_uniform(&self, loc: GLint);
}

impl UniformValue for (f32, f32, f32) {
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::Uniform3f(loc, self.0, self.1, self.2);
        }
    }
}

impl UniformValue for Matrix4<f32> {
    fn set_uniform(&self, loc: GLint) {
        unsafe {
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, &self[0][0]);
        }
    }
}

//===========================================================================//
