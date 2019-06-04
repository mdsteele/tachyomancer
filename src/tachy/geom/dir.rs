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

use super::coords::{Coords, CoordsDelta};
use cgmath::{Deg, vec2};
use std::ops;
use strum::IntoEnumIterator;

//===========================================================================//

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum Direction {
    East,
    South,
    West,
    North,
}

impl Direction {
    pub fn all() -> DirectionIter { Direction::iter() }

    pub fn delta(self) -> CoordsDelta {
        match self {
            Direction::East => vec2(1, 0),
            Direction::South => vec2(0, 1),
            Direction::West => vec2(-1, 0),
            Direction::North => vec2(0, -1),
        }
    }

    pub fn angle_from_east(self) -> Deg<f32> {
        match self {
            Direction::East => Deg(0.0),
            Direction::South => Deg(90.0),
            Direction::West => Deg(180.0),
            Direction::North => Deg(-90.0),
        }
    }

    pub fn is_vertical(self) -> bool {
        match self {
            Direction::East | Direction::West => false,
            Direction::North | Direction::South => true,
        }
    }

    pub fn flip_vert(self) -> Direction {
        match self {
            Direction::East => Direction::East,
            Direction::South => Direction::North,
            Direction::West => Direction::West,
            Direction::North => Direction::South,
        }
    }

    pub fn rotate_cw(self) -> Direction {
        match self {
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            Direction::North => Direction::East,
        }
    }

    pub fn rotate_ccw(self) -> Direction {
        match self {
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
            Direction::North => Direction::West,
        }
    }
}

impl ops::Add<Direction> for Coords {
    type Output = Coords;

    fn add(self, other: Direction) -> Coords { self + other.delta() }
}

impl ops::Sub<Direction> for Coords {
    type Output = Coords;

    fn sub(self, other: Direction) -> Coords { self - other.delta() }
}

impl ops::Neg for Direction {
    type Output = Direction;

    fn neg(self) -> Direction {
        match self {
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::North => Direction::South,
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Coords, Direction};

    #[test]
    fn direction_add_sub_neg() {
        let coords = Coords { x: 3, y: -4 };
        for dir in Direction::all() {
            let opp = -dir;
            assert_eq!(dir, -opp);
            assert_eq!(coords + dir, coords - opp);
            assert_eq!(coords - dir, coords + opp);
        }
    }
}

//===========================================================================//
