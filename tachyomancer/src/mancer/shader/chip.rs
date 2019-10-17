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

use crate::mancer::gl::{
    IndexBuffer, Primitive, Shader, ShaderProgram, ShaderSampler, ShaderType,
    ShaderUniform, Texture2D, VertexArray, VertexBuffer,
};
use cgmath::Matrix4;
use tachy::geom::{Color3, Color4, Rect, RectSize};

//===========================================================================//

const CHIP_VERT_CODE: &[u8] = include_bytes!("chip.vert");
const CHIP_GEOM_CODE: &[u8] = include_bytes!("chip.geom");
const CHIP_FRAG_CODE: &[u8] = include_bytes!("chip.frag");

const TEX_MIN: f32 = 1.0 / 128.0;
const TEX_MAX: f32 = 127.0 / 128.0;

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
    2, 1, 0,   5, 4, 3,  8, 7, 6,  11, 10, 9, // corners
    1, 2, 3,   3, 2, 5,   // top edge
    4, 5, 8,   4, 8, 6,   // right edge
    7, 8, 9,   9, 8, 11,  // bottom edge
    0, 10, 2,  2, 10, 11, // left edge
    2, 11, 5,  5, 11, 8,  // face
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const BASIC_VERTEX_DATA: &[f32] = &[
     0.00,  0.06, 0.05,  TEX_MIN, TEX_MIN,
     0.06,  0.00, 0.05,  TEX_MIN, TEX_MIN,
     0.06,  0.06, 0.10,  TEX_MIN, TEX_MIN,
    -0.06,  0.00, 0.05,  TEX_MAX, TEX_MIN,
     0.00,  0.06, 0.05,  TEX_MAX, TEX_MIN,
    -0.06,  0.06, 0.10,  TEX_MAX, TEX_MIN,
     0.00, -0.06, 0.05,  TEX_MAX, TEX_MAX,
    -0.06,  0.00, 0.05,  TEX_MAX, TEX_MAX,
    -0.06, -0.06, 0.10,  TEX_MAX, TEX_MAX,
     0.06,  0.00, 0.05,  TEX_MIN, TEX_MAX,
     0.00, -0.06, 0.05,  TEX_MIN, TEX_MAX,
     0.06, -0.06, 0.10,  TEX_MIN, TEX_MAX,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const BASIC_CORNER_DATA: &[u8] = &[
    0, 0,  0, 0,  0, 0,
    1, 0,  1, 0,  1, 0,
    1, 1,  1, 1,  1, 1,
    0, 1,  0, 1,  0, 1,
];

const BASIC_PLASTIC_COLOR: Color4 = Color4::new(0.4, 0.4, 0.4, 1.0);

//===========================================================================//

// 0--3
// |  |
// 1--2

#[cfg_attr(rustfmt, rustfmt_skip)]
const COMMENT_INDEX_DATA: &[u8] = &[
    0, 1, 2,  2, 3, 0,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const COMMENT_VERTEX_DATA: &[f32] = &[
    0.00,  0.00, 0.01,  TEX_MIN, TEX_MIN,
    0.00,  0.00, 0.01,  TEX_MIN, TEX_MAX,
    0.00,  0.00, 0.01,  TEX_MAX, TEX_MAX,
    0.00,  0.00, 0.01,  TEX_MAX, TEX_MIN,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const COMMENT_CORNER_DATA: &[u8] = &[
    0, 0,
    0, 1,
    1, 1,
    1, 0,
];

//===========================================================================//

struct ChipModel {
    ibuffer: IndexBuffer<u8>,
    _vertex_vbuffer: VertexBuffer<f32>,
    _corner_vbuffer: VertexBuffer<u8>,
    varray: VertexArray,
}

impl ChipModel {
    fn new(
        index_data: &[u8],
        vertex_data: &[f32],
        corner_data: &[u8],
    ) -> ChipModel {
        let ibuffer = IndexBuffer::new(index_data);
        let vertex_vbuffer = VertexBuffer::new(vertex_data);
        let corner_vbuffer = VertexBuffer::new(corner_data);
        let varray = VertexArray::new(3);
        varray.bind();
        vertex_vbuffer.attribf(0, 3, 5, 0);
        vertex_vbuffer.attribf(1, 2, 5, 3);
        corner_vbuffer.attribi(2, 2, 0, 0);
        ChipModel {
            ibuffer,
            _vertex_vbuffer: vertex_vbuffer,
            _corner_vbuffer: corner_vbuffer,
            varray,
        }
    }

    fn draw(&self) {
        self.varray.bind();
        self.varray.draw_elements(Primitive::Triangles, &self.ibuffer);
    }
}

//===========================================================================//

pub struct ChipShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    chip_size: ShaderUniform<RectSize<f32>>,
    tex_rect: ShaderUniform<Rect<f32>>,
    plastic_color: ShaderUniform<Color4>,
    icon_color: ShaderUniform<Color3>,
    icon_texture: ShaderSampler<Texture2D>,
    basic_model: ChipModel,
    comment_model: ChipModel,
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
        let chip_size = program.get_uniform("ChipSize")?;
        let tex_rect = program.get_uniform("TexRect")?;
        let plastic_color = program.get_uniform("PlasticColor")?;
        let icon_color = program.get_uniform("IconColor")?;
        let icon_texture = program.get_sampler(0, "IconTexture")?;

        Ok(ChipShader {
            program,
            mvp,
            chip_size,
            tex_rect,
            plastic_color,
            icon_color,
            icon_texture,
            basic_model: ChipModel::new(
                BASIC_INDEX_DATA,
                BASIC_VERTEX_DATA,
                BASIC_CORNER_DATA,
            ),
            comment_model: ChipModel::new(
                COMMENT_INDEX_DATA,
                COMMENT_VERTEX_DATA,
                COMMENT_CORNER_DATA,
            ),
        })
    }

    /// The matrix should place the origin at the center of the chip.
    pub fn draw_basic(
        &self,
        matrix: &Matrix4<f32>,
        size: RectSize<f32>,
        icon_index: u32,
        icon_color: Color3,
        icon_texture: &Texture2D,
    ) {
        self.bind(
            matrix,
            &size,
            &BASIC_PLASTIC_COLOR,
            icon_index,
            &icon_color,
            icon_texture,
        );
        self.basic_model.draw();
    }

    /// The matrix should place the origin at the center of the chip.
    pub fn draw_comment(
        &self,
        matrix: &Matrix4<f32>,
        size: RectSize<f32>,
        icon_index: u32,
        icon_color: Color3,
        icon_texture: &Texture2D,
    ) {
        self.bind(
            matrix,
            &size,
            &Color4::TRANSPARENT,
            icon_index,
            &icon_color,
            icon_texture,
        );
        self.comment_model.draw();
    }

    fn bind(
        &self,
        matrix: &Matrix4<f32>,
        size: &RectSize<f32>,
        plastic_color: &Color4,
        icon_index: u32,
        icon_color: &Color3,
        icon_texture: &Texture2D,
    ) {
        let (tex_row, tex_col) = (icon_index / 8, icon_index % 8);
        let tex_rect = Rect::new(
            0.125 * (tex_col as f32),
            0.125 * (tex_row as f32),
            0.125,
            0.125,
        );
        self.program.bind();
        self.mvp.set(matrix);
        self.chip_size.set(size);
        self.tex_rect.set(&tex_rect);
        self.plastic_color.set(plastic_color);
        self.icon_color.set(icon_color);
        self.icon_texture.set(icon_texture);
    }
}

//===========================================================================//
