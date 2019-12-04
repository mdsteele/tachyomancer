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

use super::super::interface::{Interface, InterfacePort, InterfacePosition};
use super::shared::{FabricationData, TutorialBubblePosition, NIL};
use crate::geom::Direction;
use crate::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const TUTORIAL_RAM_DATA: &FabricationData = &FabricationData {
    interfaces: RAM_INTERFACES,
    expected_table_values: RAM_EXPECTED_TABLE_VALUES,
};

pub(super) const RAM_INTERFACES: &[Interface] = &[
    Interface {
        name: "Push",
        description: "TODO",
        side: Direction::West,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Push",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Pop",
        description: "TODO",
        side: Direction::West,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Pop",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Out",
        description: "TODO",
        side: Direction::East,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const RAM_EXPECTED_TABLE_VALUES: &[u32] = &[
      5, NIL, NIL,
      3, NIL, NIL,
     12, NIL, NIL,
    NIL,   0,  12,
    NIL,   0,   3,
    NIL,   0,   5,
    NIL,   0, NIL,
     12, NIL, NIL,
     14, NIL, NIL,
      9, NIL, NIL,
      4, NIL, NIL,
    NIL,   0,   4,
    NIL,   0,   9,
      2, NIL, NIL,
    NIL,   0,   2,
    NIL,   0,  14,
      9,   0,   9,
    NIL,   0,  12,
    NIL,   0, NIL,
      1,   0,   1,
];

pub(super) const RAM_BUBBLES: &[(TutorialBubblePosition, &str)] =
    &[(TutorialBubblePosition::Bounds(Direction::North), "TODO")];

//===========================================================================//
