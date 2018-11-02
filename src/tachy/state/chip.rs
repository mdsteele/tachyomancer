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

use super::geom::{Direction, Orientation};
use super::port::PortSpec;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChipType {
    Const(u32),
    // Bitwise:
    Not,
    And,
    // Events:
    Delay,
}

impl ChipType {
    pub(super) fn ports(self, orient: Orientation) -> Vec<PortSpec> {
        match self {
            ChipType::Const(_) => {
                vec![PortSpec::bsend(orient * Direction::East)]
            }
            ChipType::Not => {
                vec![
                    PortSpec::bsend(orient * Direction::East),
                    PortSpec::brecv(orient * Direction::West),
                ]
            }
            ChipType::And => {
                vec![
                    PortSpec::bsend(orient * Direction::East),
                    PortSpec::brecv(orient * Direction::West),
                    PortSpec::brecv(orient * Direction::South),
                ]
            }
            ChipType::Delay => {
                vec![
                    PortSpec::esend(orient * Direction::East),
                    PortSpec::erecv(orient * Direction::West),
                ]
            }
        }
    }
}

//===========================================================================//
