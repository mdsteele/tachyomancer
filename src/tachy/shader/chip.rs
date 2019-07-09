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
use tachy::geom::{Color3, Rect};
use tachy::gl::{IndexBuffer, Primitive, Shader, ShaderProgram, ShaderType,
                ShaderUniform, VertexArray, VertexBuffer};

//===========================================================================//

const CHIP_VERT_CODE: &[u8] = include_bytes!("chip.vert");
const CHIP_GEOM_CODE: &[u8] = include_bytes!("chip.geom");
const CHIP_FRAG_CODE: &[u8] = include_bytes!("chip.frag");

//===========================================================================//

//    1----3
//   /|    |\
//  0-2----5-4
//  | |    | |
//  | |    | |
// 10-11---8-6
//   \|    |/
//    9----7

#[cfg_attr(rustfmt, rustfmt_skip)]
const BASIC_INDEX_DATA: &[u8] = &[
    2, 1, 0,  5, 4, 3,  8, 7, 6,  11, 10, 9, // corners
    1, 2, 3,  3, 2, 5, // top edge
    4, 5, 8,  4, 8, 6, // right edge
    7, 8, 9,  9, 8, 11, // bottom edge
    0, 2, 10,  2, 10, 11, // left edge
    2, 11, 5,  5, 11, 8, // face
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const BASIC_VERTEX_DATA: &[f32] = &[
    0.00, 0.08, 0.00,  0.001, 0.001,
    0.08, 0.00, 0.00,  0.001, 0.001,
    0.08, 0.08, 0.05,  0.001, 0.001,
    0.92, 0.00, 0.00,  0.999, 0.001,
    1.00, 0.08, 0.00,  0.999, 0.001,
    0.92, 0.08, 0.05,  0.999, 0.001,
    1.00, 0.92, 0.00,  0.999, 0.999,
    0.92, 1.00, 0.00,  0.999, 0.999,
    0.92, 0.92, 0.05,  0.999, 0.999,
    0.08, 1.00, 0.00,  0.001, 0.999,
    0.00, 0.92, 0.00,  0.001, 0.999,
    0.08, 0.92, 0.05,  0.001, 0.999,
];

//===========================================================================//

pub struct ChipShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    tex_rect: ShaderUniform<Rect<f32>>,
    icon_color: ShaderUniform<Color3>,
    basic_ibuffer: IndexBuffer<u8>,
    _basic_vbuffer: VertexBuffer<f32>,
    basic_varray: VertexArray,
}

impl ChipShader {
    pub(super) fn new() -> Result<ChipShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "chip.vert", CHIP_VERT_CODE)?;
        let geom =
            Shader::new(ShaderType::Geometry, "chip.geom", CHIP_GEOM_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "chip.frag", CHIP_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &geom, &frag])?;

        let mvp = program.get_uniform("MVP")?;
        let tex_rect = program.get_uniform("TexRect")?;
        let icon_color = program.get_uniform("IconColor")?;

        let basic_ibuffer = IndexBuffer::new(BASIC_INDEX_DATA);
        let basic_vbuffer = VertexBuffer::new(BASIC_VERTEX_DATA);
        let basic_varray = VertexArray::new(2);
        basic_varray.bind();
        basic_vbuffer.attribf(0, 3, 5, 0);
        basic_vbuffer.attribf(1, 2, 5, 3);

        let shader = ChipShader {
            program,
            mvp,
            tex_rect,
            icon_color,
            basic_ibuffer,
            _basic_vbuffer: basic_vbuffer,
            basic_varray,
        };
        Ok(shader)
    }

    pub fn draw_basic(&self, matrix: &Matrix4<f32>, icon_index: u32,
                      icon_color: Color3) {
        let (tex_row, tex_col) = (icon_index / 8, icon_index % 8);
        let tex_rect = Rect::new(0.125 * (tex_col as f32),
                                 0.125 * (tex_row as f32),
                                 0.125,
                                 0.125);
        self.program.bind();
        self.mvp.set(matrix);
        self.tex_rect.set(&tex_rect);
        self.icon_color.set(&icon_color);
        self.basic_varray.bind();
        self.basic_varray
            .draw_elements(Primitive::Triangles, &self.basic_ibuffer);
    }
}

//===========================================================================//
