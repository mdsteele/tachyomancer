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

use cgmath::{Point2, Vector2};

//===========================================================================//

pub trait AsFloat {
    type Output32;

    fn as_f32(&self) -> Self::Output32;
}

impl AsFloat for Point2<i32> {
    type Output32 = Point2<f32>;

    fn as_f32(&self) -> Point2<f32> {
        Point2::new(self.x as f32, self.y as f32)
    }
}

impl AsFloat for Vector2<i32> {
    type Output32 = Vector2<f32>;

    fn as_f32(&self) -> Vector2<f32> {
        Vector2::new(self.x as f32, self.y as f32)
    }
}

//===========================================================================//

pub trait AsInt {
    type Output32;

    fn as_i32_floor(&self) -> Self::Output32;

    fn as_i32_round(&self) -> Self::Output32;
}

impl AsInt for Point2<f32> {
    type Output32 = Point2<i32>;

    fn as_i32_floor(&self) -> Point2<i32> {
        Point2::new(self.x.floor() as i32, self.y.floor() as i32)
    }

    fn as_i32_round(&self) -> Point2<i32> {
        Point2::new(self.x.round() as i32, self.y.round() as i32)
    }
}

impl AsInt for Vector2<f32> {
    type Output32 = Vector2<i32>;

    fn as_i32_floor(&self) -> Vector2<i32> {
        Vector2::new(self.x.floor() as i32, self.y.floor() as i32)
    }

    fn as_i32_round(&self) -> Vector2<i32> {
        Vector2::new(self.x.round() as i32, self.y.round() as i32)
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::AsInt;
    use cgmath::Point2;

    #[test]
    fn point2_as_i32_floor() {
        assert_eq!(Point2::new(3.0, -1.0).as_i32_floor(), Point2::new(3, -1));
        assert_eq!(Point2::new(2.1, 4.9).as_i32_floor(), Point2::new(2, 4));
        assert_eq!(Point2::new(3.7, -1.2).as_i32_floor(), Point2::new(3, -2));
    }

    #[test]
    fn point2_as_i32_round() {
        assert_eq!(Point2::new(3.0, -1.0).as_i32_round(), Point2::new(3, -1));
        assert_eq!(Point2::new(2.1, 4.9).as_i32_round(), Point2::new(2, 5));
        assert_eq!(Point2::new(3.7, -1.2).as_i32_round(), Point2::new(4, -1));
    }
}

//===========================================================================//
