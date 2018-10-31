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

use cgmath::{Matrix4, vec3};
use tachy::gl::{Primitive, Shader, ShaderProgram, ShaderType, ShaderUniform,
                VertexArray, VertexBuffer};

//===========================================================================//

const SOLID_VERT_CODE: &[u8] = include_bytes!("solid.vert");
const SOLID_FRAG_CODE: &[u8] = include_bytes!("solid.frag");

const WIRE_VERT_CODE: &[u8] = include_bytes!("wire.vert");
const WIRE_FRAG_CODE: &[u8] = include_bytes!("wire.frag");

//===========================================================================//

pub struct Shaders {
    solid: SolidShader,
    wire: WireShader,
}

impl Shaders {
    pub fn new() -> Result<Shaders, String> {
        let solid_vert =
            Shader::new(ShaderType::Vertex, "solid.vert", SOLID_VERT_CODE)?;
        let solid_frag =
            Shader::new(ShaderType::Fragment, "solid.frag", SOLID_FRAG_CODE)?;
        let solid_prog = ShaderProgram::new(&[&solid_vert, &solid_frag])?;
        let solid = SolidShader::new(solid_prog)?;

        let wire_vert =
            Shader::new(ShaderType::Vertex, "wire.vert", WIRE_VERT_CODE)?;
        let wire_frag =
            Shader::new(ShaderType::Fragment, "wire.frag", WIRE_FRAG_CODE)?;
        let wire_prog = ShaderProgram::new(&[&wire_vert, &wire_frag])?;
        let wire = WireShader::new(wire_prog)?;

        Ok(Shaders { solid, wire })
    }

    pub fn solid(&self) -> &SolidShader { &self.solid }

    pub fn wire(&self) -> &WireShader { &self.wire }
}

//===========================================================================//

pub struct SolidShader {
    program: ShaderProgram,
    color: ShaderUniform<(f32, f32, f32)>,
    mvp: ShaderUniform<Matrix4<f32>>,
    varray: VertexArray,
    rect_vbuffer: VertexBuffer<u8>,
}

impl SolidShader {
    fn new(program: ShaderProgram) -> Result<SolidShader, String> {
        let color = program.get_uniform("SolidColor")?;
        let mvp = program.get_uniform("MVP")?;
        let varray = VertexArray::new(1);
        let rect_vbuffer =
            VertexBuffer::new(&[0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0]);
        Ok(SolidShader {
               program,
               color,
               mvp,
               varray,
               rect_vbuffer,
           })
    }

    pub fn fill_rect(&self, matrix: &Matrix4<f32>, color: (f32, f32, f32),
                     (left, top, width, height): (f32, f32, f32, f32)) {
        self.program.bind();
        self.color.set(&color);
        let mvp = matrix * Matrix4::from_translation(vec3(left, top, 0.0)) *
            Matrix4::from_nonuniform_scale(width, height, 1.0);
        self.mvp.set(&mvp);
        self.varray.bind();
        self.rect_vbuffer.attribf(0, 3, 0, 0);
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }
}

//===========================================================================//

pub struct WireShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    wire_color: ShaderUniform<(f32, f32, f32)>,
}

impl WireShader {
    fn new(program: ShaderProgram) -> Result<WireShader, String> {
        let mvp = program.get_uniform("MVP")?;
        let wire_color = program.get_uniform("WireColor")?;
        Ok(WireShader {
               program,
               mvp,
               wire_color,
           })
    }

    pub fn bind(&self) { self.program.bind(); }

    pub fn set_wire_color(&self, color: (f32, f32, f32)) {
        self.wire_color.set(&color);
    }

    pub fn set_mvp(&self, mvp: &Matrix4<f32>) { self.mvp.set(mvp); }
}

//===========================================================================//
