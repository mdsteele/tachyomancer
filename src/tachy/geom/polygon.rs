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

#[derive(Clone, Debug)]
pub struct Polygon {
    vertices: Vec<Point2<f32>>,
}

impl Polygon {
    pub fn new(vertices: Vec<Point2<f32>>) -> Polygon { Polygon { vertices } }

    /// Given the endpoints of a linear path, returns the index of the first
    /// polygon edge intersected by the path and the point where the
    /// intersection occurs, if any.
    pub fn edge_intersection(&self, start: Point2<f32>, mut end: Point2<f32>)
                             -> Option<(usize, Point2<f32>)> {
        let mut hit: Option<(usize, Point2<f32>)> = None;
        for (index, edge) in self.edges().enumerate() {
            if let Some(pos) = edge.intersection(start, end) {
                end = pos;
                hit = Some((index, pos));
            }
        }
        hit
    }

    fn edges(&self) -> PolygonEdges { PolygonEdges::new(&self.vertices) }
}

//===========================================================================//

struct PolygonEdges<'a> {
    vertices: &'a [Point2<f32>],
    index: usize,
}

impl<'a> PolygonEdges<'a> {
    fn new(vertices: &'a [Point2<f32>]) -> PolygonEdges<'a> {
        PolygonEdges { vertices, index: 0 }
    }
}

impl<'a> Iterator for PolygonEdges<'a> {
    type Item = LineSegment;

    fn next(&mut self) -> Option<LineSegment> {
        let num_vertices = self.vertices.len();
        if self.index >= num_vertices {
            None
        } else {
            let p1 = self.vertices[self.index];
            self.index += 1;
            let p2 = if self.index == num_vertices {
                self.vertices[0]
            } else {
                self.vertices[self.index]
            };
            Some(LineSegment::new(p1, p2))
        }
    }
}

//===========================================================================//

struct LineSegment {
    p1: Point2<f32>,
    p2: Point2<f32>,
}

impl LineSegment {
    fn new(p1: Point2<f32>, p2: Point2<f32>) -> LineSegment {
        LineSegment { p1, p2 }
    }

    /// Given the endpoints of a linear path, returns the line segment point
    /// intersected by the path, if any.
    fn intersection(&self, start: Point2<f32>, end: Point2<f32>)
                    -> Option<Point2<f32>> {
        let segment_delta = self.p2 - self.p1;
        let path_delta = end - start;
        let denom = cross(path_delta, segment_delta);
        // Make sure the line segment isn't parallel to the path.
        if denom == 0.0 {
            return None;
        }
        let rel = self.p1 - start;
        // Make sure that the path hits the line between the two vertices.
        let u = cross(rel, path_delta) / denom;
        if u < 0.0 || u >= 1.0 {
            return None;
        }
        // Make sure that the path hits the line segment within the bounds of
        // the path.
        let t = cross(rel, segment_delta) / denom;
        if t < 0.0 || t > 1.0 {
            return None;
        }
        return Some(start + path_delta * t);
    }
}

//===========================================================================//

fn cross(v1: Vector2<f32>, v2: Vector2<f32>) -> f32 {
    v1.x * v2.y - v1.y * v2.x
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{LineSegment, Polygon};
    use cgmath::Point2;

    #[test]
    fn line_segment_intersection() {
        let line = LineSegment::new(Point2::new(1.0, 1.0),
                                    Point2::new(5.0, 3.0));
        assert_eq!(line.intersection(Point2::new(1.0, 2.0),
                                     Point2::new(5.0, 4.0)),
                   None);
        assert_eq!(line.intersection(Point2::new(0.0, 5.0),
                                     Point2::new(0.0, 0.0)),
                   None);
        assert_eq!(line.intersection(Point2::new(2.0, 5.0),
                                     Point2::new(2.0, 2.0)),
                   None);
        assert_eq!(line.intersection(Point2::new(1.0, 3.0),
                                     Point2::new(5.0, 1.0)),
                   Some(Point2::new(3.0, 2.0)));
        assert_eq!(line.intersection(Point2::new(2.0, 5.0),
                                     Point2::new(2.0, 0.0)),
                   Some(Point2::new(2.0, 1.5)));
    }

    #[test]
    fn polygon_edges() {
        let polygon = Polygon::new(vec![
            Point2::new(-1.0, -1.0),
            Point2::new(2.0, -1.0),
            Point2::new(-1.0, 2.0),
        ]);
        let edges: Vec<LineSegment> = polygon.edges().collect();
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn polygon_edge_intersection() {
        let polygon = Polygon::new(vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(1.0, 3.0),
        ]);
        assert_eq!(polygon.edge_intersection(Point2::new(5.0, 2.0),
                                             Point2::new(-1.0, 2.0)),
                   Some((1, Point2::new(2.0, 2.0))));
        assert_eq!(polygon.edge_intersection(Point2::new(-1.0, 2.0),
                                             Point2::new(5.0, 2.0)),
                   Some((2, Point2::new(1.0, 2.0))));
        assert_eq!(polygon.edge_intersection(Point2::new(5.0, 2.0),
                                             Point2::new(-1.0, 10.0)),
                   None);
    }
}

//===========================================================================//
