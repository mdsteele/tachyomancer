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
use super::shared::{FabricationData, TutorialBubblePosition};
use crate::geom::Direction;
use crate::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const TUTORIAL_OR_DATA: &FabricationData = &FabricationData {
    interfaces: OR_INTERFACES,
    expected_table_values: OR_EXPECTED_TABLE_VALUES,
};

pub(super) const OR_INTERFACES: &[Interface] = &[
    Interface {
        name: "In1",
        description: "First input (0 or 1).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In1",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "In2",
        description: "Second input (0 or 1).",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In2",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Out",
        description: "\
                      Should be 1 if either input is 1.\n\
                      Should be 0 if both inputs are 0.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const OR_EXPECTED_TABLE_VALUES: &[u32] = &[
    0, 0, 0,
    1, 0, 1,
    0, 1, 1,
    1, 1, 1,
];

pub(super) const OR_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::PartsTray,
        "Drag chips from the parts\ntray onto the board.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "Drag between board squares to\ncreate wires between chip ports.",
    ),
    (
        TutorialBubblePosition::ControlsTray,
        "When you're ready, press\nthe play button to test\nyour design.",
    ),
];

//===========================================================================//

pub const TUTORIAL_MUX_DATA: &FabricationData = &FabricationData {
    interfaces: MUX_INTERFACES,
    expected_table_values: MUX_EXPECTED_TABLE_VALUES,
};

pub(super) const MUX_INTERFACES: &[Interface] = &[
    Interface {
        name: "In0",
        description: "The input to use when the control value is 0.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In0",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "In1",
        description: "The input to use when the control value is 1.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In1",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Ctrl",
        description: "Indicates which input should be sent to the output.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Ctrl",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
    Interface {
        name: "Out",
        description: "Should be the value of $*In0$* if $*Ctrl$* is 0, or \
                      of $*In1$* if $*Ctrl$* is 1.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const MUX_EXPECTED_TABLE_VALUES: &[u32] = &[
    0, 0, 0, 0,
    0, 0, 1, 0,
    0, 1, 0, 0,
    0, 1, 1, 1,
    1, 0, 0, 1,
    1, 0, 1, 0,
    1, 1, 0, 1,
    1, 1, 1, 1,
];

pub(super) const MUX_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "Drag sideways from a wire to split it.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::South),
        "Perpendicular wires can cross over each other.  Click on a \
         crossing to toggle whether the wires are connected.",
    ),
];

//===========================================================================//

pub const TUTORIAL_ADD_DATA: &FabricationData = &FabricationData {
    interfaces: ADD_INTERFACES,
    expected_table_values: ADD_EXPECTED_TABLE_VALUES,
};

pub(super) const ADD_INTERFACES: &[Interface] = &[
    Interface {
        name: "In1",
        description: "First input (from 0 to 15).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In1",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "In2",
        description: "Second input (from 0 to 15).",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In2",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Out",
        description:
            "\
             Should be the sum of the two inputs (which will never be more \
             than 15 for this task).",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const ADD_EXPECTED_TABLE_VALUES: &[u32] = &[
    2,  1,  3,
    5,  7, 12,
    8,  2, 10,
    4,  4,  8,
    7,  7, 14,
    3,  6,  9,
    6,  5, 11,
    1,  3,  4,
    0, 15, 15,
];

pub(super) const ADD_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "Drag the edges of the board to resize it.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::South),
        "Drag from any grid cell corner in the board to select part of the \
         circuit and cut/copy/paste.",
    ),
];

//===========================================================================//
