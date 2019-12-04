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
use super::shared::FabricationData;
use crate::geom::Direction;
use crate::state::{PortColor, PortFlow, WireSize};

//===========================================================================//

pub const FABRICATE_XOR_DATA: &FabricationData = &FabricationData {
    interfaces: XOR_INTERFACES,
    expected_table_values: XOR_EXPECTED_TABLE_VALUES,
};

pub(super) const XOR_INTERFACES: &[Interface] = &[
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
        description: "Should be 1 if exactly one input is 1.\n\
                      Should be 0 if the inputs are both 0 or both 1.",
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
const XOR_EXPECTED_TABLE_VALUES: &[u32] = &[
    0, 0, 0,
    1, 0, 1,
    0, 1, 1,
    1, 1, 0,
];

//===========================================================================//

pub const FABRICATE_MUL_DATA: &FabricationData = &FabricationData {
    interfaces: MUL_INTERFACES,
    expected_table_values: MUL_EXPECTED_TABLE_VALUES,
};

pub(super) const MUL_INTERFACES: &[Interface] = &[
    Interface {
        name: "In1",
        description: "First input (from 0 to 255).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In1",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "In2",
        description: "Second input (from 0 to 255).",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In2",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "Out",
        description:
            "Should be the product of the two inputs (which will never be \
             more than 255 for this task).",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const MUL_EXPECTED_TABLE_VALUES: &[u32] = &[
     4,   3,  12,
     3,  10,  30,
    20,  12, 240,
     1, 197, 197,
    83,   0,   0,
    13,  19, 247,
    12,   1,  12,
     2,  73, 146,
     0,   7,   0,
     7,  13,  91,
];

//===========================================================================//

pub const FABRICATE_HALVE_DATA: &FabricationData = &FabricationData {
    interfaces: HALVE_INTERFACES,
    expected_table_values: HALVE_EXPECTED_TABLE_VALUES,
};

pub(super) const HALVE_INTERFACES: &[Interface] = &[
    Interface {
        name: "In",
        description: "Input (from 0 to 15).",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Out",
        description: "Should be half the value of the input, rounded down.",
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
const HALVE_EXPECTED_TABLE_VALUES: &[u32] = &[
     0, 0,
     1, 0,
     2, 1,
     3, 1,
     4, 2,
     5, 2,
     6, 3,
     7, 3,
     8, 4,
     9, 4,
    10, 5,
    11, 5,
    12, 6,
    13, 6,
    14, 7,
    15, 7,
];

//===========================================================================//
