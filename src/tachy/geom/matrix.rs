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

use cgmath::{BaseFloat, Matrix4, SquareMatrix, Vector2, vec3};

//===========================================================================//

pub trait MatrixExt: SquareMatrix
where
    Self::Scalar: BaseFloat,
{
    fn scale2(x: Self::Scalar, y: Self::Scalar) -> Self;

    fn trans2(x: Self::Scalar, y: Self::Scalar) -> Self;

    fn trans2v(v: Vector2<Self::Scalar>) -> Self { Self::trans2(v.x, v.y) }
}

impl<S: BaseFloat> MatrixExt for Matrix4<S> {
    fn scale2(x: S, y: S) -> Matrix4<S> {
        Matrix4::from_nonuniform_scale(x, y, S::one())
    }

    fn trans2(x: S, y: S) -> Matrix4<S> {
        Matrix4::from_translation(vec3(x, y, S::zero()))
    }
}

//===========================================================================//
