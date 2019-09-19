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

use crate::tachy::geom::Rect;
use crate::tachy::gl::{
    Primitive, Shader, ShaderProgram, ShaderType, ShaderUniform, VertexArray,
    VertexBuffer,
};
use cgmath::Matrix4;

//===========================================================================//

const BOARD_VERT_CODE: &[u8] = include_bytes!("board.vert");
const BOARD_FRAG_CODE: &[u8] = include_bytes!("board.frag");

//===========================================================================//

pub struct BoardShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    coords_rect: ShaderUniform<Rect<f32>>,
    varray: VertexArray,
    _vbuffer: VertexBuffer<u8>,
}

impl BoardShader {
    pub(crate) fn new() -> Result<BoardShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "board.vert", BOARD_VERT_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "board.frag", BOARD_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;

        let mvp = program.get_uniform("MVP")?;
        let coords_rect = program.get_uniform("CoordsRect")?;
        let varray = VertexArray::new(1);
        let vbuffer = VertexBuffer::new(&[0, 0, 1, 0, 0, 1, 1, 1]);
        varray.bind();
        vbuffer.attribf(0, 2, 0, 0);
        Ok(BoardShader {
            program,
            mvp,
            coords_rect,
            varray,
            _vbuffer: vbuffer,
        })
    }

    pub fn draw(&self, matrix: &Matrix4<f32>, coords_rect: Rect<f32>) {
        self.program.bind();
        self.mvp.set(matrix);
        self.coords_rect.set(&coords_rect);
        self.varray.bind();
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//
