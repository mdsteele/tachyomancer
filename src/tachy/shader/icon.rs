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

use crate::tachy::geom::{Color4, MatrixExt, Rect};
use crate::tachy::gl::{
    Primitive, Shader, ShaderProgram, ShaderSampler, ShaderType,
    ShaderUniform, Texture2D, VertexArray, VertexBuffer,
};
use cgmath::Matrix4;

//===========================================================================//

const ICON_VERT_CODE: &[u8] = include_bytes!("icon.vert");
const ICON_FRAG_CODE: &[u8] = include_bytes!("icon.frag");

//===========================================================================//

pub struct IconShader {
    program: ShaderProgram,
    color: ShaderUniform<Color4>,
    icon_index: ShaderUniform<u32>,
    icon_texture: ShaderSampler<Texture2D>,
    mvp: ShaderUniform<Matrix4<f32>>,
    varray: VertexArray,
    rect_vbuffer: VertexBuffer<u8>,
}

impl IconShader {
    pub(super) fn new() -> Result<IconShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "icon.vert", ICON_VERT_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "icon.frag", ICON_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;
        let color = program.get_uniform("IconColor")?;
        let icon_index = program.get_uniform("IconIndex")?;
        let icon_texture = program.get_sampler(0, "IconTexture")?;
        let mvp = program.get_uniform("MVP")?;
        let varray = VertexArray::new(1);
        let rect_vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        Ok(IconShader {
            program,
            color,
            icon_index,
            icon_texture,
            mvp,
            varray,
            rect_vbuffer,
        })
    }

    pub fn draw(
        &self,
        matrix: &Matrix4<f32>,
        rect: Rect<f32>,
        index: u32,
        color: &Color4,
        texture: &Texture2D,
    ) {
        self.program.bind();
        self.color.set(color);
        self.icon_index.set(&index);
        self.icon_texture.set(texture);
        let mvp = matrix
            * Matrix4::trans2(rect.x, rect.y)
            * Matrix4::scale2(rect.width, rect.height);
        self.mvp.set(&mvp);
        self.varray.bind();
        self.rect_vbuffer.attribi(0, 2, 0, 0);
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//
