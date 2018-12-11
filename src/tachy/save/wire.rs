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

//===========================================================================//

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum WireShape {
    /// Wire enters from side of cell but stops immediately.
    Stub,
    /// Wire enters from side of cell and goes straight to the other side.  The
    /// opposite side will also be `Straight`.
    Straight,
    /// Wire enters from side of cell and turns 90 degrees left.  The adjacent
    /// side will be `TurnRight`.
    TurnLeft,
    /// Wire enters from side of cell and turns 90 degrees right.  The adjacent
    /// side will be `TurnLeft`.
    TurnRight,
    /// Wire enters from side of cell and splits, going straight and turning
    /// left.
    SplitLeft,
    /// Wire enters from side of cell and splits, going straight and turning
    /// right.
    SplitRight,
    /// Wire enters from side of cell and splits, turning left and right.
    SplitTee,
    /// Wire enters from side of cell and splits in all directions.
    Cross,
}

//===========================================================================//
