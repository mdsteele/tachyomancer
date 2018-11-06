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

use cgmath::Matrix4;
use tachy::gl::{Primitive, VertexArray, VertexBuffer};
use tachy::gui::Resources;
use tachy::state::{WireColor, WireSize};

//===========================================================================//

const VERTICES_PER_WIRE_SIZE: usize = 57;

// The cosine of 67.5 degrees:
const COS_67_5: f32 = 0.38268343236508984;

// The semi-thickness, inner texture param, and outer texture param for each
// wire size:
#[cfg_attr(rustfmt, rustfmt_skip)]
const SIZES: &[(f32, f32, f32)] = &[
    ( 2./32.,  0./128.,   4./128.), // 0-bit
    ( 3./32.,  4./128.,  10./128.), // 1-bit
    ( 5./32., 10./128.,  20./128.), // 2-bit
    ( 8./32., 20./128.,  36./128.), // 4-bit
    (11./32., 36./128.,  58./128.), // 8-bit
    (15./32., 58./128.,  88./128.), // 16-bit
    (20./32., 88./128., 128./128.), // 32-bit
];

#[cfg_attr(rustfmt, rustfmt_skip)]
fn generate_wire_vertex_buffer() -> VertexBuffer<f32> {
    let data_len = 3 * VERTICES_PER_WIRE_SIZE * SIZES.len();
    let mut data = Vec::with_capacity(data_len);
    for &(st, outer, inner) in SIZES {
        // Stub (east):
        data.extend_from_slice(&[
            1.0,     0.0,       inner,
            1.0,     st,        outer,
            25./32., 0.5 * st,  outer,
            25./32., -0.5 * st, outer,
            1.0,     -st,       outer,
        ]);
        // Straight (horz):
        data.extend_from_slice(&[
            -1.0, st,  outer,
            1.0,  st,  outer,
            -1.0, 0.0, inner,
            1.0,  0.0, inner,
            -1.0, -st, outer,
            1.0,  -st, outer,
        ]);
        // Corner (south and east):
        data.extend_from_slice(&[
            -st,     1.0,      outer,
            0.0,     1.0,      inner,
            -st,     25./32. - COS_67_5 * st, outer,
            0.0,     25./32.,  inner,
            25./32. - COS_67_5 * st, -st,     outer,
            25./32., 0.0,      inner,
            1.0,     -st,      outer,
            28./32., 0.0,      inner,
            1.0,     st,       outer,
            25./32., 0.0,      inner,
            25./32. + COS_67_5 * st, st,      outer,
            0.0,     25./32.,  inner,
            st,      25./32. + COS_67_5 * st, outer,
            0.0,     1.0,      inner,
            st,      1.0,      outer,
        ]);
        // Tee (south/east/north):
        data.extend_from_slice(&[
            0.0,  0.0,  inner,
            1.0,  st,   outer,
            1.0,  0.0,  inner,
            1.0,  -st,  outer,
            st,   -st,  outer,
            st,   -1.0, outer,
            0.0,  -1.0, inner,
            -st,  -1.0, outer,
            -st,  1.0,  outer,
            0.0,  1.0,  inner,
            st,   1.0,  outer,
            st,   st,   outer,
            1.0,  st,   outer,
        ]);
        // Cross:
        data.extend_from_slice(&[
            0.0,  0.0,  inner,
            1.0,  st,   outer,
            1.0,  0.0,  inner,
            1.0,  -st,  outer,
            st,   -st,  outer,
            st,   -1.0, outer,
            0.0,  -1.0, inner,
            -st,  -1.0, outer,
            -st,  -st,  outer,
            -1.0, -st,  outer,
            -1.0, 0.0,  inner,
            -1.0, st,   outer,
            -st,  st,   outer,
            -st,  1.0,  outer,
            0.0,  1.0,  inner,
            st,   1.0,  outer,
            st,   st,   outer,
            1.0,  st,   outer,
        ]);
    }
    debug_assert_eq!(data.len(), data_len);
    VertexBuffer::new(&data)
}

fn wire_size_start(size: WireSize) -> usize {
    let index = match size {
        WireSize::Zero => 0,
        WireSize::One => 1,
        WireSize::Two => 2,
        WireSize::Four => 3,
        WireSize::Eight => 4,
        WireSize::Sixteen => 5,
        WireSize::ThirtyTwo => 6,
    };
    index * VERTICES_PER_WIRE_SIZE
}

//===========================================================================//

pub struct WireModel {
    varray: VertexArray,
    _vbuffer: VertexBuffer<f32>,
}

impl WireModel {
    pub fn new() -> WireModel {
        let varray = VertexArray::new(2);
        let vbuffer = generate_wire_vertex_buffer();
        varray.bind();
        vbuffer.attribf(0, 2, 3, 0);
        vbuffer.attribf(1, 1, 3, 2);
        WireModel {
            varray,
            _vbuffer: vbuffer,
        }
    }

    /// Draws an east wire stub in the box from (-1, -1) to (1, 1).
    pub fn draw_stub(&self, resources: &Resources, matrix: &Matrix4<f32>,
                     color: WireColor, size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 0;
        self.varray.draw(Primitive::TriangleFan, start, 5);
    }

    /// Draws a horizontal straight wire in the box from (-1, -1) to (1, 1).
    pub fn draw_straight(&self, resources: &Resources,
                         matrix: &Matrix4<f32>, color: WireColor,
                         size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 5;
        self.varray.draw(Primitive::TriangleStrip, start, 6);
    }

    /// Draws a south/east wire corner in the box from (-1, -1) to (1, 1).
    pub fn draw_corner(&self, resources: &Resources, matrix: &Matrix4<f32>,
                       color: WireColor, size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 11;
        self.varray.draw(Primitive::TriangleStrip, start, 15);
    }

    /// Draws a south/east/north wire tee in the box from (-1, -1) to (1, 1).
    pub fn draw_tee(&self, resources: &Resources, matrix: &Matrix4<f32>,
                    color: WireColor, size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 26;
        self.varray.draw(Primitive::TriangleFan, start, 13);
    }

    /// Draws a wire cross in the box from (-1, -1) to (1, 1).
    pub fn draw_cross(&self, resources: &Resources, matrix: &Matrix4<f32>,
                      color: WireColor, size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 39;
        self.varray.draw(Primitive::TriangleFan, start, 18);
    }
}

fn wire_color(color: WireColor) -> (f32, f32, f32) {
    match color {
        WireColor::Unknown => (0.5, 0.5, 0.5),
        WireColor::Error => (1.0, 0.0, 0.0),
        WireColor::Behavior => (1.0, 0.5, 0.0),
        WireColor::Event => (0.0, 1.0, 1.0),
    }
}

//===========================================================================//
