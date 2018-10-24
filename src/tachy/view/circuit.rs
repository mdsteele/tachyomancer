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

use cgmath;
use gl;
use tachy::gui::{Event, Keycode, Resources};
use tachy::state::GameState;

//===========================================================================//

pub struct CircuitView {
    width: u32,
    height: u32,
}

impl CircuitView {
    pub fn new(size: (u32, u32)) -> CircuitView {
        CircuitView {
            width: size.0,
            height: size.1,
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
        let shader = resources.shaders().solid();
        shader.bind();
        shader.set_color((1.0, 0.0, 0.0));
        shader.set_mvp(&projection);
        // TODO: draw something
    }

    pub fn handle_event(&mut self, event: &Event, _state: &mut GameState)
                        -> bool {
        match event {
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
