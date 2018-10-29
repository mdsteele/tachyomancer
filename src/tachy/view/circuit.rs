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

use cgmath::{self, Matrix4};
use gl;
use tachy::font::Align;
use tachy::gl::{VertexArray, VertexBuffer};
use tachy::gui::{Event, Keycode, Rect, Resources};
use tachy::state::GameState;

//===========================================================================//

const TEX_START: f32 = 4.0 / 128.0;
const TEX_END: f32 = 10.0 / 128.0;

#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD_VERTEX_DATA: &[f32] = &[
    0.0, 0.0, 0.0,  TEX_START,
    1.0, 0.0, 0.0,  TEX_START,
    0.0, 1.0, 0.0,  TEX_END,

    1.0, 0.0, 0.0,  TEX_START,
    0.0, 1.0, 0.0,  TEX_END,
    1.0, 1.0, 0.0,  TEX_END,
];

//===========================================================================//

pub struct CircuitView {
    width: u32,
    height: u32,
    varray: VertexArray,
    vbuffer: VertexBuffer<f32>,
}

impl CircuitView {
    pub fn new(size: (u32, u32)) -> CircuitView {
        CircuitView {
            width: size.0,
            height: size.1,
            varray: VertexArray::new(2),
            vbuffer: VertexBuffer::new(QUAD_VERTEX_DATA),
        }
    }

    pub fn draw(&self, resources: &Resources, _state: &GameState) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.4, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let projection = cgmath::ortho(0.0,
                                       self.width as f32,
                                       self.height as f32,
                                       0.0,
                                       -1.0,
                                       1.0);
        resources.fonts().roman().draw(&projection,
                                       (16.0, 30.0),
                                       Align::Left,
                                       (50.0, 50.0),
                                       "Hello, world!");
        let model_mtx =
            Matrix4::from_translation(cgmath::vec3(200.0, 150.0, 0.0)) *
                Matrix4::from_nonuniform_scale(100.0, 50.0, 1.0);
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(&(projection * model_mtx));
        shader.set_wire_color((0.0, 1.0, 1.0));
        resources.textures().wires().bind();
        self.varray.bind();
        self.vbuffer.attribf(0, 3, 4, 0);
        self.vbuffer.attribf(1, 1, 4, 3);
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

    pub fn handle_event(&mut self, event: &Event, _state: &mut GameState)
                        -> bool {
        match event {
            Event::MouseDown(mouse) => {
                if mouse.left &&
                    Rect::new(200, 150, 100, 50).contains_point(mouse.pt)
                {
                    return true;
                }
            }
            Event::KeyDown(key) => {
                if key.command && key.shift && key.code == Keycode::F {
                    return true;
                }
            }
            _ => {}
        }
        return false;
    }
}

//===========================================================================//
