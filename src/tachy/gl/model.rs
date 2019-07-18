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

use super::index::IndexBuffer;
use super::vertex::{Primitive, VertexArray, VertexBuffer};
use cgmath::{Angle, InnerSpace, Point3, Quaternion, Rad, Rotation, Rotation3,
             Vector3};
use tachy::geom::Color3;

//===========================================================================//

pub struct Model {
    varray: VertexArray,
    _vbuffer: VertexBuffer<f32>,
    ibuffer: IndexBuffer<u16>,
}

impl Model {
    pub fn draw(&self) {
        self.varray.bind();
        self.varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }
}

//===========================================================================//

pub struct ModelBuilder {
    indices: Vec<u16>,
    vertices: Vec<f32>,
}

impl ModelBuilder {
    pub fn new() -> ModelBuilder {
        ModelBuilder {
            indices: Vec::new(),
            vertices: Vec::new(),
        }
    }

    fn start_index(&self) -> u16 {
        debug_assert!(self.indices.len() % 3 == 0);
        debug_assert!(self.vertices.len() % 9 == 0);
        (self.vertices.len() / 9) as u16
    }

    fn push_vertex(&mut self, vertex: Point3<f32>, normal: Vector3<f32>,
                   color: Color3) {
        // TODO: Apply current matrix/inversion
        self.vertices.extend(
            &[
                vertex.x,
                vertex.y,
                vertex.z,
                normal.x,
                normal.y,
                normal.z,
                color.r,
                color.g,
                color.b,
            ],
        );
    }

    fn push_triangle(&mut self, i0: u16, i1: u16, i2: u16) {
        // TODO: If inverted, flip ordering
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }

    pub fn cylinder(&mut self, c1: Point3<f32>, c2: Point3<f32>,
                    radius: f32, num_faces: u16, color: Color3) {
        debug_assert!(num_faces >= 3);
        let start = self.start_index();
        let mut prev1 = start + 2 * (num_faces - 1);
        let mut prev2 = prev1 + 1;
        let axis_unit = (c2 - c1).normalize();
        let perp_unit = perpendicular_unit(axis_unit);
        let theta_step = Rad::full_turn() / (num_faces as f32);
        for face in 0..num_faces {
            // Vertices:
            let theta = theta_step * (face as f32);
            let normal = Quaternion::from_axis_angle(axis_unit, theta)
                .rotate_vector(perp_unit);
            let spoke = normal * radius;
            self.push_vertex(c1 + spoke, normal, color);
            self.push_vertex(c2 + spoke, normal, color);
            // Indices:
            let curr1 = start + 2 * face;
            let curr2 = curr1 + 1;
            self.push_triangle(prev1, curr1, curr2);
            self.push_triangle(prev1, curr2, prev2);
            prev1 = curr1;
            prev2 = curr2;
        }
    }

    pub fn sphere(&mut self, center: Point3<f32>, radius: f32,
                  num_steps: u16, color: Color3) {
        debug_assert!(num_steps >= 3);
        let start = self.start_index();

        let north_pole = start;
        self.push_vertex(center + Vector3::unit_y() * radius,
                         Vector3::unit_y(),
                         color);

        let south_pole = start + 1;
        self.push_vertex(center - Vector3::unit_y() * radius,
                         -Vector3::unit_y(),
                         color);

        let longitude_step = Rad::full_turn() / (num_steps as f32);
        let latitude_step = Rad::turn_div_2() / (num_steps as f32);
        let mut prev_long_start = start + 2 +
            (num_steps - 1) * (num_steps - 1);
        for longitude_index in 0..num_steps {
            let curr_long_start = start + 2 +
                (num_steps - 1) * longitude_index;
            let longitude = longitude_step * (longitude_index as f32);

            // Vertices:
            for latitude_index in 1..num_steps {
                let latitude = latitude_step * (latitude_index as f32) -
                    Rad::turn_div_4();
                let normal =
                    Quaternion::from_angle_y(longitude)
                        .rotate_vector(Quaternion::from_angle_z(latitude)
                                           .rotate_vector(Vector3::unit_x()));
                let vertex = center + normal * radius;
                self.push_vertex(vertex, normal, color);
            }

            // Indices:
            self.push_triangle(south_pole, curr_long_start, prev_long_start);
            for index in 0..(num_steps - 2) {
                self.push_triangle(curr_long_start + index,
                                   curr_long_start + index + 1,
                                   prev_long_start + index);
                self.push_triangle(curr_long_start + index + 1,
                                   prev_long_start + index + 1,
                                   prev_long_start + index);
            }
            self.push_triangle(north_pole,
                               prev_long_start + num_steps - 2,
                               curr_long_start + num_steps - 2);

            prev_long_start = curr_long_start;
        }
    }

    pub fn build(self) -> Model {
        let ibuffer = IndexBuffer::new(&self.indices);
        let vbuffer = VertexBuffer::new(&self.vertices);
        let varray = VertexArray::new(3);
        varray.bind();
        vbuffer.attribf(0, 3, 9, 0);
        vbuffer.attribf(1, 3, 9, 3);
        vbuffer.attribf(2, 2, 9, 6);
        Model {
            varray,
            _vbuffer: vbuffer,
            ibuffer,
        }
    }
}

//===========================================================================//

/// Returns an arbitrary unit vector that is perpendicular to the given vector.
fn perpendicular_unit(vec: Vector3<f32>) -> Vector3<f32> {
    let cx = Vector3::unit_x().cross(vec);
    let cy = Vector3::unit_y().cross(vec);
    if cx.magnitude() > cy.magnitude() {
        cx.normalize()
    } else {
        cy.normalize()
    }
}

//===========================================================================//
