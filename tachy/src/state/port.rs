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

use crate::geom::{Coords, Direction};
use crate::save::WireSize;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PortFlow {
    Source,
    Sink,
}

impl PortFlow {
    pub fn tooltip_format(self) -> &'static str {
        match self {
            PortFlow::Source => "source",
            PortFlow::Sink => "sink",
        }
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PortColor {
    Behavior,
    Event,
    Analog,
}

impl PortColor {
    pub fn tooltip_format(self) -> &'static str {
        match self {
            PortColor::Behavior => "$Obehavior$D",
            PortColor::Event => "$Cevent$D",
            PortColor::Analog => "$Ganalog$D",
        }
    }
}

//===========================================================================//

pub struct PortSpec {
    pub flow: PortFlow,
    pub color: PortColor,
    pub coords: Coords,
    pub dir: Direction,
    pub max_size: WireSize,
}

impl PortSpec {
    pub fn loc(&self) -> (Coords, Direction) {
        (self.coords, self.dir)
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug)]
pub enum PortConstraint {
    /// The port must be the given size.
    Exact((Coords, Direction), WireSize),
    /// The port must be no bigger than the given size.
    AtMost((Coords, Direction), WireSize),
    /// The port must be no smaller than the given size.
    AtLeast((Coords, Direction), WireSize),
    /// The two ports must be the same size.
    Equal((Coords, Direction), (Coords, Direction)),
    /// The first port must be double the size of the second port.
    Double((Coords, Direction), (Coords, Direction)),
}

//===========================================================================//

/// Indicates that output of the specified source port depends on the value of
/// the specified sink port.
pub struct PortDependency {
    pub sink: (Coords, Direction),
    pub source: (Coords, Direction),
}

//===========================================================================//
