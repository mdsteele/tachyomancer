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
use cgmath::{vec2, Deg};
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
    pub fn all() -> DirectionIter {
        Direction::iter()
    }

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

    fn add(self, other: Direction) -> Coords {
        self + other.delta()
    }
}

impl ops::Add<DirDelta> for Direction {
    type Output = Direction;

    fn add(self, other: DirDelta) -> Direction {
        match other {
            DirDelta::Same => self,
            DirDelta::RotateCw => self.rotate_cw(),
            DirDelta::Opposite => -self,
            DirDelta::RotateCcw => self.rotate_ccw(),
        }
    }
}

impl ops::Sub<Direction> for Coords {
    type Output = Coords;

    fn sub(self, other: Direction) -> Coords {
        self - other.delta()
    }
}

impl ops::Sub<Direction> for Direction {
    type Output = DirDelta;

    fn sub(self, other: Direction) -> DirDelta {
        if self == other {
            DirDelta::Same
        } else if self == other.rotate_cw() {
            DirDelta::RotateCw
        } else if self == -other {
            DirDelta::Opposite
        } else {
            debug_assert_eq!(self, other.rotate_ccw());
            DirDelta::RotateCcw
        }
    }
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
pub enum DirDelta {
    Same,
    RotateCw,
    Opposite,
    RotateCcw,
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Coords, Direction};
    use cgmath::{Angle, Deg};

    #[test]
    fn direction_delta_angle() {
        for dir in Direction::all() {
            let delta = dir.delta();
            let angle = dir.angle_from_east();
            assert_eq!(Deg::atan2(delta.y as f32, delta.x as f32), angle);
        }
    }

    #[test]
    fn direction_flip() {
        for dir in Direction::all() {
            assert_eq!(dir.is_vertical(), dir.flip_vert().is_vertical());
            if dir.is_vertical() {
                assert_eq!(dir.flip_vert(), -dir);
            } else {
                assert_eq!(dir.flip_vert(), dir);
            }
        }
    }

    #[test]
    fn direction_rotate() {
        for dir in Direction::all() {
            assert_ne!(dir, dir.rotate_cw());
            assert_ne!(dir, dir.rotate_ccw());
            assert_eq!(dir, dir.rotate_cw().rotate_ccw());
            assert_eq!(dir, dir.rotate_ccw().rotate_cw());
            assert_eq!(dir.is_vertical(), !dir.rotate_cw().is_vertical());
            assert_eq!(dir.is_vertical(), !dir.rotate_ccw().is_vertical());
        }
    }

    #[test]
    fn direction_add_sub_neg() {
        let coords = Coords { x: 3, y: -4 };
        for dir in Direction::all() {
            let opp = -dir;
            assert_ne!(dir, opp);
            assert_eq!(dir, -opp);
            assert_eq!(coords + dir, coords - opp);
            assert_eq!(coords - dir, coords + opp);
        }
    }
}

//===========================================================================//
