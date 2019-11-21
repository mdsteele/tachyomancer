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
    Primitive, Shader, ShaderProgram, ShaderSampler, ShaderType,
    ShaderUniform, Texture2D, VertexArray, VertexBuffer,
};
use cgmath::Matrix4;
use tachy::geom::{Color4, MatrixExt, Rect};

//===========================================================================//

const DIAGRAM_VERT_CODE: &[u8] = include_bytes!("diagram.vert");
const DIAGRAM_FRAG_CODE: &[u8] = include_bytes!("diagram.frag");

//===========================================================================//

pub struct DiagramShader {
    program: ShaderProgram,
    color_mult: ShaderUniform<Color4>,
    tex_rect: ShaderUniform<Rect<f32>>,
    texture: ShaderSampler<Texture2D>,
    mvp: ShaderUniform<Matrix4<f32>>,
    varray: VertexArray,
    _rect_vbuffer: VertexBuffer<u8>,
}

impl DiagramShader {
    pub(super) fn new() -> Result<DiagramShader, String> {
        let vert = Shader::new(
            ShaderType::Vertex,
            "diagram.vert",
            DIAGRAM_VERT_CODE,
        )?;
        let frag = Shader::new(
            ShaderType::Fragment,
            "diagram.frag",
            DIAGRAM_FRAG_CODE,
        )?;
        let program = ShaderProgram::new(&[&vert, &frag])?;
        let color_mult = program.get_uniform("ColorMult")?;
        let tex_rect = program.get_uniform("TexRect")?;
        let texture = program.get_sampler(0, "Texture")?;
        let mvp = program.get_uniform("MVP")?;

        let varray = VertexArray::new(1);
        let rect_vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        varray.bind();
        rect_vbuffer.attribi(0, 2, 0, 0);

        Ok(DiagramShader {
            program,
            color_mult,
            tex_rect,
            texture,
            mvp,
            varray,
            _rect_vbuffer: rect_vbuffer,
        })
    }

    pub fn draw(
        &self,
        matrix: &Matrix4<f32>,
        rect: Rect<f32>,
        tex_rect: Rect<f32>,
        texture: &Texture2D,
    ) {
        self.draw_tinted(matrix, rect, &Color4::WHITE, tex_rect, texture);
    }

    pub fn draw_tinted(
        &self,
        matrix: &Matrix4<f32>,
        rect: Rect<f32>,
        color_mult: &Color4,
        tex_rect: Rect<f32>,
        texture: &Texture2D,
    ) {
        self.program.bind();
        self.color_mult.set(color_mult);
        self.tex_rect.set(&tex_rect);
        self.texture.set(texture);
        let mvp = matrix
            * Matrix4::trans2(rect.x, rect.y)
            * Matrix4::scale2(rect.width, rect.height);
        self.mvp.set(&mvp);
        self.varray.bind();
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//
