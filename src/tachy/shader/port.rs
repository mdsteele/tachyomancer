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
use tachy::geom::Color4;
use tachy::gl::{Primitive, Shader, ShaderProgram, ShaderType, ShaderUniform,
                VertexArray, VertexBuffer};

//===========================================================================//

const PORT_VERT_CODE: &[u8] = include_bytes!("port.vert");
const PORT_FRAG_CODE: &[u8] = include_bytes!("port.frag");

const BEHAVIOR_WIRE_COLOR: Color4 = Color4::ORANGE4;
const EVENT_WIRE_COLOR: Color4 = Color4::new(0.533, 0.667, 0.667, 1.0);

//===========================================================================//

const NUM_PORT_VERTICES: usize = 24;

#[cfg_attr(rustfmt, rustfmt_skip)]
const PORT_VERTICES: &[f32; 2 * NUM_PORT_VERTICES] = &[
    0.0, 0.0, // center
    0.0, -1.0, // first corner

    // edge:
    0.88, -1.0,  0.88, -0.9,  0.88, -0.8,  0.88, -0.7,  0.88, -0.6,
    0.88, -0.5,  0.88, -0.4,  0.88, -0.3,  0.88, -0.2,  0.88, -0.1,
    0.88, -0.0,  0.88,  0.1,  0.88,  0.2,  0.88,  0.3,  0.88,  0.4,
    0.88,  0.5,  0.88,  0.6,  0.88,  0.7,  0.88,  0.8,  0.88,  0.9,
    0.88,  1.0,

    0.0, 1.0, // last corner
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const PORT_VERTEX_IS_EDGE: &[u8; NUM_PORT_VERTICES] = &[
    0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
];

//===========================================================================//

pub struct PortShader {
    program: ShaderProgram,
    mvp: ShaderUniform<Matrix4<f32>>,
    flow_and_color: ShaderUniform<u32>,
    width_scale: ShaderUniform<f32>,
    color_tint: ShaderUniform<Color4>,
    port_varray: VertexArray,
    _port_vbuffer: VertexBuffer<f32>,
    _port_edge_vbuffer: VertexBuffer<u8>,
}

impl PortShader {
    pub(super) fn new() -> Result<PortShader, String> {
        let vert =
            Shader::new(ShaderType::Vertex, "port.vert", PORT_VERT_CODE)?;
        let frag =
            Shader::new(ShaderType::Fragment, "port.frag", PORT_FRAG_CODE)?;
        let program = ShaderProgram::new(&[&vert, &frag])?;

        let mvp = program.get_uniform("MVP")?;
        let flow_and_color = program.get_uniform("FlowAndColor")?;
        let width_scale = program.get_uniform("WidthScale")?;
        let color_tint = program.get_uniform("ColorTint")?;

        let port_varray = VertexArray::new(2);
        let port_vbuffer = VertexBuffer::new(PORT_VERTICES);
        let port_edge_vbuffer = VertexBuffer::new(PORT_VERTEX_IS_EDGE);
        port_varray.bind();
        port_vbuffer.attribf(0, 2, 0, 0);
        port_edge_vbuffer.attribi(1, 1, 0, 0);

        let shader = PortShader {
            program,
            mvp,
            flow_and_color,
            width_scale,
            color_tint,
            port_varray,
            _port_vbuffer: port_vbuffer,
            _port_edge_vbuffer: port_edge_vbuffer,
        };
        Ok(shader)
    }

    pub fn bind(&self) {
        self.program.bind();
        self.port_varray.bind();
    }

    pub fn set_mvp(&self, mvp: &Matrix4<f32>) { self.mvp.set(mvp); }

    pub fn set_port_flow_and_color(&self, flow: bool, color: bool) {
        let mut value = 0;
        if flow {
            value |= 0x2;
        }
        if color {
            value |= 0x1;
            self.color_tint.set(&EVENT_WIRE_COLOR);
        } else {
            self.color_tint.set(&BEHAVIOR_WIRE_COLOR);
        }
        self.flow_and_color.set(&value);
    }

    pub fn draw(&self, width_scale: f32) {
        self.width_scale.set(&width_scale);
        self.port_varray.draw(Primitive::TriangleFan, 0, NUM_PORT_VERTICES);
    }
}

//===========================================================================//
