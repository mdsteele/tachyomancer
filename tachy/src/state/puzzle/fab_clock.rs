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

pub const FABRICATE_EGG_TIMER_DATA: &FabricationData = &FabricationData {
    interfaces: EGG_TIMER_INTERFACES,
    expected_table_values: EGG_TIMER_EXPECTED_TABLE_VALUES,
};

pub(super) const EGG_TIMER_INTERFACES: &[Interface] = &[
    Interface {
        name: "Set",
        description:
            "When an event value arrives here, the timer should be set to go \
             off in that many time steps.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Set",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "Remain",
        description:
            "Should be the number of time steps before the timer goes off.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Remain",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
    Interface {
        name: "Alarm",
        description: "Send an event here when the timer goes off.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Alarm",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const EGG_TIMER_EXPECTED_TABLE_VALUES: &[u32] = &[
    NIL, 0, NIL,
      3, 3, NIL,
    NIL, 2, NIL,
    NIL, 1, NIL,
    NIL, 0,   0,
    NIL, 0, NIL,
      5, 5, NIL,
    NIL, 4, NIL,
      1, 1, NIL,
    NIL, 0,   0,
      3, 3, NIL,
    NIL, 2, NIL,
    NIL, 1, NIL,
      9, 9, NIL,
    NIL, 8, NIL,
    NIL, 7, NIL,
      0, 0,   0,
    NIL, 0, NIL,
      0, 0,   0,
    NIL, 0, NIL,
];

//===========================================================================//

pub const FABRICATE_STOPWATCH_DATA: &FabricationData = &FabricationData {
    interfaces: STOPWATCH_INTERFACES,
    expected_table_values: STOPWATCH_EXPECTED_TABLE_VALUES,
};

pub(super) const STOPWATCH_INTERFACES: &[Interface] = &[
    Interface {
        name: "Start",
        description:
            "When an event value arrives here, the timer should start \
             counting up from its current value.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Start",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Stop",
        description:
            "When an event value arrives here, the timer should pause.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Stop",
            description: "",
            flow: PortFlow::Send,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Reset",
        description:
            "When an event value arrives here, the timer value should be \
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
        name: "Time",
        description: "Should be the current timer value, starting at zero.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Time",
            description: "",
            flow: PortFlow::Recv,
            color: PortColor::Behavior,
            size: WireSize::Eight,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const STOPWATCH_EXPECTED_TABLE_VALUES: &[u32] = &[
    NIL, NIL, NIL, 0,
      0, NIL, NIL, 0,
    NIL, NIL, NIL, 1,
    NIL, NIL, NIL, 2,
    NIL,   0, NIL, 3,
    NIL, NIL, NIL, 3,
    NIL, NIL,   0, 0,
    NIL, NIL, NIL, 0,
      0, NIL, NIL, 0,
      0, NIL, NIL, 1,
    NIL, NIL, NIL, 2,
    NIL, NIL,   0, 0,
    NIL, NIL, NIL, 1,
    NIL, NIL, NIL, 2,
    NIL,   0, NIL, 3,
    NIL,   0, NIL, 3,
    NIL, NIL, NIL, 3,
      0, NIL,   0, 0,
    NIL, NIL, NIL, 1,
    NIL, NIL, NIL, 2,
    NIL,   0,   0, 0,
    NIL, NIL, NIL, 0,
];

//===========================================================================//
