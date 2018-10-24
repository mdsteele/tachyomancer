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
use gl::types::{GLchar, GLint, GLsizei, GLuint};
use std::ffi::CString;
use std::ptr;

//===========================================================================//

pub struct ShaderProgram {
    name: GLuint,
}

impl ShaderProgram {
    pub fn new(shaders: &[&Shader]) -> Result<ShaderProgram, String> {
        unsafe {
            let program = ShaderProgram { name: gl::CreateProgram() };
            for shader in shaders.iter() {
                gl::AttachShader(program.name, shader.name);
            }
            gl::LinkProgram(program.name);
            for shader in shaders.iter() {
                gl::DetachShader(program.name, shader.name);
            }
            let mut result: GLint = 0;
            gl::GetProgramiv(program.name, gl::LINK_STATUS, &mut result);
            if result != (gl::TRUE as GLint) {
                return Err(program.get_info_log());
            }
            Ok(program)
        }
    }

    pub fn get_info_log(&self) -> String {
        let mut length: GLint = 0;
        unsafe {
            gl::GetProgramiv(self.name, gl::INFO_LOG_LENGTH, &mut length);
        }
        if length > 0 {
            let mut buffer = vec![0u8; length as usize + 1];
            unsafe {
                gl::GetProgramInfoLog(self.name,
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
            gl::UseProgram(self.name);
        }
    }

    pub fn get_uniform<T: UniformValue>(
        &self, name: &str)
        -> Result<ShaderUniform<T>, String> {
        let cstring =
            CString::new(name)
                .map_err(|err| {
                             format!("Invalid uniform name {:?}: {}",
                                     name,
                                     err)
                         })?;
        let loc =
            unsafe { gl::GetUniformLocation(self.name, cstring.as_ptr()) };
        if loc < 0 {
            return Err(format!("No uniform named {:?}", name));
        }
        // TODO: Use glGetActiveUniform to check that the type of the uniform
        //   matches T.
        return Ok(ShaderUniform::new(loc));
    }
}

/// Deletes the underlying GL shader when dropped.
impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.name);
        }
    }
}

// TODO: impl !Send for ShaderProgram {}

// TODO: impl !Sync for ShaderProgram {}

//===========================================================================//
