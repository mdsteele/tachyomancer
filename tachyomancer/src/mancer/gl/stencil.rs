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

pub struct Stencil {
    // This PhantomData ensures that this struct is not Send or Sync, which
    // helps ensure that we keep all our OpenGL stuff on the main thread.
    phantom: PhantomData<*mut ()>,
}

assert_not_impl_any!(Stencil: Send, Sync);

impl Stencil {
    /// Clears the stencil buffer, and enables the stencil test until the
    /// returned object is dropped.  At most one `Stencil` object should exist
    /// at once.
    pub fn new() -> Stencil {
        unsafe {
            gl::Clear(gl::STENCIL_BUFFER_BIT);
            gl::Enable(gl::STENCIL_TEST);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
        let stencil = Stencil { phantom: PhantomData };
        stencil.enable_updates();
        stencil
    }

    /// After calling this, future draw calls will expand the stencil area.
    /// The `Stencil` starts in update mode.
    pub fn enable_updates(&self) {
        unsafe {
            gl::StencilFunc(gl::ALWAYS, 1, 0x1);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }

    /// After calling this, future draw calls will be clipped by the stencil
    /// area.
    pub fn enable_clipping(&self) {
        unsafe {
            gl::StencilFunc(gl::EQUAL, 1, 0x1);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }

    pub fn disable(self) {}
}

/// Disables the stencil test when dropped.
impl Drop for Stencil {
    fn drop(&mut self) {
        unsafe {
            gl::Disable(gl::STENCIL_TEST);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }
    }
}

//===========================================================================//
