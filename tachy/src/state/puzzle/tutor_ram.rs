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
const RAM_EXPECTED_TABLE_VALUES: &[u32] = &[
      7,   7,
      2,   2,
      7, NIL,
     14,  14,
    NIL, NIL,
      9,   9,
      2, NIL,
     10,  10,
      0,   0,
      2, NIL,
      1,   1,
     10, NIL,
      0, NIL,
      5,   5,
      8,   8,
];

pub(super) const RAM_BUBBLES: &[(TutorialBubblePosition, &str)] = &[(
    TutorialBubblePosition::PartsTray,
    "$*Ram$* chips can be found in the \"Memory\" section of the parts \
     tray.",
)];

//===========================================================================//
