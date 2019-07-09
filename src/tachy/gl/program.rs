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

use super::shader::Shader;
use super::uniform::{ShaderUniform, UniformValue};
use gl;
use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

//===========================================================================//

pub struct ShaderProgram {
    id: GLuint,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

impl ShaderProgram {
    pub fn new(shaders: &[&Shader]) -> Result<ShaderProgram, String> {
        unsafe {
            let program = ShaderProgram {
                id: gl::CreateProgram(),
                phantom: PhantomData,
            };
            for shader in shaders.iter() {
                gl::AttachShader(program.id, shader.id);
            }
            gl::LinkProgram(program.id);
            for shader in shaders.iter() {
                gl::DetachShader(program.id, shader.id);
            }
            let mut result: GLint = 0;
            gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut result);
            if result != (gl::TRUE as GLint) {
                return Err(program.get_info_log());
            }
            Ok(program)
        }
    }

    pub fn get_info_log(&self) -> String {
        let mut length: GLint = 0;
        unsafe {
            gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut length);
        }
        if length > 0 {
            let mut buffer = vec![0u8; length as usize + 1];
            unsafe {
                gl::GetProgramInfoLog(self.id,
                                      buffer.len() as GLsizei,
                                      ptr::null_mut(),
                                      buffer.as_mut_ptr() as *mut GLchar);
            }
            String::from_utf8_lossy(&buffer).to_string()
        } else {
            String::new()
        }
    }

    /// Sets this as the current shader program.
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn get_uniform<T>(&self, name: &str)
                          -> Result<ShaderUniform<T>, String>
    where
        T: UniformValue,
    {
        // Get the location for the uniform (and make sure it exists):
        let cstring =
            CString::new(name)
                .map_err(|err| {
                             format!("Invalid uniform name {:?}: {}",
                                     name,
                                     err)
                         })?;
        let loc = unsafe { gl::GetUniformLocation(self.id, cstring.as_ptr()) };
        if loc < 0 {
            return Err(format!("No uniform named {:?}", name));
        }

        // Make sure that the actual type of the uniform, as declared in the
        // shader code, matches the type we're ascribing to it.
        let mut array_size: GLint = 0;
        let mut gl_type: GLenum = 0;
        unsafe {
            let mut index: GLuint = 0;
            gl::GetUniformIndices(self.id, 1, &cstring.as_ptr(), &mut index);
            gl::GetActiveUniform(self.id,
                                 index,
                                 0,
                                 ptr::null_mut(),
                                 &mut array_size,
                                 &mut gl_type,
                                 ptr::null_mut());
        }
        if gl_type != T::gl_type() {
            return Err(format!("Uniform {:?} actually has type {}, not {}",
                               name,
                               gl_type_name(gl_type),
                               gl_type_name(T::gl_type())));
        }
        if array_size != T::array_size() {
            return Err(format!("Uniform {:?} actually has size {}, not {}",
                               name,
                               array_size,
                               T::array_size()));
        }

        return Ok(ShaderUniform::new(loc));
    }
}

/// Deletes the underlying GL shader when dropped.
impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

//===========================================================================//

fn gl_type_name(gl_type: GLenum) -> String {
    if gl_type == gl::FLOAT {
        "FLOAT".to_string()
    } else if gl_type == gl::FLOAT_MAT4 {
        "FLOAT_MAT4".to_string()
    } else if gl_type == gl::FLOAT_VEC2 {
        "FLOAT_VEC2".to_string()
    } else if gl_type == gl::FLOAT_VEC3 {
        "FLOAT_VEC3".to_string()
    } else if gl_type == gl::FLOAT_VEC4 {
        "FLOAT_VEC4".to_string()
    } else if gl_type == gl::INT {
        "INT".to_string()
    } else if gl_type == gl::UNSIGNED_INT {
        "UNSIGNED_INT".to_string()
    } else if gl_type == gl::UNSIGNED_INT_VEC2 {
        "UNSIGNED_INT_VEC2".to_string()
    } else {
        format!("({})", gl_type)
    }
}

//===========================================================================//
