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

use crate::geom::{Coords, CoordsSize, Direction, Orientation};
use crate::state::{PortColor, PortConstraint, PortFlow, WireSize};

//===========================================================================//

pub struct ChipData {
    pub ports: &'static [AbstractPort],
    pub constraints: &'static [AbstractConstraint],
    pub dependencies: &'static [(usize, usize)],
}

//===========================================================================//

pub type AbstractPort = (PortFlow, PortColor, (i32, i32), Direction);

pub fn localize(
    coords: Coords,
    orient: Orientation,
    size: CoordsSize,
    port: &AbstractPort,
) -> (Coords, Direction) {
    let &(_, _, delta, dir) = port;
    (coords + orient.transform_in_size(delta.into(), size), orient * dir)
}

//===========================================================================//

pub enum AbstractConstraint {
    /// The port must be the given size.
    Exact(usize, WireSize),
    /// The port must be no bigger than the given size.
    AtMost(usize, WireSize),
    /// The port must be no smaller than the given size.
    AtLeast(usize, WireSize),
    /// The two ports must be the same size.
    Equal(usize, usize),
    /// The first port must be double the size of the second port.
    Double(usize, usize),
}

impl AbstractConstraint {
    pub fn reify(
        &self,
        coords: Coords,
        orient: Orientation,
        size: CoordsSize,
        ports: &[AbstractPort],
    ) -> PortConstraint {
        match *self {
            AbstractConstraint::Exact(index, wsize) => {
                let loc = localize(coords, orient, size, &ports[index]);
                PortConstraint::Exact(loc, wsize)
            }
            AbstractConstraint::AtMost(index, wsize) => {
                let loc = localize(coords, orient, size, &ports[index]);
                PortConstraint::AtMost(loc, wsize)
            }
            AbstractConstraint::AtLeast(index, wsize) => {
                let loc = localize(coords, orient, size, &ports[index]);
                PortConstraint::AtLeast(loc, wsize)
            }
            AbstractConstraint::Equal(index1, index2) => {
                let loc1 = localize(coords, orient, size, &ports[index1]);
                let loc2 = localize(coords, orient, size, &ports[index2]);
                PortConstraint::Equal(loc1, loc2)
            }
            AbstractConstraint::Double(index1, index2) => {
                let loc1 = localize(coords, orient, size, &ports[index1]);
                let loc2 = localize(coords, orient, size, &ports[index2]);
                PortConstraint::Double(loc1, loc2)
            }
        }
    }
}

//===========================================================================//
