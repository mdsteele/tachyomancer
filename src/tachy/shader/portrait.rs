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

use cgmath::{Matrix4, Point2};
use tachy::geom::MatrixExt;
use tachy::gl::{Primitive, Shader, ShaderProgram, ShaderSampler, ShaderType,
                ShaderUniform, Texture2D, VertexArray, VertexBuffer};

//===========================================================================//

const PORTRAIT_VERT_CODE: &[u8] = include_bytes!("portrait.vert");
const PORTRAIT_FRAG_CODE: &[u8] = include_bytes!("portrait.frag");

//===========================================================================//

pub struct PortraitShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    portrait_index: ShaderUniform<u32>,
    texture: ShaderSampler<Texture2D>,
    varray: VertexArray,
    _vbuffer: VertexBuffer<u8>,
}

impl PortraitShader {
    pub(super) fn new() -> Result<PortraitShader, String> {
        let vert = Shader::new(ShaderType::Vertex,
                               "portrait.vert",
                               PORTRAIT_VERT_CODE)?;
        let frag = Shader::new(ShaderType::Fragment,
                               "portrait.frag",
                               PORTRAIT_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;

        let mvp = program.get_uniform("MVP")?;
        let portrait_index = program.get_uniform("PortraitIndex")?;
        let texture = program.get_sampler(0, "Texture")?;

        let varray = VertexArray::new(1);
        let vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        varray.bind();
        vbuffer.attribi(0, 2, 0, 0);

        let shader = PortraitShader {
            program,
            mvp,
            portrait_index,
            texture,
            varray,
            _vbuffer: vbuffer,
        };
        Ok(shader)
    }

    pub fn draw(&self, matrix: &Matrix4<f32>, portrait_index: u32,
                left_top: Point2<f32>, texture: &Texture2D) {
        self.program.bind();
        self.mvp.set(&(matrix * Matrix4::trans2(left_top.x, left_top.y)));
        self.portrait_index.set(&portrait_index);
        self.texture.set(texture);
        self.varray.bind();
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//
