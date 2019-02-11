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
use gl::types::{GLenum, GLsizeiptr, GLuint};
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;

//===========================================================================//

pub struct IndexBuffer<A> {
    name: GLuint,
    len: usize,
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut A>,
}

impl<A: IndexAtom> IndexBuffer<A> {
    pub fn new(data: &[A]) -> IndexBuffer<A> {
        let mut name: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut name);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, name);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (mem::size_of::<A>() * data.len()) as GLsizeiptr,
                           data.as_ptr() as *const c_void,
                           gl::STATIC_DRAW);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        IndexBuffer {
            name,
            len: data.len(),
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize { self.len }

    pub(super) fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.name);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

/// Deletes the underlying GL buffer when dropped.
impl<A> Drop for IndexBuffer<A> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.name);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//

pub trait IndexAtom {
    fn gl_type() -> GLenum;
}

impl IndexAtom for u8 {
    fn gl_type() -> GLenum { gl::UNSIGNED_BYTE }
}

impl IndexAtom for u16 {
    fn gl_type() -> GLenum { gl::UNSIGNED_SHORT }
}

impl IndexAtom for u32 {
    fn gl_type() -> GLenum { gl::UNSIGNED_INT }
}

//===========================================================================//
