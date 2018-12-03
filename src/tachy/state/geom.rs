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

use cgmath::{BaseNum, Deg, Matrix4, Point2, Vector2, vec2};
use std::ops;

//===========================================================================//

pub type Coords = Point2<i32>;

pub type CoordsDelta = Vector2<i32>;

pub type CoordsRect = Rect<i32>;

pub type CoordsSize = RectSize<i32>;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RectSize<T> {
    pub width: T,
    pub height: T,
}

impl<T> RectSize<T> {
    pub fn new(width: T, height: T) -> RectSize<T> {
        RectSize { width, height }
    }
}

impl<T> From<(T, T)> for RectSize<T> {
    fn from((width, height): (T, T)) -> RectSize<T> {
        RectSize { width, height }
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T: BaseNum> Rect<T> {
    pub fn new(x: T, y: T, width: T, height: T) -> Rect<T> {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn with_size(top_left: Point2<T>, size: RectSize<T>) -> Rect<T> {
        Rect {
            x: top_left.x,
            y: top_left.y,
            width: size.width,
            height: size.height,
        }
    }

    // TODO: get rid of these
    pub fn x(&self) -> T { self.x }
    pub fn y(&self) -> T { self.y }
    pub fn width(&self) -> T { self.width }
    pub fn height(&self) -> T { self.height }

    pub fn top_left(&self) -> Point2<T> { Point2::new(self.x, self.y) }

    pub fn right(&self) -> T { self.x + self.width }

    pub fn bottom(&self) -> T { self.y + self.height }

    pub fn contains_point(&self, pt: Point2<T>) -> bool {
        pt.x >= self.x && pt.y >= self.y && pt.x < self.x + self.width &&
            pt.y < self.y + self.height
    }

    pub fn contains_rect(&self, rect: Rect<T>) -> bool {
        rect.x >= self.x && rect.y >= self.y &&
            rect.x + rect.width <= self.x + self.width &&
            rect.y + rect.height <= self.y + self.height
    }
}

impl<T: BaseNum> ops::Mul<T> for Rect<T> {
    type Output = Rect<T>;

    fn mul(self, other: T) -> Rect<T> {
        Rect::new(self.x * other,
                  self.y * other,
                  self.width * other,
                  self.height * other)
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    East,
    South,
    West,
    North,
}

impl Direction {
    pub fn all() -> &'static [Direction] {
        &[
            Direction::East,
            Direction::South,
            Direction::North,
            Direction::West,
        ]
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Orientation {
    rotate: u8,
    mirror: bool,
}

impl Orientation {
    pub fn is_mirrored(&self) -> bool { self.mirror }

    pub fn matrix(&self) -> Matrix4<f32> {
        let matrix = Matrix4::from_angle_z(Deg(90.0 * (self.rotate as f32)));
        if self.mirror {
            matrix * Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0)
        } else {
            matrix
        }
    }

    pub fn transform_in_rect(&self, delta: CoordsDelta, size: CoordsSize)
                             -> CoordsDelta {
        let x = delta.x;
        let y = if self.mirror {
            size.height - delta.y - 1
        } else {
            delta.y
        };
        let (x, y) = match self.rotate {
            0 => (x, y),
            1 => (size.height - y - 1, x),
            2 => (size.width - x - 1, size.height - y - 1),
            3 => (y, size.width - x - 1),
            _ => unreachable!(),
        };
        CoordsDelta { x, y }
    }

    pub fn flip_horz(self) -> Orientation {
        Orientation {
            rotate: if self.rotate % 2 != 0 {
                self.rotate
            } else {
                (self.rotate + 2) % 4
            },
            mirror: !self.mirror,
        }
    }

    pub fn flip_vert(self) -> Orientation {
        Orientation {
            rotate: if self.rotate % 2 == 0 {
                self.rotate
            } else {
                (self.rotate + 2) % 4
            },
            mirror: !self.mirror,
        }
    }

    pub fn rotate_cw(self) -> Orientation {
        Orientation {
            rotate: (self.rotate + 1) % 4,
            mirror: self.mirror,
        }
    }

    pub fn rotate_ccw(self) -> Orientation {
        Orientation {
            rotate: (self.rotate + 3) % 4,
            mirror: self.mirror,
        }
    }
}

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation {
            rotate: 0,
            mirror: false,
        }
    }
}

impl ops::Mul<Orientation> for Orientation {
    type Output = Orientation;

    fn mul(self, mut other: Orientation) -> Orientation {
        if self.mirror {
            other = other.flip_vert();
        }
        other.rotate = (other.rotate + self.rotate) % 4;
        other
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

impl<T> ops::Mul<RectSize<T>> for Orientation {
    type Output = RectSize<T>;

    fn mul(self, size: RectSize<T>) -> RectSize<T> {
        if self.rotate % 2 == 0 {
            size
        } else {
            RectSize {
                width: size.height,
                height: size.width,
            }
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Coords, Direction, Orientation, Rect};
    use cgmath::Point2;

    #[test]
    fn rect_contains_point() {
        let rect = Rect::new(1, 2, 3, 4);
        assert!(rect.contains_point(Point2::new(1, 2)));
        assert!(!rect.contains_point(Point2::new(0, 2)));
        assert!(rect.contains_point(Point2::new(3, 5)));
        assert!(!rect.contains_point(Point2::new(4, 5)));
        assert!(!rect.contains_point(Point2::new(3, 6)));
    }

    #[test]
    fn rect_contains_rect() {
        let rect = Rect::new(1, 2, 3, 4);
        assert!(rect.contains_rect(rect));
        assert!(rect.contains_rect(Rect::new(1, 2, 2, 2)));
        assert!(rect.contains_rect(Rect::new(2, 4, 2, 2)));
        assert!(!rect.contains_rect(Rect::new(0, 2, 2, 2)));
        assert!(!rect.contains_rect(Rect::new(1, 1, 2, 2)));
        assert!(!rect.contains_rect(Rect::new(3, 2, 2, 2)));
        assert!(!rect.contains_rect(Rect::new(1, 5, 2, 2)));
    }

    #[test]
    fn direction_add_sub_neg() {
        let coords = Coords { x: 3, y: -4 };
        for &dir in Direction::all() {
            let opp = -dir;
            assert_eq!(dir, -opp);
            assert_eq!(coords + dir, coords - opp);
            assert_eq!(coords - dir, coords + opp);
        }
    }

    #[test]
    fn orientation_times_direction() {
        let orient = Orientation::default();
        for &dir in Direction::all() {
            assert_eq!(orient * dir, dir);
        }

        let orient = Orientation::default().rotate_cw();
        for &dir in Direction::all() {
            assert_eq!(orient * dir, dir.rotate_cw());
        }

        let orient = Orientation::default().rotate_ccw();
        for &dir in Direction::all() {
            assert_eq!(orient * dir, dir.rotate_ccw());
        }

        let orient = Orientation::default().flip_vert();
        for &dir in Direction::all() {
            assert_eq!(orient * dir, dir.flip_vert());
        }
    }

    #[test]
    fn orientation_times_orientation() {
        let orient = Orientation::default().flip_vert().rotate_cw();
        assert_eq!(orient * orient, Orientation::default());

        let orient = Orientation::default().rotate_cw().flip_vert();
        assert_eq!(orient.rotate_ccw(), Orientation::default().flip_horz());
    }
}

//===========================================================================//
