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
use crate::geom::{Direction, Fixed};
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow};

//===========================================================================//

const fn voltage(numerator: i32, denominator: i32) -> u32 {
    Fixed::from_ratio(numerator, denominator).to_encoded()
}

//===========================================================================//

pub const TUTORIAL_ADC_DATA: &FabricationData = &FabricationData {
    interfaces: ADC_INTERFACES,
    expected_table_values: ADC_EXPECTED_TABLE_VALUES,
};

pub(super) const ADC_INTERFACES: &[Interface] = &[
    Interface {
        name: "In",
        description: "The analog input voltage to be converted.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Analog,
            size: WireSize::ANALOG,
        }],
    },
    Interface {
        name: "Sample",
        description:
            "When an event arrives here, sample the analog input, and if it's \
             non-negative, send a 2-bit representation of the voltage to \
             $*Out$*.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Sample",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "Out",
        description: "Send digitized voltage values here:\n    \
                      Send 0 for 0.0 to +0.25\n    \
                      Send 1 for +0.25 to +0.5\n    \
                      Send 2 for +0.5 to +0.75\n    \
                      Send 3 for +0.75 to +1.0\n\
                      Send nothing for negative voltages.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Sink,
            color: PortColor::Event,
            size: WireSize::Two,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const ADC_EXPECTED_TABLE_VALUES: &[u32] = &[
    voltage( 0, 16), NIL, NIL,
    voltage( 2, 16),   0,   0,
    voltage(10, 16),   0,   2,
    voltage( 6, 16),   0,   1,
    voltage(10, 16), NIL, NIL,
    voltage(16, 16),   0,   3,
    voltage(-1, 16),   0, NIL,
    voltage( 9, 16),   0,   2,
    voltage( 0, 16),   0,   0,
    voltage( 5, 16),   0,   1,
    voltage(-5, 16),   0, NIL,
    voltage(13, 16),   0,   3,
];

pub(super) const ADC_BUBBLES: &[(TutorialBubblePosition, &str)] = &[(
    TutorialBubblePosition::Bounds(Direction::North),
    "An $Ganalog$D wire carries a continuous voltage from -1 to +1 that can \
     change smoothly over time.",
)];

//===========================================================================//

pub const TUTORIAL_INTEGRATE_DATA: &FabricationData = &FabricationData {
    interfaces: INTEGRATE_INTERFACES,
    expected_table_values: INTEGRATE_EXPECTED_TABLE_VALUES,
};

pub(super) const INTEGRATE_INTERFACES: &[Interface] = &[
    Interface {
        name: "In",
        description: "The analog input voltage to be integrated.",
        side: Direction::West,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "In",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Analog,
            size: WireSize::ANALOG,
        }],
    },
    Interface {
        name: "Reset",
        description:
            "When an event arrives here, the output voltage should be reset \
             to the current $*IC$* voltage.",
        side: Direction::North,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Reset",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Event,
            size: WireSize::Zero,
        }],
    },
    Interface {
        name: "IC",
        description:
            "The \"initial condition\" voltage to reset to when a $*Reset$* \
             event arrives.",
        side: Direction::South,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "IC",
            description: "",
            flow: PortFlow::Source,
            color: PortColor::Analog,
            size: WireSize::ANALOG,
        }],
    },
    Interface {
        name: "Out",
        description:
            "This should start at zero, and sum up the input voltages over \
             time.",
        side: Direction::East,
        pos: InterfacePosition::Center,
        ports: &[InterfacePort {
            name: "Out",
            description: "",
            flow: PortFlow::Sink,
            color: PortColor::Analog,
            size: WireSize::ANALOG,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const INTEGRATE_EXPECTED_TABLE_VALUES: &[u32] = &[
    voltage( 1, 2), NIL, voltage( 0, 1), voltage( 1, 2),
    voltage( 1, 4), NIL, voltage( 0, 1), voltage( 3, 4),
    voltage(-1, 2), NIL, voltage(-1, 4), voltage( 1, 4),
    voltage( 0, 1), NIL, voltage(-1, 4), voltage( 1, 4),
    voltage( 0, 1),   0, voltage(-1, 4), voltage(-1, 4),
    voltage( 0, 1), NIL, voltage( 0, 1), voltage(-1, 4),
    voltage( 3, 4), NIL, voltage( 0, 1), voltage( 1, 2),
    voltage( 3, 4), NIL, voltage( 0, 1), voltage( 1, 1),
    voltage( 3, 4), NIL, voltage(-1, 1), voltage( 1, 1),
    voltage( 3, 4),   0, voltage(-1, 1), voltage(-1, 4),
    voltage( 1, 4), NIL, voltage(-1, 1), voltage( 0, 1),
    voltage( 1, 4), NIL, voltage(-1, 1), voltage( 1, 4),
];

pub(super) const INTEGRATE_BUBBLES: &[(TutorialBubblePosition, &str)] =
    &[(TutorialBubblePosition::Bounds(Direction::North), "TODO")];

//===========================================================================//
