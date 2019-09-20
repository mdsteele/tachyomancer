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
    Primitive, Shader, ShaderProgram, ShaderType, ShaderUniform, VertexArray,
    VertexBuffer,
};
use cgmath::Matrix4;
use tachy::geom::{Color3, Color4, MatrixExt, Rect};

//===========================================================================//

const SOLID_VERT_CODE: &[u8] = include_bytes!("solid.vert");
const SOLID_FRAG_CODE: &[u8] = include_bytes!("solid.frag");

//===========================================================================//

pub struct SolidShader {
    program: ShaderProgram,
    color: ShaderUniform<Color4>,
    mvp: ShaderUniform<Matrix4<f32>>,
    rect_varray: VertexArray,
    _rect_vbuffer: VertexBuffer<u8>,
}

impl SolidShader {
    pub(crate) fn new() -> Result<SolidShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "solid.vert", SOLID_VERT_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "solid.frag", SOLID_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;

        let color = program.get_uniform("SolidColor")?;
        let mvp = program.get_uniform("MVP")?;
        let rect_varray = VertexArray::new(1);
        let rect_vbuffer =
            VertexBuffer::new(&[0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0]);
        rect_varray.bind();
        rect_vbuffer.attribf(0, 3, 0, 0);

        Ok(SolidShader {
            program,
            color,
            mvp,
            rect_varray,
            _rect_vbuffer: rect_vbuffer,
        })
    }

    pub fn fill_rect(
        &self,
        matrix: &Matrix4<f32>,
        color: Color3,
        rect: Rect<f32>,
    ) {
        self.tint_rect(matrix, color.with_alpha(1.0), rect);
    }

    pub fn tint_rect(
        &self,
        matrix: &Matrix4<f32>,
        color: Color4,
        rect: Rect<f32>,
    ) {
        self.program.bind();
        self.color.set(&color);
        let mvp = matrix
            * Matrix4::trans2(rect.x, rect.y)
            * Matrix4::scale2(rect.width, rect.height);
        self.mvp.set(&mvp);
        self.rect_varray.bind();
        self.rect_varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//
