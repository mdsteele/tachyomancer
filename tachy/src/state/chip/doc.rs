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

use super::data::{AbstractConstraint, AbstractPort, ChipData};
use crate::geom::Direction;
use crate::save::WireSize;
use crate::state::{PortColor, PortFlow};

//===========================================================================//

pub const DOC_AN_CHIP_DATA: &ChipData = &ChipData {
    ports: &[
        (PortFlow::Sink, PortColor::Analog, (0, 0), Direction::West),
        (PortFlow::Source, PortColor::Analog, (0, 0), Direction::East),
    ],
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::ANALOG),
        AbstractConstraint::Exact(1, WireSize::ANALOG),
    ],
    dependencies: &[],
};

//===========================================================================//

const DOC_BV_PORTS: &[AbstractPort] = &[
    (PortFlow::Sink, PortColor::Behavior, (0, 0), Direction::West),
    (PortFlow::Source, PortColor::Behavior, (0, 0), Direction::East),
];

const DOC_BV_CHIP_DATA_1: &ChipData = &ChipData {
    ports: DOC_BV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::One),
        AbstractConstraint::Exact(1, WireSize::One),
    ],
    dependencies: &[],
};

const DOC_BV_CHIP_DATA_2: &ChipData = &ChipData {
    ports: DOC_BV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Two),
        AbstractConstraint::Exact(1, WireSize::Two),
    ],
    dependencies: &[],
};

const DOC_BV_CHIP_DATA_4: &ChipData = &ChipData {
    ports: DOC_BV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Four),
        AbstractConstraint::Exact(1, WireSize::Four),
    ],
    dependencies: &[],
};

const DOC_BV_CHIP_DATA_8: &ChipData = &ChipData {
    ports: DOC_BV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Eight),
        AbstractConstraint::Exact(1, WireSize::Eight),
    ],
    dependencies: &[],
};

pub fn doc_bv_chip_data(size: WireSize) -> &'static ChipData {
    match size {
        WireSize::Zero | WireSize::One => DOC_BV_CHIP_DATA_1,
        WireSize::Two => DOC_BV_CHIP_DATA_2,
        WireSize::Four => DOC_BV_CHIP_DATA_4,
        WireSize::Eight => DOC_BV_CHIP_DATA_8,
    }
}

//===========================================================================//

const DOC_EV_PORTS: &[AbstractPort] = &[
    (PortFlow::Sink, PortColor::Event, (0, 0), Direction::West),
    (PortFlow::Source, PortColor::Event, (0, 0), Direction::East),
];

const DOC_EV_CHIP_DATA_0: &ChipData = &ChipData {
    ports: DOC_EV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Zero),
        AbstractConstraint::Exact(1, WireSize::Zero),
    ],
    dependencies: &[],
};

const DOC_EV_CHIP_DATA_1: &ChipData = &ChipData {
    ports: DOC_EV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::One),
        AbstractConstraint::Exact(1, WireSize::One),
    ],
    dependencies: &[],
};

const DOC_EV_CHIP_DATA_2: &ChipData = &ChipData {
    ports: DOC_EV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Two),
        AbstractConstraint::Exact(1, WireSize::Two),
    ],
    dependencies: &[],
};

const DOC_EV_CHIP_DATA_4: &ChipData = &ChipData {
    ports: DOC_EV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Four),
        AbstractConstraint::Exact(1, WireSize::Four),
    ],
    dependencies: &[],
};

const DOC_EV_CHIP_DATA_8: &ChipData = &ChipData {
    ports: DOC_EV_PORTS,
    constraints: &[
        AbstractConstraint::Exact(0, WireSize::Eight),
        AbstractConstraint::Exact(1, WireSize::Eight),
    ],
    dependencies: &[],
};

pub fn doc_ev_chip_data(size: WireSize) -> &'static ChipData {
    match size {
        WireSize::Zero => DOC_EV_CHIP_DATA_0,
        WireSize::One => DOC_EV_CHIP_DATA_1,
        WireSize::Two => DOC_EV_CHIP_DATA_2,
        WireSize::Four => DOC_EV_CHIP_DATA_4,
        WireSize::Eight => DOC_EV_CHIP_DATA_8,
    }
}

//===========================================================================//
