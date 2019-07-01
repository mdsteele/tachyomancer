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
use tachy::geom::{Color3, Color4};
use tachy::gl::{Primitive, VertexArray, VertexBuffer};
use tachy::gui::Resources;
use tachy::state::{WireColor, WireSize};

//===========================================================================//

const WIRE_COLOR_ERROR: Color3 = Color3::new(1.0, 0.0, 0.0);
const WIRE_COLOR_UNKNOWN: Color3 = Color3::new(0.65, 0.65, 0.65);

const VERTICES_PER_WIRE_SIZE: usize = 52;

// The cosine of 67.5 degrees:
const COS_67_5: f32 = 0.38268343236508984;

// The semi-thickness, low texture param, mid texture param, and high texture
// param for each wire size:
#[cfg_attr(rustfmt, rustfmt_skip)]
const SIZES: &[(f32, f32, f32, f32)] = &[
    ( 5./32.,  0./128.,   5./128.,  10./128.), // 0-bit
    ( 6./32., 10./128.,  16./128.,  22./128.), // 1-bit
    ( 8./32., 22./128.,  30./128.,  38./128.), // 2-bit
    (11./32., 38./128.,  49./128.,  60./128.), // 4-bit
    (14./32., 60./128.,  74./128.,  88./128.), // 8-bit
    (18./32., 88./128., 106./128., 124./128.), // 16-bit
];

#[cfg_attr(rustfmt, rustfmt_skip)]
fn generate_wire_vertex_buffer() -> VertexBuffer<f32> {
    let data_len = 3 * VERTICES_PER_WIRE_SIZE * SIZES.len();
    let mut data = Vec::with_capacity(data_len);
    for &(st, outer, inner, outer2) in SIZES {
        // Stub (east):
        data.extend_from_slice(&[
            28./32., 0.0,       inner,
            1.0,     0.0,       inner,
            1.0,     st,        outer,
            28./32., st,        outer,
            24./32., 0.5 * st,  outer,
            24./32., -0.5 * st, outer,
            28./32., -st,       outer,
            1.0,     -st,       outer,
            1.0,     0.0,       inner,
        ]);
        // Straight (horz):
        data.extend_from_slice(&[
            -1.0, st,  outer,
            1.0,  st,  outer,
            -1.0, -st, outer2,
            1.0,  -st, outer2,
        ]);
        // Corner (south and east):
        data.extend_from_slice(&[
            -st,     1.0,      outer,
            st,      1.0,      outer2,
            -st,     25./32. - COS_67_5 * st, outer,
            st,      25./32. + COS_67_5 * st, outer2,
            25./32. - COS_67_5 * st, -st,     outer,
            25./32. + COS_67_5 * st, st,      outer2,
            1.0,     -st,      outer,
            1.0,     st,       outer2,
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
                     color: WireColor, size: WireSize, hilight: &Color4) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        shader.set_hilight_color(hilight);
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 0;
        self.varray.draw(Primitive::TriangleFan, start, 9);
    }

    /// Draws a horizontal straight wire in the box from (-1, -1) to (1, 1).
    pub fn draw_straight(&self, resources: &Resources,
                         matrix: &Matrix4<f32>, color: WireColor,
                         size: WireSize, hilight: &Color4) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        shader.set_hilight_color(hilight);
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 9;
        self.varray.draw(Primitive::TriangleStrip, start, 4);
    }

    /// Draws a south/east wire corner in the box from (-1, -1) to (1, 1).
    pub fn draw_turn(&self, resources: &Resources, matrix: &Matrix4<f32>,
                     color: WireColor, size: WireSize, hilight: &Color4) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        shader.set_hilight_color(hilight);
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 13;
        self.varray.draw(Primitive::TriangleStrip, start, 8);
    }

    /// Draws a south/east/north wire tee in the box from (-1, -1) to (1, 1).
    pub fn draw_tee(&self, resources: &Resources, matrix: &Matrix4<f32>,
                    color: WireColor, size: WireSize, hilight: &Color4) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        shader.set_hilight_color(hilight);
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 21;
        self.varray.draw(Primitive::TriangleFan, start, 13);
    }

    /// Draws a wire cross in the box from (-1, -1) to (1, 1).
    pub fn draw_cross(&self, resources: &Resources, matrix: &Matrix4<f32>,
                      color: WireColor, size: WireSize, hilight: &Color4) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        shader.set_hilight_color(hilight);
        resources.textures().wire().bind();
        self.varray.bind();
        let start = wire_size_start(size) + 34;
        self.varray.draw(Primitive::TriangleFan, start, 18);
    }
}

fn wire_color(color: WireColor) -> &'static Color3 {
    match color {
        WireColor::Unknown => &WIRE_COLOR_UNKNOWN,
        WireColor::Ambiguous => &WIRE_COLOR_ERROR,
        WireColor::Behavior => &Color3::ORANGE3,
        WireColor::Event => &Color3::CYAN3,
    }
}

//===========================================================================//
