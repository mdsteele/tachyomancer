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
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow};

//===========================================================================//

pub const TUTORIAL_RAM_DATA: &FabricationData = &FabricationData {
    interfaces: RAM_INTERFACES,
    expected_table_values: RAM_EXPECTED_TABLE_VALUES,
};

pub(super) const RAM_INTERFACES: &[Interface] = &[
    Interface {
        name: "Push",
        description:
            "When an event arrives here, that value should be pushed onto \
             the top of the stack.",
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
        description:
            "When an event arrives here, the top value of the stack should \
             be popped off and sent to the $*Out$* port.",
        side: Direction::East,
        pos: InterfacePosition::Left(0),
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
        description: "Values popped off the stack should be sent here.",
        side: Direction::East,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Count",
        description:
            "Should be equal to the number of values currently in the stack.",
        side: Direction::West,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Count",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const RAM_EXPECTED_TABLE_VALUES: &[u32] = &[
      5, NIL, NIL, 1,
      3, NIL, NIL, 2,
     12, NIL, NIL, 3,
    NIL,   0,  12, 2,
    NIL,   0,   3, 1,
    NIL,   0,   5, 0,
    NIL,   0, NIL, 0,
     12, NIL, NIL, 1,
     14, NIL, NIL, 2,
      9, NIL, NIL, 3,
      4, NIL, NIL, 4,
    NIL,   0,   4, 3,
    NIL,   0,   9, 2,
      2, NIL, NIL, 3,
    NIL,   0,   2, 2,
    NIL,   0,  14, 1,
      9,   0,   9, 1,
    NIL,   0,  12, 0,
    NIL,   0, NIL, 0,
      1,   0,   1, 0,
];

pub(super) const RAM_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::PartsTray,
        "$*Ram$* chips can be found in the \"Memory\" section of the parts \
         tray.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::South),
        "If necessary, a $*Coerce$* chip can be used to set the address size \
         for a $*Ram$* chip.",
    ),
];

//===========================================================================//
