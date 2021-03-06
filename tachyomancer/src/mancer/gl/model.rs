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
use cgmath::{
    Angle, InnerSpace, Matrix4, Point2, Point3, Quaternion, Rad, Rotation,
    Rotation3, SquareMatrix, Vector3,
};
use tachy::geom::{Color3, RectSize};

//===========================================================================//

const FLOATS_PER_VERTEX: usize = 11;

const ORIGIN: Point3<f32> = Point3 { x: 0.0, y: 0.0, z: 0.0 };

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
        ModelBuilder { indices: Vec::new(), vertices: Vec::new() }
    }

    pub fn context(&mut self) -> ModelBuilderContext {
        self.with_transform(Matrix4::identity())
    }

    pub fn with_transform(
        &mut self,
        matrix: Matrix4<f32>,
    ) -> ModelBuilderContext {
        ModelBuilderContext {
            indices: &mut self.indices,
            vertices: &mut self.vertices,
            matrix,
        }
    }

    pub fn plane(
        &mut self,
        center: Point3<f32>,
        size: RectSize<f32>,
        normal: Vector3<f32>,
        color: Color3,
    ) {
        self.context().plane(center, size, normal, color);
    }

    pub fn build(self) -> Model {
        let ibuffer = IndexBuffer::new(&self.indices);
        let vbuffer = VertexBuffer::new(&self.vertices);
        let varray = VertexArray::new(4);
        varray.bind();
        vbuffer.attribf(0, 3, FLOATS_PER_VERTEX, 0);
        vbuffer.attribf(1, 3, FLOATS_PER_VERTEX, 3);
        vbuffer.attribf(2, 3, FLOATS_PER_VERTEX, 6);
        vbuffer.attribf(3, 2, FLOATS_PER_VERTEX, 9);
        Model { varray, _vbuffer: vbuffer, ibuffer }
    }
}

//===========================================================================//

pub struct ModelBuilderContext<'a> {
    indices: &'a mut Vec<u16>,
    vertices: &'a mut Vec<f32>,
    matrix: Matrix4<f32>,
}

impl<'a> ModelBuilderContext<'a> {
    fn start_index(&self) -> u16 {
        debug_assert!(self.indices.len() % 3 == 0);
        debug_assert!(self.vertices.len() % FLOATS_PER_VERTEX == 0);
        (self.vertices.len() / FLOATS_PER_VERTEX) as u16
    }

    fn push_vertex(
        &mut self,
        vertex: Point3<f32>,
        normal: Vector3<f32>,
        color: Color3,
        texture_uv: Point2<f32>,
    ) {
        let vertex =
            Point3::from_homogeneous(self.matrix * vertex.to_homogeneous());
        let mut normal = (self.matrix * normal.extend(0.0)).truncate();
        if self.matrix.determinant() < 0.0 {
            normal = -normal;
        }
        let floats = &[
            vertex.x,
            vertex.y,
            vertex.z,
            normal.x,
            normal.y,
            normal.z,
            color.r,
            color.g,
            color.b,
            texture_uv.x,
            texture_uv.y,
        ];
        debug_assert_eq!(floats.len(), FLOATS_PER_VERTEX);
        self.vertices.extend(floats);
    }

