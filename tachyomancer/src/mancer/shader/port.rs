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
use tachy::geom::Color4;
use tachy::state::{PortColor, PortFlow};

//===========================================================================//

const PORT_VERT_CODE: &[u8] = include_bytes!("port.vert");
const PORT_FRAG_CODE: &[u8] = include_bytes!("port.frag");

// TODO: use Color3 here and in the uniform
const ANALOG_PORT_COLOR: Color4 = Color4::new(0.5, 0.7, 0.5, 1.0);
const BEHAVIOR_PORT_COLOR: Color4 = Color4::ORANGE4;
const EVENT_PORT_COLOR: Color4 = Color4::new(0.533, 0.667, 0.667, 1.0);

//===========================================================================//

const NUM_PORT_VERTICES: usize = 24;

#[cfg_attr(rustfmt, rustfmt_skip)]
const PORT_VERTICES: &[f32; 2 * NUM_PORT_VERTICES] = &[
    0.2, 0.0, // center
    0.2, 1.0, // first corner

    // edge:
    0.88,  1.0,  0.88,  0.9,  0.88,  0.8,  0.88,  0.7,  0.88,  0.6,
    0.88,  0.5,  0.88,  0.4,  0.88,  0.3,  0.88,  0.2,  0.88,  0.1,
    0.88,  0.0,  0.88, -0.1,  0.88, -0.2,  0.88, -0.3,  0.88, -0.4,
    0.88, -0.5,  0.88, -0.6,  0.88, -0.7,  0.88, -0.8,  0.88, -0.9,
    0.88, -1.0,

    0.2, -1.0, // last corner
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
    texture: ShaderSampler<Texture2D>,
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
        let texture = program.get_sampler(0, "Texture")?;
        let color_tint = program.get_uniform("ColorTint")?;

        let port_varray = VertexArray::new(2);
        let port_vbuffer = VertexBuffer::new(PORT_VERTICES);
        let port_edge_vbuffer = VertexBuffer::new(PORT_VERTEX_IS_EDGE);
        port_varray.bind();
        port_vbuffer.attribf(0, 2, 0, 0);
        port_edge_vbuffer.attribi(1, 1, 0, 0);

        Ok(PortShader {
            program,
            mvp,
            flow_and_color,
            width_scale,
            texture,
            color_tint,
            port_varray,
            _port_vbuffer: port_vbuffer,
            _port_edge_vbuffer: port_edge_vbuffer,
        })
    }

    pub fn bind(&self) {
        self.program.bind();
        self.port_varray.bind();
    }

    pub fn set_mvp(&self, mvp: &Matrix4<f32>) {
        self.mvp.set(mvp);
    }

    pub fn set_port_flow_and_color(&self, flow: PortFlow, color: PortColor) {
        let mut value = 0;
        match flow {
            PortFlow::Source => value |= 0x1,
            PortFlow::Sink => {}
        }
        match color {
            PortColor::Behavior => {
                self.color_tint.set(&BEHAVIOR_PORT_COLOR);
            }
            PortColor::Event => {
                value |= 0x2;
                self.color_tint.set(&EVENT_PORT_COLOR);
            }
            PortColor::Analog => {
                value |= 0x4;
                self.color_tint.set(&ANALOG_PORT_COLOR);
            }
        }
        self.flow_and_color.set(&value);
    }

    pub fn set_texture(&self, texture: &Texture2D) {
        self.texture.set(texture);
    }

    pub fn draw(&self, width_scale: f32) {
        self.width_scale.set(&width_scale);
        self.port_varray.draw(Primitive::TriangleFan, 0, NUM_PORT_VERTICES);
    }
}

//===========================================================================//
