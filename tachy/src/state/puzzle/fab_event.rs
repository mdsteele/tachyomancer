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
use super::shared::{FabricationData, NIL};
use crate::geom::Direction;
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow};

//===========================================================================//

pub const FABRICATE_COUNTER_DATA: &FabricationData = &FabricationData {
    interfaces: COUNTER_INTERFACES,
    expected_table_values: COUNTER_EXPECTED_TABLE_VALUES,
};

pub(super) const COUNTER_INTERFACES: &[Interface] = &[
    Interface {
        name: "Set",
        description:
            "When an event is sent from here, the counter should be set to \
             that value.",
        side: Direction::South,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Set",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Inc",
        description:
            "When an event is sent from here, the counter value should be \
             incremented by one.",
        side: Direction::North,
        pos: InterfacePosition::Left(0),
        ports: &[InterfacePort {
            name: "Inc",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Dec",
        description:
            "When an event is sent from here, the counter value should be \
             decremented by one.",
        side: Direction::South,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Dec",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Out",
        description:
            "Should be equal to the current counter value (initially zero).",
        side: Direction::North,
        pos: InterfacePosition::Right(0),
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Sink,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const COUNTER_EXPECTED_TABLE_VALUES: &[u32] = &[
    NIL, NIL, NIL,  0,
    NIL,   0, NIL,  1,
    NIL,   0, NIL,  2,
    NIL,   0, NIL,  3,
    NIL, NIL,   0,  2,
    NIL, NIL,   0,  1,
    NIL, NIL,   0,  0,
    NIL, NIL,   0, 15,
    NIL, NIL,   0, 14,
    NIL,   0, NIL, 15,
    NIL,   0, NIL,  0,
    NIL,   0, NIL,  1,
      7, NIL, NIL,  7,
    NIL,   0, NIL,  8,
     11, NIL, NIL, 11,
    NIL, NIL,   0, 10,
      3,   0, NIL,  4,
     13, NIL,   0, 12,
    NIL,   0,   0, 12,
      5,   0,   0,  5,
];

//===========================================================================//

pub const FABRICATE_INC_DATA: &FabricationData = &FabricationData {
    interfaces: INC_INTERFACES,
    expected_table_values: INC_EXPECTED_TABLE_VALUES,
};

pub(super) const INC_INTERFACES: &[Interface] = &[
    Interface {
        name: "InE",
        description: "Input events arrive here.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "InE",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "InB",
        description: "Provides the value that should be added to each event.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "InB",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
    Interface {
        name: "Out",
        description:
            "Whenever an input event arrives, send an event here with the sum \
             of the two input values.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Sink,
            color: PortColor::Event,
            size: WireSize::Four,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const INC_EXPECTED_TABLE_VALUES: &[u32] = &[
      4,  7,  11,
    NIL, 12, NIL,
      6,  0,   6,
      9,  1,  10,
    NIL,  8, NIL,
      0, 14,  14,
      1,  2,   3,
      5, 10,  15,
];

//===========================================================================//

pub const FABRICATE_LATCH_DATA: &FabricationData = &FabricationData {
    interfaces: LATCH_INTERFACES,
    expected_table_values: LATCH_EXPECTED_TABLE_VALUES,
};

pub(super) const LATCH_INTERFACES: &[Interface] = &[
    Interface {
        name: "Set",
        description:
            "When an event arrives here, the output should be set to 1.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Set",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Rst",
        description:
            "When an event arrives here, the output should be reset to 0.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Rst",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Out",
        description: "Should be set to the current output (initially zero).",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Sink,
            color: PortColor::Behavior,
            size: WireSize::One,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const LATCH_EXPECTED_TABLE_VALUES: &[u32] = &[
    NIL, NIL, 0,
      0, NIL, 1,
    NIL,   0, 0,
    NIL,   0, 0,
      0, NIL, 1,
      0, NIL, 1,
    NIL, NIL, 1,
      0,   0, 0,
    NIL, NIL, 0,
      0,   0, 1,
];

//===========================================================================//
