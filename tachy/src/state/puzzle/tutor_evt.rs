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

pub const TUTORIAL_DEMUX_DATA: &FabricationData = &FabricationData {
    interfaces: DEMUX_INTERFACES,
    expected_table_values: DEMUX_EXPECTED_TABLE_VALUES,
};

pub(super) const DEMUX_INTERFACES: &[Interface] = &[
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
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Ctrl",
        description:
            "Indicates which output (0-3) the event should be sent to.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Ctrl",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Behavior,
            size: WireSize::Two,
        }],
    },
    Interface {
        name: "Out",
        description:
            "Input events should be sent to $*Out0$* when $*Ctrl$* is 0, to \
             $*Out1$* when $*Ctrl$* is 1, and so on.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[
            InterfacePort {
                name: "Out0",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Out1",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Out2",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
            InterfacePort {
                name: "Out3",
                description: "",
                flow: PortFlow::Recv,
                color: PortColor::Event,
                size: WireSize::Zero,
            },
        ],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const DEMUX_EXPECTED_TABLE_VALUES: &[u32] = &[
      0, 0,   0, NIL, NIL, NIL,
    NIL, 0, NIL, NIL, NIL, NIL,
      0, 1, NIL,   0, NIL, NIL,
    NIL, 1, NIL, NIL, NIL, NIL,
      0, 2, NIL, NIL,   0, NIL,
    NIL, 2, NIL, NIL, NIL, NIL,
      0, 3, NIL, NIL, NIL,   0,
    NIL, 3, NIL, NIL, NIL, NIL,
];

pub(super) const DEMUX_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "A wire can carry either $Obehaviors$D or $Cevents$D, depending on \
         which color of ports it is connected to.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::East),
        "A single $*Demux$* chip can direct an event to one of two places.  \
         A chain of multiple $*Demux$* chips can bifurcate even further.",
    ),
];

//===========================================================================//

pub const TUTORIAL_AMP_DATA: &FabricationData = &FabricationData {
    interfaces: AMP_INTERFACES,
    expected_table_values: AMP_EXPECTED_TABLE_VALUES,
};

pub(super) const AMP_INTERFACES: &[Interface] = &[
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
        description:
            "Output events should be sent here.  The output value should be \
             twice the input value, unless that would be more than 10, in \
             which case no output value should be sent.",
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
const AMP_EXPECTED_TABLE_VALUES: &[u32] = &[
      3,   6,
    NIL, NIL,
      1,   2,
      5,  10,
      6, NIL,
      4,   8,
      7, NIL,
      0,   0,
    NIL, NIL,
      2,   4,
      1,   2,
      6, NIL,
      3,   6,
      5,  10,
];

pub(super) const AMP_BUBBLES: &[(TutorialBubblePosition, &str)] = &[
    (
        TutorialBubblePosition::Bounds(Direction::North),
        "Splitting an event wire sends the same event to multiple \
         receiver ports.  Then each copy of the event can be used \
         separately.",
    ),
    (
        TutorialBubblePosition::Bounds(Direction::South),
        "A $*Demux$* chip can be used to filter out unwanted events by \
         ignoring one of its two output ports.",
    ),
];

//===========================================================================//

pub const TUTORIAL_SUM_DATA: &FabricationData = &FabricationData {
    interfaces: SUM_INTERFACES,
    expected_table_values: SUM_EXPECTED_TABLE_VALUES,
};

pub(super) const SUM_INTERFACES: &[Interface] = &[
    Interface {
        name: "Reset",
        description: "When an event arrives here, the output sum should be \
                      reset to zero.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Reset",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
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
        name: "Total",
        description:
            "Should equal the sum of all input events since the last \
             reset.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Total",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Four,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const SUM_EXPECTED_TABLE_VALUES: &[u32] = &[
    NIL, NIL,  0,
    NIL,   5,  5,
    NIL,   7, 12,
    NIL, NIL, 12,
    NIL,   1, 13,
      0, NIL,  0,
    NIL,   6,  6,
      0,   2,  2,
    NIL,   3,  5,
    NIL,   9, 14,
      0, NIL,  0,
      0, NIL,  0,
      0,  15, 15,
      0,   8,  8,
];

pub(super) const SUM_BUBBLES: &[(TutorialBubblePosition, &str)] = &[(
    TutorialBubblePosition::Bounds(Direction::South),
    "A $*Latest$* chip can act as a memory cell.  Use it in a loop with a \
     $*Delay$* chip to set a new value based on the old one.",
)];

//===========================================================================//
