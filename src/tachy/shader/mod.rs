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
use tachy::gl::{Shader, ShaderProgram, ShaderType, ShaderUniform};

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

    #[allow(dead_code)]
    pub fn solid(&self) -> &SolidShader { &self.solid }

    pub fn wire(&self) -> &WireShader { &self.wire }
}

//===========================================================================//

pub struct SolidShader {
    program: ShaderProgram,
    color: ShaderUniform<(f32, f32, f32)>,
    mvp: ShaderUniform<Matrix4<f32>>,
}

impl SolidShader {
    fn new(program: ShaderProgram) -> Result<SolidShader, String> {
        let color = program.get_uniform("SolidColor")?;
        let mvp = program.get_uniform("MVP")?;
        Ok(SolidShader {
               program,
               color,
               mvp,
           })
    }

    #[allow(dead_code)]
    pub fn bind(&self) { self.program.bind() }

    #[allow(dead_code)]
    pub fn set_color(&self, color: (f32, f32, f32)) { self.color.set(&color); }

    #[allow(dead_code)]
    pub fn set_mvp(&self, mvp: &Matrix4<f32>) { self.mvp.set(mvp); }
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

    pub fn bind(&self) { self.program.bind() }

    pub fn set_wire_color(&self, color: (f32, f32, f32)) {
        self.wire_color.set(&color);
    }

    pub fn set_mvp(&self, mvp: &Matrix4<f32>) { self.mvp.set(mvp); }
}

//===========================================================================//
