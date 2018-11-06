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
use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid};
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum Primitive {
    Triangles,
    TriangleStrip,
    TriangleFan,
}

impl Primitive {
    fn to_gl_enum(self) -> GLenum {
        match self {
            Primitive::Triangles => gl::TRIANGLES,
            Primitive::TriangleStrip => gl::TRIANGLE_STRIP,
            Primitive::TriangleFan => gl::TRIANGLE_FAN,
        }
    }
}

//===========================================================================//

pub struct VertexArray {
    name: GLuint,
}

impl VertexArray {
    pub fn new(size: GLuint) -> VertexArray {
        let mut name: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
            for index in 0..size {
                gl::EnableVertexAttribArray(index);
            }
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        VertexArray { name }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.name);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }

    pub fn draw(&self, primitive: Primitive, first: usize, count: usize) {
        unsafe {
            gl::DrawArrays(primitive.to_gl_enum(),
                           first as GLint,
                           count as GLsizei);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

/// Deletes the underlying GL vertex array when dropped.
impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.name);
        }
    }
}

//===========================================================================//

pub struct VertexBuffer<A> {
    phantom: PhantomData<A>,
    name: GLuint,
}

impl<A: VertexAtom> VertexBuffer<A> {
    pub fn new(data: &[A]) -> VertexBuffer<A> {
        let mut name: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut name);
            gl::BindBuffer(gl::ARRAY_BUFFER, name);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (mem::size_of::<A>() * data.len()) as GLsizeiptr,
                           data.as_ptr() as *const c_void,
                           gl::STATIC_DRAW);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        VertexBuffer {
            phantom: PhantomData,
            name,
        }
    }

    pub fn attribf(&self, attrib_index: GLuint, atoms_per_vertex: GLint,
                   stride: usize, offset: usize) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.name);
            gl::VertexAttribPointer(attrib_index,
                                    atoms_per_vertex,
                                    A::gl_type(),
                                    gl::FALSE,
                                    (mem::size_of::<A>() * stride) as GLsizei,
                                    (mem::size_of::<A>() * offset) as
                                        *const GLvoid);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

impl<A: VertexIntAtom> VertexBuffer<A> {
    pub fn attribi(&self, attrib_index: GLuint, atoms_per_vertex: GLint,
                   stride: usize, offset: usize) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.name);
            gl::VertexAttribIPointer(attrib_index,
                                     atoms_per_vertex,
                                     A::gl_type(),
                                     (mem::size_of::<A>() * stride) as
                                         GLsizei,
                                     (mem::size_of::<A>() * offset) as
                                         *const GLvoid);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }

    #[allow(dead_code)]
    pub fn attribn(&self, attrib_index: GLuint, atoms_per_vertex: GLint,
                   stride: usize, offset: usize) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.name);
            gl::VertexAttribPointer(attrib_index,
                                    atoms_per_vertex,
                                    A::gl_type(),
                                    gl::TRUE,
                                    (mem::size_of::<A>() * stride) as GLsizei,
                                    (mem::size_of::<A>() * offset) as
                                        *const GLvoid);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

/// Deletes the underlying GL buffer when dropped.
impl<A> Drop for VertexBuffer<A> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.name);
        }
    }
}

//===========================================================================//

pub trait VertexAtom {
    fn gl_type() -> GLenum;
}

pub trait VertexIntAtom: VertexAtom {}

impl VertexAtom for f32 {
    fn gl_type() -> GLenum { gl::FLOAT }
}

impl VertexAtom for i8 {
    fn gl_type() -> GLenum { gl::BYTE }
}
impl VertexIntAtom for i8 {}

impl VertexAtom for u8 {
    fn gl_type() -> GLenum { gl::UNSIGNED_BYTE }
}
impl VertexIntAtom for u8 {}

//===========================================================================//
