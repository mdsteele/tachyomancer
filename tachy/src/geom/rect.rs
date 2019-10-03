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

use super::cast::{AsFloat, AsInt};
use cgmath::{BaseNum, Point2, Vector2};
use std::ops;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RectSize<T> {
    pub width: T,
    pub height: T,
}

impl<T: BaseNum> RectSize<T> {
    pub fn new(width: T, height: T) -> RectSize<T> {
        RectSize { width, height }
    }

    pub fn is_empty(&self) -> bool {
        self.width <= T::zero() || self.height <= T::zero()
    }

    pub fn area(&self) -> T {
        self.width * self.height
    }

    pub fn expand(&self, margin: T) -> RectSize<T> {
        let margin2 = margin + margin;
        RectSize::new(self.width + margin2, self.height + margin2)
    }
}

impl AsFloat for RectSize<i32> {
    type Output32 = RectSize<f32>;

    fn as_f32(&self) -> RectSize<f32> {
        RectSize::new(self.width as f32, self.height as f32)
    }
}

impl AsFloat for RectSize<usize> {
    type Output32 = RectSize<f32>;

    fn as_f32(&self) -> RectSize<f32> {
        RectSize::new(self.width as f32, self.height as f32)
    }
}

impl AsInt for RectSize<f32> {
    type Output32 = RectSize<i32>;

    fn as_i32_floor(&self) -> RectSize<i32> {
        RectSize::new(self.width.floor() as i32, self.height.floor() as i32)
    }

    fn as_i32_round(&self) -> RectSize<i32> {
        RectSize::new(self.width.round() as i32, self.height.round() as i32)
    }
}

impl<T> From<(T, T)> for RectSize<T> {
    fn from((width, height): (T, T)) -> RectSize<T> {
        RectSize { width, height }
    }
}

impl<T: BaseNum> ops::Mul<T> for RectSize<T> {
    type Output = RectSize<T>;

    fn mul(self, other: T) -> RectSize<T> {
        RectSize::new(self.width * other, self.height * other)
    }
}

impl<'d, T: serde::Deserialize<'d>> serde::Deserialize<'d> for RectSize<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let (width, height) = <(T, T)>::deserialize(deserializer)?;
        Ok(RectSize { width, height })
    }
}

impl<T: serde::Serialize> serde::Serialize for RectSize<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&self.width, &self.height).serialize(serializer)
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
        Rect { x, y, width, height }
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

    pub fn top_left(&self) -> Point2<T> {
        Point2::new(self.x, self.y)
    }

    pub fn right(&self) -> T {
        self.x + self.width
    }

    pub fn bottom(&self) -> T {
        self.y + self.height
    }

    pub fn size(&self) -> RectSize<T> {
        RectSize::new(self.width, self.height)
    }

    pub fn area(&self) -> T {
        self.width * self.height
    }

    pub fn contains_point(&self, pt: Point2<T>) -> bool {
        pt.x >= self.x
            && pt.y >= self.y
            && pt.x < self.x + self.width
            && pt.y < self.y + self.height
    }

    pub fn contains_rect(&self, rect: Rect<T>) -> bool {
        rect.x >= self.x
            && rect.y >= self.y
            && rect.x + rect.width <= self.x + self.width
            && rect.y + rect.height <= self.y + self.height
    }

    pub fn expand(&self, margin: T) -> Rect<T> {
        let margin2 = margin + margin;
        Rect::new(
            self.x - margin,
            self.y - margin,
            self.width + margin2,
            self.height + margin2,
        )
    }

    pub fn expand2(&self, horz_margin: T, vert_margin: T) -> Rect<T> {
        Rect::new(
            self.x - horz_margin,
            self.y - vert_margin,
            self.width + horz_margin + horz_margin,
            self.height + vert_margin + vert_margin,
        )
    }

    pub fn intersection(&self, other: Rect<T>) -> Rect<T> {
        if self.x > other.right()
            || self.y > other.bottom()
            || self.right() < other.x
            || self.bottom() < other.y
        {
            return Rect::new(self.x, self.y, T::zero(), T::zero());
        }
        let left = if self.x >= other.x { self.x } else { other.x };
        let right = if self.right() <= other.right() {
            self.right()
        } else {
            other.right()
        };
        let top = if self.y >= other.y { self.y } else { other.y };
        let bottom = if self.bottom() <= other.bottom() {
            self.bottom()
        } else {
            other.bottom()
        };
        return Rect::new(left, top, right - left, bottom - top);
    }
}

impl AsFloat for Rect<i32> {
    type Output32 = Rect<f32>;

    fn as_f32(&self) -> Rect<f32> {
        Rect::new(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }
}

impl IntoIterator for Rect<i32> {
    type Item = Point2<i32>;
    type IntoIter = RectPointsIter<i32>;

    fn into_iter(self) -> RectPointsIter<i32> {
        RectPointsIter::new(self)
    }
}

impl<T: BaseNum> ops::Add<Vector2<T>> for Rect<T> {
    type Output = Rect<T>;