    fn push_triangle(&mut self, i0: u16, i1: u16, i2: u16) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }

    pub fn transformed(self, matrix: Matrix4<f32>) -> ModelBuilderContext<'a> {
        ModelBuilderContext {
            indices: self.indices,
            vertices: self.vertices,
            matrix: self.matrix * matrix,
        }
    }

    pub fn with_transform(
        &mut self,
        matrix: Matrix4<f32>,
    ) -> ModelBuilderContext {
        ModelBuilderContext {
            indices: self.indices,
            vertices: self.vertices,
            matrix: self.matrix * matrix,
        }
    }

    pub fn cylinder(
        &mut self,
        c1: Point3<f32>,
        c2: Point3<f32>,
        radius: f32,
        num_faces: u16,
        color: Color3,
    ) {
        debug_assert!(num_faces >= 3);
        let start = self.start_index();
        let axis_unit = (c2 - c1).normalize();
        let perp_unit = perpendicular_unit(axis_unit);
        let theta_step = Rad::full_turn() / (num_faces as f32);
        // Vertices:
        for face in 0..(num_faces + 1) {
            let theta = theta_step * (face as f32);
            let normal = Quaternion::from_axis_angle(axis_unit, theta)
                .rotate_vector(perp_unit);
            let spoke = normal * radius;
            let u = (face as f32) / (num_faces as f32);
            self.push_vertex(c1 + spoke, normal, color, Point2::new(u, 0.0));
            self.push_vertex(c2 + spoke, normal, color, Point2::new(u, 1.0));
        }
        // Indices:
        for face in 0..num_faces {
            let curr1 = start + 2 * face;
            let curr2 = curr1 + 1;
            let next1 = start + 2 * (face + 1);
            let next2 = next1 + 1;
            self.push_triangle(curr1, next1, next2);
            self.push_triangle(curr1, next2, curr2);
        }
    }

    pub fn plane(
        &mut self,
        center: Point3<f32>,
        size: RectSize<f32>,
        normal: Vector3<f32>,
        color: Color3,
    ) {
        let start = self.start_index();
        let normal = normal.normalize();
        let rotation = Quaternion::from_arc(Vector3::unit_z(), normal, None);
        let corners = &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];
        for &(x, y) in corners {
            let vertex = Vector3::new(
                (x - 0.5) * size.width,
                (y - 0.5) * size.height,
                0.0,
            );
            let vertex = center + rotation.rotate_vector(vertex);
            self.push_vertex(vertex, normal, color, Point2::new(x, y));
        }
        self.push_triangle(start + 0, start + 1, start + 2);
        self.push_triangle(start + 2, start + 1, start + 3);
    }

    /// Creates a unit circle, centered at the origin, with its normal in the
    /// positive-Y direction.
    pub fn unit_circle(&mut self, num_steps: u16, color: Color3) {
        debug_assert!(num_steps >= 3);
        let start = self.start_index();
        let step = Rad::full_turn() / (num_steps as f32);
        for index in 0..num_steps {
            let theta = step * (index as f32);
            let vertex = ORIGIN
                + Quaternion::from_angle_y(theta)
                    .rotate_vector(Vector3::unit_x());
            let texture_uv =
                Point2::new(0.5 + vertex.x * 0.5, 0.5 + vertex.z * 0.5);
            self.push_vertex(vertex, Vector3::unit_y(), color, texture_uv);
        }
        for index in 1..(num_steps - 1) {
            self.push_triangle(start, start + index, start + index + 1);
        }
    }

    /// Creates a unit hemisphere, with the sphere center at the origin, and
    /// the hemisphere occupying the positive-Y portion of space.
    pub fn unit_hemisphere(&mut self, num_long_steps: u16, color: Color3) {
        debug_assert!(num_long_steps >= 3);
        let num_lat_steps = (num_long_steps + 1) / 2;
        let start = self.start_index();

        let north_pole = start;
        self.push_vertex(
            ORIGIN + Vector3::unit_y(),
            Vector3::unit_y(),
            color,
            Point2::new(0.5, 1.0),
        );

        // Vertices:
        let longitude_step = Rad::full_turn() / (num_long_steps as f32);
        let latitude_step = Rad::turn_div_4() / (num_lat_steps as f32);
        for longitude_index in 0..(num_long_steps + 1) {
            let longitude = longitude_step * (longitude_index as f32);
            let texture_u = (longitude_index as f32) / (num_long_steps as f32);

            for latitude_index in 1..(num_lat_steps + 1) {
                let latitude = Rad::turn_div_4()
                    - latitude_step * (latitude_index as f32);
                let normal = Quaternion::from_angle_y(longitude)
                    .rotate_vector(
                        Quaternion::from_angle_z(latitude)
                            .rotate_vector(Vector3::unit_x()),
                    );
                let vertex = ORIGIN + normal;
                let texture_v =
                    (latitude_index as f32) / (num_lat_steps as f32);
                let texture_uv = Point2::new(texture_u, texture_v);
                self.push_vertex(vertex, normal, color, texture_uv);
            }
        }

        // Indices:
        for longitude_index in 0..num_long_steps {
            let curr_long_start = start + 1 + num_lat_steps * longitude_index;
            let next_long_start = curr_long_start + num_lat_steps;
            self.push_triangle(north_pole, curr_long_start, next_long_start);
            for latitude_index in 0..(num_lat_steps - 1) {
                self.push_triangle(
                    next_long_start + latitude_index,
                    curr_long_start + latitude_index,
                    next_long_start + latitude_index + 1,
                );
                self.push_triangle(
                    curr_long_start + latitude_index,
                    curr_long_start + latitude_index + 1,
                    next_long_start + latitude_index + 1,
                );
            }
        }
    }

    /// Creates a unit sphere, centered at the origin.
    pub fn unit_sphere(&mut self, num_steps: u16, color: Color3) {
        debug_assert!(num_steps >= 3);
        let start = self.start_index();

        let north_pole = start;
        self.push_vertex(
            ORIGIN + Vector3::unit_y(),
            Vector3::unit_y(),
            color,
            Point2::new(0.5, 1.0),
        );

        let south_pole = start + 1;
        self.push_vertex(
            ORIGIN - Vector3::unit_y(),
            -Vector3::unit_y(),
            color,
            Point2::new(0.5, 0.0),
        );

        // Vertices:
        let longitude_step = Rad::full_turn() / (num_steps as f32);
        let latitude_step = Rad::turn_div_2() / (num_steps as f32);
        for longitude_index in 0..(num_steps + 1) {
            let longitude = longitude_step * (longitude_index as f32);
            let texture_u = (longitude_index as f32) / (num_steps as f32);

            for latitude_index in 1..num_steps {
                let latitude = latitude_step * (latitude_index as f32)
                    - Rad::turn_div_4();
                let normal = Quaternion::from_angle_y(longitude)
                    .rotate_vector(
                        Quaternion::from_angle_z(latitude)
                            .rotate_vector(Vector3::unit_x()),
                    );
                let vertex = ORIGIN + normal;
                let texture_v = (latitude_index as f32) / (num_steps as f32);
                let texture_uv = Point2::new(texture_u, texture_v);
                self.push_vertex(vertex, normal, color, texture_uv);
            }
        }

        // Indices:
        for longitude_index in 0..num_steps {
            let curr_long_start =
                start + 2 + (num_steps - 1) * longitude_index;
            let next_long_start = curr_long_start + num_steps - 1;
            self.push_triangle(south_pole, next_long_start, curr_long_start);
            for index in 0..(num_steps - 2) {
                self.push_triangle(
                    next_long_start + index,
                    next_long_start + index + 1,
                    curr_long_start + index,
                );
                self.push_triangle(
                    next_long_start + index + 1,
                    curr_long_start + index + 1,
                    curr_long_start + index,
                );
            }
            self.push_triangle(
                north_pole,
                curr_long_start + num_steps - 2,
                next_long_start + num_steps - 2,
            );
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
