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

use cgmath::{Point2, Vector2, vec2};
use std::ops;

//===========================================================================//

pub type Coords = Point2<i32>;

pub type CoordsDelta = Vector2<i32>;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    East,
    South,
    West,
    North,
}

impl Direction {
    pub fn delta(self) -> CoordsDelta {
        match self {
            Direction::East => vec2(1, 0),
            Direction::South => vec2(0, 1),
            Direction::West => vec2(-1, 0),
            Direction::North => vec2(0, -1),
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Orientation {
    rotate: u8,
    mirror: bool,
}

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation {
            rotate: 0,
            mirror: false,
        }
    }
}

impl ops::Mul<Direction> for Orientation {
    type Output = Direction;

    fn mul(self, mut dir: Direction) -> Direction {
        if self.mirror {
            dir = dir.flip_vert();
        }
        match self.rotate {
            0 => dir,
            1 => dir.rotate_cw(),
            2 => -dir,
            3 => dir.rotate_ccw(),
            _ => unreachable!(),
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Direction, Orientation};

    const ALL_DIRECTIONS: &[Direction] = &[
        Direction::East,
        Direction::South,
        Direction::West,
        Direction::North,
    ];

    #[test]
    fn default_orient_does_not_change_dir() {
        for &dir in ALL_DIRECTIONS {
            assert_eq!(Orientation::default() * dir, dir);
        }
    }
}

//===========================================================================//
