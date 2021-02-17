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
            flow: PortFlow::Send,
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
            flow: PortFlow::Send,
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
            flow: PortFlow::Recv,
            color: PortColor::Event,
            size: WireSize::Two,
        }],
    },
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const ADC_EXPECTED_TABLE_VALUES: &[u32] = &[
    Fixed::from_ratio( 0, 16).to_encoded(), NIL, NIL,
    Fixed::from_ratio( 2, 16).to_encoded(),   0,   0,
    Fixed::from_ratio(10, 16).to_encoded(),   0,   2,
    Fixed::from_ratio( 6, 16).to_encoded(),   0,   1,
    Fixed::from_ratio(10, 16).to_encoded(), NIL, NIL,
    Fixed::from_ratio(16, 16).to_encoded(),   0,   3,
    Fixed::from_ratio(-1, 16).to_encoded(),   0, NIL,
    Fixed::from_ratio( 9, 16).to_encoded(),   0,   2,
    Fixed::from_ratio( 0, 16).to_encoded(),   0,   0,
    Fixed::from_ratio( 5, 16).to_encoded(),   0,   1,
    Fixed::from_ratio(-5, 16).to_encoded(),   0, NIL,
    Fixed::from_ratio(13, 16).to_encoded(),   0,   3,
];

pub(super) const ADC_BUBBLES: &[(TutorialBubblePosition, &str)] = &[(
    TutorialBubblePosition::Bounds(Direction::North),
    "An $Ganalog$D wire carries a continuous voltage from -1 to +1 that can \
     change smoothly over time.",
)];

//===========================================================================//
