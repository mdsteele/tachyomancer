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

// The cosine of 67.5 degrees:
const COS_67_5: f32 = 0.38268343236508984;

// Texture param for the outside of a 2-bit wire:
const OUT_2: f32 = 10.0 / 128.0;
// Texture param for the inside of a 2-bit wire:
const IN_2: f32 = 20.0 / 128.0;
// Semi-thickness of a 2-bit wire:
const ST_2: f32 = 5.0 / 32.0;

#[cfg_attr(rustfmt, rustfmt_skip)]
const WIRE_VERTEX_DATA: &[f32] = &[
    // 2-bit stub (east):
    1.0,     0.0,           IN_2,
    1.0,     ST_2,          OUT_2,
    25./32., 0.5 * ST_2,    OUT_2,
    25./32., -0.5 * ST_2,   OUT_2,
    1.0,     -ST_2,         OUT_2,
    // 2-bit straight (horz):
    -1.0, ST_2,    OUT_2,
    1.0,  ST_2,    OUT_2,
    -1.0, 0.0,     IN_2,
    1.0,  0.0,     IN_2,
    -1.0, -ST_2,   OUT_2,
    1.0,  -ST_2,   OUT_2,
    // 2-bit corner (south and east):
    -ST_2,   1.0,      OUT_2,
    0.0,     1.0,      IN_2,
    -ST_2,   25./32. - COS_67_5 * ST_2,  OUT_2,
    0.0,     25./32.,  IN_2,
    25./32. - COS_67_5 * ST_2, -ST_2,    OUT_2,
    25./32., 0.0,      IN_2,
    1.0,     -ST_2,  OUT_2,
    28./32., 0.0,      IN_2,
    1.0,     ST_2,   OUT_2,
    25./32., 0.0,      IN_2,
    25./32. + COS_67_5 * ST_2, ST_2,     OUT_2,
    0.0,     25./32.,  IN_2,
    ST_2,    25./32. + COS_67_5 * ST_2,  OUT_2,
    0.0,     1.0,      IN_2,
    ST_2,    1.0,      OUT_2,
    // 2-bit tee (south/east/north):
    0.0,   0.0,     IN_2,
    1.0,   ST_2,    OUT_2,
    1.0,   0.0,     IN_2,
    1.0,   -ST_2,   OUT_2,
    ST_2,  -ST_2,   OUT_2,
    ST_2,  -1.0,    OUT_2,
    0.0,   -1.0,    IN_2,
    -ST_2, -1.0,    OUT_2,
    -ST_2, 1.0,     OUT_2,
    0.0,   1.0,     IN_2,
    ST_2,  1.0,     OUT_2,
    ST_2,  ST_2,    OUT_2,
    1.0,   ST_2,    OUT_2,
    // 2-bit cross:
    0.0,   0.0,     IN_2,
    1.0,   ST_2,    OUT_2,
    1.0,   0.0,     IN_2,
    1.0,   -ST_2,   OUT_2,
    ST_2,  -ST_2,   OUT_2,
    ST_2,  -1.0,    OUT_2,
    0.0,   -1.0,    IN_2,
    -ST_2, -1.0,    OUT_2,
    -ST_2, -ST_2,   OUT_2,
    -1.0,  -ST_2,   OUT_2,
    -1.0,  0.0,     IN_2,
    -1.0,  ST_2,    OUT_2,
    -ST_2, ST_2,    OUT_2,
    -ST_2, 1.0,     OUT_2,
    0.0,   1.0,     IN_2,
    ST_2,  1.0,     OUT_2,
    ST_2,  ST_2,    OUT_2,
    1.0,   ST_2,    OUT_2,
    // TODO: add other wire sizes
];

//===========================================================================//

pub struct WireModel {
    varray: VertexArray,
    _vbuffer: VertexBuffer<f32>,
}

impl WireModel {
    pub fn new() -> WireModel {
        let varray = VertexArray::new(2);
        let vbuffer = VertexBuffer::new(WIRE_VERTEX_DATA);
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
                     color: WireColor, _size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        // TODO use wire size
        self.varray.draw(Primitive::TriangleFan, 0, 5);
    }

    /// Draws a horizontal straight wire in the box from (-1, -1) to (1, 1).
    pub fn draw_straight(&self, resources: &Resources,
                         matrix: &Matrix4<f32>, color: WireColor,
                         _size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        // TODO use wire size
        self.varray.draw(Primitive::TriangleStrip, 5, 6);
    }

    /// Draws a south/east wire corner in the box from (-1, -1) to (1, 1).
    pub fn draw_corner(&self, resources: &Resources, matrix: &Matrix4<f32>,
                       color: WireColor, _size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        // TODO: use wire size
        self.varray.draw(Primitive::TriangleStrip, 11, 15);
    }

    /// Draws a south/east/north wire tee in the box from (-1, -1) to (1, 1).
    pub fn draw_tee(&self, resources: &Resources, matrix: &Matrix4<f32>,
                    color: WireColor, _size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        // TODO use wire size
        self.varray.draw(Primitive::TriangleFan, 26, 13);
    }

    /// Draws a wire cross in the box from (-1, -1) to (1, 1).
    pub fn draw_cross(&self, resources: &Resources, matrix: &Matrix4<f32>,
                      color: WireColor, _size: WireSize) {
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(matrix);
        shader.set_wire_color(wire_color(color));
        resources.textures().wire().bind();
        self.varray.bind();
        // TODO use wire size
        self.varray.draw(Primitive::TriangleFan, 39, 18);
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
