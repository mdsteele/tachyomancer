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

mod cast;
mod color;
mod coords;
mod dir;
mod matrix;
mod orient;
mod polygon;
mod rect;

pub use self::cast::{AsFloat, AsInt};
pub use self::color::{Color3, Color4};
pub use self::coords::{Coords, CoordsDelta, CoordsRect, CoordsSize};
pub use self::dir::{Direction, DirectionIter};
pub use self::matrix::MatrixExt;
pub use self::orient::Orientation;
pub use self::polygon::{Polygon, PolygonRef};
pub use self::rect::{Rect, RectPointsIter, RectSize};

//===========================================================================//
