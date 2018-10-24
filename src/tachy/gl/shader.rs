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
use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use std::ptr;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum ShaderType {
    Fragment,
    Vertex,
}

impl ShaderType {
    fn to_gl_enum(self) -> GLenum {
        match self {
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::Vertex => gl::VERTEX_SHADER,
        }
    }
}

//===========================================================================//

/// Represents a GL shader.
pub struct Shader {
    pub(super) name: GLuint,
}

impl Shader {
    pub fn new(shader_type: ShaderType, name: &str, code: &[u8])
               -> Result<Shader, String> {
        let shader = unsafe {
            let shader =
                Shader { name: gl::CreateShader(shader_type.to_gl_enum()) };
            gl::ShaderSource(shader.name,
                             1,
                             &(code.as_ptr() as *const GLchar),
                             &(code.len() as GLint));
            gl::CompileShader(shader.name);
            let mut result: GLint = 0;
            gl::GetShaderiv(shader.name, gl::COMPILE_STATUS, &mut result);
            if result != (gl::TRUE as GLint) {
                return Err(format!("Error compiling {}:\n{}",
                                   name,
                                   shader.get_info_log()));
            }
            shader
        };
        if cfg!(debug_assertions) {
            let log = shader.get_info_log();
            if !log.is_empty() {
                debug_log!("Info log for {}:\n{}", name, log);
            }
        }
        Ok(shader)
    }

    pub fn get_info_log(&self) -> String {
        let mut length: GLint = 0;
        unsafe {
            gl::GetShaderiv(self.name, gl::INFO_LOG_LENGTH, &mut length);
        }
        if length > 0 {
            let mut buffer = vec![0u8; length as usize + 1];
            unsafe {
                gl::GetShaderInfoLog(self.name,
                                     buffer.len() as GLsizei,
                                     ptr::null_mut(),
                                     buffer.as_mut_ptr() as *mut GLchar);
            }
            String::from_utf8_lossy(&buffer).to_string()
        } else {
            String::new()
        }
    }
}

/// Deletes the underlying GL shader when dropped.
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.name);
        }
    }
}

// TODO: impl !Send for Shader {}

// TODO: impl !Sync for Shader {}

//===========================================================================//
