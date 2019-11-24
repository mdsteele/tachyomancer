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
use std::marker::PhantomData;

//===========================================================================//

pub struct Depth {
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

assert_not_impl_any!(Depth: Send, Sync);

impl Depth {
    /// Clears the depth buffer, and enables the depth test and optional face
    /// culling until the returned object is dropped.  At most one `Depth`
    /// object should exist at once.
    pub fn enable_with_face_culling(cull: bool) -> Depth {
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
            if cull {
                gl::Enable(gl::CULL_FACE);
            }
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        Depth { phantom: PhantomData }
    }

    pub fn disable(self) {}
}

/// Disables the depth test and face culling when dropped.
impl Drop for Depth {
    fn drop(&mut self) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//
