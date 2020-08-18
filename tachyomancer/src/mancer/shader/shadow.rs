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
use cgmath::{Matrix4, Vector2};
use tachy::geom::{Color3, Color4, MatrixExt, Rect, RectSize};

//===========================================================================//

const SHADOW_VERT_CODE: &[u8] = include_bytes!("shadow.vert");
const SHADOW_FRAG_CODE: &[u8] = include_bytes!("shadow.frag");

//===========================================================================//

pub struct ShadowShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    rect_size: ShaderUniform<RectSize<f32>>,
    shadow_color: ShaderUniform<Color4>,
    shadow_radius: ShaderUniform<f32>,
    rect_varray: VertexArray,
    _rect_vbuffer: VertexBuffer<u8>,
}

impl ShadowShader {
    pub(crate) fn new() -> Result<ShadowShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "shadow.vert", SHADOW_VERT_CODE)?;
        let frag = Shader::new(
            ShaderType::Fragment,
            "shadow.frag",
            SHADOW_FRAG_CODE,
        )?;
        let program = ShaderProgram::new(&[&vert, &frag])?;

        let mvp = program.get_uniform("MVP")?;
        let rect_size = program.get_uniform("RectSize")?;
        let shadow_color = program.get_uniform("ShadowColor")?;
        let shadow_radius = program.get_uniform("ShadowRadius")?;

        let rect_varray = VertexArray::new(1);
        let rect_vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        rect_varray.bind();
        rect_vbuffer.attribf(0, 2, 0, 0);

        Ok(ShadowShader {
            program,
            mvp,
            rect_size,
            shadow_color,
            shadow_radius,
            rect_varray,
            _rect_vbuffer: rect_vbuffer,
        })
    }

    pub fn rect_shadow_basic(
        &self,
        matrix: &Matrix4<f32>,
        rect: Rect<f32>,
        color: Color3,
    ) {
        self.rect_shadow_depth(matrix, rect, color, 2.0);
    }

    pub fn rect_shadow_depth(
        &self,
        matrix: &Matrix4<f32>,
        rect: Rect<f32>,
        color: Color3,
        depth: f32,
    ) {
        self.rect_shadow(
            matrix,
            rect,
            color.with_alpha(0.6),
            Vector2::new(depth, depth),
            6.0 * depth,
        );
    }

    pub fn rect_shadow(
        &self,
        matrix: &Matrix4<f32>,
        rect: Rect<f32>,
        color: Color4,
        offset: Vector2<f32>,
        radius: f32,
    ) {
        self.program.bind();
        let mvp = matrix
            * Matrix4::trans2(
                rect.x + offset.x - radius,
                rect.y + offset.y - radius,
            )
            * Matrix4::scale2(
                rect.width + 2.0 * radius,
                rect.height + 2.0 * radius,
            );
        self.mvp.set(&mvp);
        self.rect_size.set(&rect.size());
        self.shadow_color.set(&color);
        self.shadow_radius.set(&radius);
        self.rect_varray.bind();
        self.rect_varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//
