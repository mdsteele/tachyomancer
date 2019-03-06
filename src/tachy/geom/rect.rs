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

use super::cast::AsFloat;
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

impl AsFloat for Rect<i32> {
    type Output32 = Rect<f32>;

    fn as_f32(&self) -> Rect<f32> {
        Rect::new(self.x as f32,
                  self.y as f32,
                  self.width as f32,
                  self.height as f32)
    }
}

impl IntoIterator for Rect<i32> {
    type Item = Point2<i32>;
    type IntoIter = RectPointsIter<i32>;

    fn into_iter(self) -> RectPointsIter<i32> { RectPointsIter::new(self) }
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

pub struct RectPointsIter<T> {
    x: T,
    x_lo: T,
    x_hi: T,
    y: T,
    y_hi: T,
}

impl<T: BaseNum> RectPointsIter<T> {
    fn new(rect: Rect<T>) -> RectPointsIter<T> {
        RectPointsIter {
            x: rect.x,
            x_lo: rect.x,
            x_hi: rect.right(),
            y: rect.y,
            y_hi: rect.bottom(),
        }
    }
}

impl Iterator for RectPointsIter<i32> {
    type Item = Point2<i32>;

    fn next(&mut self) -> Option<Point2<i32>> {
        if self.y < self.y_hi {
            if self.x < self.x_hi {
                let point = Point2::new(self.x, self.y);
                self.x += 1;
                if self.x == self.x_hi {
                    self.x = self.x_lo;
                    self.y += 1;
                }
                return Some(point);
            }
        }
        return None;
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let width = self.x_hi - self.x_lo;
        let height = self.y_hi - self.y;
        let size = width * height - (self.x - self.x_lo);
        let size = size.max(0) as usize;
        (size, Some(size))
    }
}

impl ExactSizeIterator for RectPointsIter<i32> {}

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

    #[test]
    fn rect_iter() {
        let rect = Rect::new(4, 1, 2, 3);
        let points: Vec<(i32, i32)> =
            rect.into_iter().map(|pt| (pt.x, pt.y)).collect();
        assert_eq!(points,
                   vec![(4, 1), (5, 1), (4, 2), (5, 2), (4, 3), (5, 3)]);
    }

    #[test]
    fn rect_iter_exact_size() {
        let rect = Rect::new(1, 4, 3, 2);
        let mut iter = rect.into_iter();
        for index in 0..6 {
            assert_eq!(iter.len(), 6 - index);
            assert!(iter.next().is_some());
        }
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }
}

//===========================================================================//
