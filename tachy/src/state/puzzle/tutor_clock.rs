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

pub const TUTORIAL_CLOCK_DATA: &FabricationData = &FabricationData {
    interfaces: CLOCK_INTERFACES,
    expected_table_values: CLOCK_EXPECTED_TABLE_VALUES,
};

pub(super) const CLOCK_INTERFACES: &[Interface] = &[
    Interface {
        name: "In",
        description: "Input events arrive here.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Out",
        description: "Output events should be sent here.",
        side: Direction::East,
        pos: InterfacePosition::Center,
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
const CLOCK_EXPECTED_TABLE_VALUES: &[u32] = &[
    NIL, NIL,
      5, NIL,
      3, NIL,
    NIL,   5,
      4, NIL,
    NIL, NIL,
      7,   4,
     14, NIL,
    NIL,   7,
    NIL, NIL,
      9, NIL,
      8, NIL,
      2,   9,
     12, NIL,
     15,   2,
      0, NIL,
      6,  15,
    NIL, NIL,
    NIL,   6,
];

pub(super) const CLOCK_BUBBLES: &[(TutorialBubblePosition, &str)] = &[(
    TutorialBubblePosition::Bounds(Direction::North),
    "A $*Clock$* chip can be used together with a $*Latest$* chip to \
     delay a value by one time step.",
)];

//===========================================================================//
