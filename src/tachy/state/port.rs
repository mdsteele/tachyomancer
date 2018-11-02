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

use super::geom::{CoordsDelta, Direction};

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PortFlow {
    Send,
    Recv,
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PortColor {
    Behavior,
    Event,
}

//===========================================================================//

pub struct PortSpec {
    pub flow: PortFlow,
    pub color: PortColor,
    pub pos: CoordsDelta, // relative to top-left of chip
    pub dir: Direction,
}

impl PortSpec {
    pub fn brecv(direction: Direction) -> PortSpec {
        PortSpec {
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            pos: (0, 0).into(),
            dir: direction,
        }
    }

    pub fn bsend(direction: Direction) -> PortSpec {
        PortSpec {
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            pos: (0, 0).into(),
            dir: direction,
        }
    }

    pub fn erecv(direction: Direction) -> PortSpec {
        PortSpec {
            flow: PortFlow::Recv,
            color: PortColor::Event,
            pos: (0, 0).into(),
            dir: direction,
        }
    }

    pub fn esend(direction: Direction) -> PortSpec {
        PortSpec {
            flow: PortFlow::Send,
            color: PortColor::Event,
            pos: (0, 0).into(),
            dir: direction,
        }
    }
}

//===========================================================================//
