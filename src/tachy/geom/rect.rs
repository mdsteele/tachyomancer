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

use cgmath::{BaseNum, Point2};
use std::ops;

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

    pub fn is_empty(&self) -> bool {
        self.width <= T::zero() || self.height <= T::zero()
    }

    pub fn top_left(&self) -> Point2<T> { Point2::new(self.x, self.y) }

    pub fn right(&self) -> T { self.x + self.width }

    pub fn bottom(&self) -> T { self.y + self.height }

    pub fn area(&self) -> T { self.width * self.height }

    pub fn contains_point(&self, pt: Point2<T>) -> bool {
        pt.x >= self.x && pt.y >= self.y && pt.x < self.x + self.width &&
            pt.y < self.y + self.height
    }

    pub fn contains_rect(&self, rect: Rect<T>) -> bool {
        rect.x >= self.x && rect.y >= self.y &&
            rect.x + rect.width <= self.x + self.width &&
            rect.y + rect.height <= self.y + self.height
    }

    pub fn shrink(&self, horz: T, vert: T) -> Rect<T> {
        Rect::new(self.x + horz,
                  self.y + vert,
                  self.width - (horz + horz),
                  self.height - (vert + vert))
    }
}

impl Rect<i32> {
    pub fn as_f32(&self) -> Rect<f32> {
        Rect::new(self.x as f32,
                  self.y as f32,
                  self.width as f32,
                  self.height as f32)
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

#[cfg(test)]
mod tests {
    use super::Rect;
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
}

//===========================================================================//
