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

use crate::tachy::geom::{AsFloat, MatrixExt, RectSize};
use crate::tachy::gl::{
    FrameBuffer, Primitive, Shader, ShaderProgram, ShaderSampler, ShaderType,
    ShaderUniform, Texture2DMultisample, VertexArray, VertexBuffer,
};
use cgmath::{Matrix4, Point2};

//===========================================================================//

const FRAME_VERT_CODE: &[u8] = include_bytes!("frame.vert");
const FRAME_FRAG_CODE: &[u8] = include_bytes!("frame.frag");

//===========================================================================//

pub struct FrameBufferShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    frame_size: ShaderUniform<RectSize<f32>>,
    tex_size: ShaderUniform<RectSize<f32>>,
    grayscale: ShaderUniform<u32>,
    texture: ShaderSampler<Texture2DMultisample>,
    varray: VertexArray,
    _vbuffer: VertexBuffer<u8>,
}

impl FrameBufferShader {
    pub(super) fn new() -> Result<FrameBufferShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "frame.vert", FRAME_VERT_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "frame.frag", FRAME_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;

        let mvp = program.get_uniform("MVP")?;
        let frame_size = program.get_uniform("FrameSize")?;
        let tex_size = program.get_uniform("TexSize")?;
        let grayscale = program.get_uniform("Grayscale")?;
        let texture = program.get_sampler(0, "Texture")?;

        let varray = VertexArray::new(1);
        let vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        varray.bind();
        vbuffer.attribi(0, 2, 0, 0);

        Ok(FrameBufferShader {
            program,
            mvp,
            frame_size,
            tex_size,
            grayscale,
            texture,
            varray,
            _vbuffer: vbuffer,
        })
    }

    pub fn draw(
        &self,
        matrix: &Matrix4<f32>,
        fbuffer: &FrameBuffer,
        left_top: Point2<f32>,
        grayscale: bool,
    ) {
        self.program.bind();
        self.texture.set(fbuffer.texture());
        self.mvp.set(&(matrix * Matrix4::trans2(left_top.x, left_top.y)));
        let size = fbuffer.size().as_f32();
        self.frame_size.set(&size);
        let texture_size = fbuffer.texture_size().as_f32();
        self.tex_size.set(&RectSize::new(
            size.width / texture_size.width,
            size.height / texture_size.height,
        ));
        self.grayscale.set(&(if grayscale { 1 } else { 0 }));
        self.varray.bind();
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//