    fn add(self, other: Vector2<T>) -> Rect<T> {
        Rect::new(self.x + other.x, self.y + other.y, self.width, self.height)
    }
}

impl<T: BaseNum> ops::Mul<T> for Rect<T> {
    type Output = Rect<T>;

    fn mul(self, other: T) -> Rect<T> {
        Rect::new(
            self.x * other,
            self.y * other,
            self.width * other,
            self.height * other,
        )
    }
}

impl<T: BaseNum> ops::Sub<Vector2<T>> for Rect<T> {
    type Output = Rect<T>;

    fn sub(self, other: Vector2<T>) -> Rect<T> {
        Rect::new(self.x - other.x, self.y - other.y, self.width, self.height)
    }
}

impl<'d, T: serde::Deserialize<'d>> serde::Deserialize<'d> for Rect<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let (x, y, width, height) = <(T, T, T, T)>::deserialize(deserializer)?;
        Ok(Rect { x, y, width, height })
    }
}

impl<T: serde::Serialize> serde::Serialize for Rect<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&self.x, &self.y, &self.width, &self.height).serialize(serializer)
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
        let (x_hi, y_hi) = if rect.is_empty() {
            (rect.x, rect.y)
        } else {
            (rect.right(), rect.bottom())
        };
        RectPointsIter { x: rect.x, x_lo: rect.x, x_hi, y: rect.y, y_hi }
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
    use super::super::cast::AsFloat;
    use super::{Rect, RectSize};
    use cgmath::Point2;
    use std::collections::HashMap;

    #[test]
    fn rect_size_as_float() {
        assert_eq!(
            RectSize::<i32>::new(3, -4).as_f32(),
            RectSize::<f32>::new(3.0, -4.0)
        );
        assert_eq!(
            RectSize::<usize>::new(3, 4).as_f32(),
            RectSize::<f32>::new(3.0, 4.0)
        );
    }

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
    fn negative_rects_do_not_contain_any_points() {
        assert!(!Rect::new(0, 0, 2, -2).contains_point(Point2::new(1, -1)));
        assert!(!Rect::new(0, 0, 2, -2).contains_point(Point2::new(1, 1)));
        assert!(!Rect::new(0, 0, -2, 2).contains_point(Point2::new(-1, 1)));
        assert!(!Rect::new(0, 0, -2, 2).contains_point(Point2::new(1, 1)));
        assert!(!Rect::new(0, 0, -2, -2).contains_point(Point2::new(-1, -1)));
        assert!(!Rect::new(0, 0, -2, -2).contains_point(Point2::new(1, 1)));
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
    fn rect_intersection() {
        let rect = Rect::new(1, 2, 3, 4);
        assert_eq!(
            rect.intersection(Rect::new(2, 3, 3, 4)),
            Rect::new(2, 3, 2, 3)
        );
        assert_eq!(
            rect.intersection(Rect::new(-1, 4, 4, 4)),
            Rect::new(1, 4, 2, 2)
        );
        assert_eq!(
            rect.intersection(Rect::new(0, 1, 2, 2)),
            Rect::new(1, 2, 1, 1)
        );
        assert_eq!(
            rect.intersection(Rect::new(2, 1, 5, 3)),
            Rect::new(2, 2, 2, 2)
        );
        assert_eq!(
            rect.intersection(Rect::new(0, 0, 10, 10)),
            Rect::new(1, 2, 3, 4)
        );
        assert_eq!(
            rect.intersection(Rect::new(2, 3, 1, 1)),
            Rect::new(2, 3, 1, 1)
        );
        assert_eq!(rect.intersection(Rect::new(10, 20, 4, 4)).area(), 0);
        assert_eq!(rect.intersection(Rect::new(-10, -20, 4, 4)).area(), 0);
    }

    #[test]
    fn rect_iter() {
        let rect = Rect::new(4, 1, 2, 3);
        let points: Vec<(i32, i32)> =
            rect.into_iter().map(|pt| (pt.x, pt.y)).collect();
        assert_eq!(
            points,
            vec![(4, 1), (5, 1), (4, 2), (5, 2), (4, 3), (5, 3)]
        );
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

    #[test]
    fn negative_rect_iter() {
        let rect = Rect::new(1, 2, -3, 4);
        let mut iter = rect.into_iter();
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());

        let rect = Rect::new(1, 2, 3, -4);
        let mut iter = rect.into_iter();
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());

        let rect = Rect::new(1, 2, -3, -4);
        let mut iter = rect.into_iter();
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn serialize_rect() {
        let mut map = HashMap::<String, Rect<i32>>::new();
        map.insert("foo".to_string(), Rect::new(-2, -1, 8, 5));
        let bytes = toml::to_vec(&map).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap().as_str(),
            "foo = [-2, -1, 8, 5]\n"
        );
    }

    #[test]
    fn deserialize_rect() {
        let toml = "foo = [-2, -1, 8, 5]\n";
        let map: HashMap<String, Rect<i32>> =
            toml::from_slice(toml.as_bytes()).unwrap();
        assert_eq!(map.get("foo"), Some(&Rect::new(-2, -1, 8, 5)));
    }
}

//===========================================================================//
