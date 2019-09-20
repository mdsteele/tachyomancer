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

use super::coords::{CoordsDelta, CoordsSize};
use super::dir::Direction;
use super::matrix::MatrixExt;
use super::rect::RectSize;
use cgmath::{Deg, Matrix4};
use std::fmt;
use std::ops;
use std::str;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Orientation {
    rotate: u8,
    mirror: bool,
}

impl Orientation {
    pub fn is_mirrored(&self) -> bool {
        self.mirror
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        let matrix = Matrix4::from_angle_z(Deg(90.0 * (self.rotate as f32)));
        if self.mirror {
            matrix * Matrix4::scale2(1.0, -1.0)
        } else {
            matrix
        }
    }

    pub fn transform_in_size(
        &self,
        delta: CoordsDelta,
        size: CoordsSize,
    ) -> CoordsDelta {
        let x = delta.x;
        let y = if self.mirror { size.height - delta.y - 1 } else { delta.y };
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
        Orientation { rotate: (self.rotate + 1) % 4, mirror: self.mirror }
    }

    pub fn rotate_ccw(self) -> Orientation {
        Orientation { rotate: (self.rotate + 3) % 4, mirror: self.mirror }
    }
}

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation { rotate: 0, mirror: false }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mirror = if self.mirror { 't' } else { 'f' };
        formatter.write_fmt(format_args!("{}{}", mirror, self.rotate))
    }
}

impl str::FromStr for Orientation {
    type Err = String;

    fn from_str(string: &str) -> Result<Orientation, String> {
        let (mirror, rotate) = match string {
            "f0" => (false, 0),
            "f1" => (false, 1),
            "f2" => (false, 2),
            "f3" => (false, 3),
            "t0" => (true, 0),
            "t1" => (true, 1),
            "t2" => (true, 2),
            "t3" => (true, 3),
            _ => return Err(string.to_string()),
        };
        Ok(Orientation { rotate, mirror })
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
            RectSize { width: size.height, height: size.width }
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Direction, Orientation};

    #[test]
    fn orientation_to_and_from_string() {
        let orient = Orientation::default();
        assert_eq!(orient.to_string(), "f0".to_string());
        assert_eq!("f0".parse(), Ok(orient));

        let orient = Orientation::default().rotate_ccw();
        assert_eq!(orient.to_string(), "f3".to_string());
        assert_eq!("f3".parse(), Ok(orient));

        let orient = Orientation::default().flip_horz();
        assert_eq!(orient.to_string(), "t2".to_string());
        assert_eq!("t2".parse(), Ok(orient));
    }

    #[test]
    fn orientation_times_direction() {
        let orient = Orientation::default();
        for dir in Direction::all() {
            assert_eq!(orient * dir, dir);
        }

        let orient = Orientation::default().rotate_cw();
        for dir in Direction::all() {
            assert_eq!(orient * dir, dir.rotate_cw());
        }

        let orient = Orientation::default().rotate_ccw();
        for dir in Direction::all() {
            assert_eq!(orient * dir, dir.rotate_ccw());
        }

        let orient = Orientation::default().flip_vert();
        for dir in Direction::all() {
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
