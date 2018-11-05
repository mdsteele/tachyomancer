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

use super::wire::WireModel;
use cgmath::{self, Deg, Matrix4, vec3};
use gl;
use tachy::font::Align;
use tachy::gl::{Primitive, VertexArray, VertexBuffer};
use tachy::gui::{Event, Keycode, Rect, Resources};
use tachy::state::{ChipType, Coords, Direction, EditGrid, Orientation,
                   WireShape};

//===========================================================================//

const TEX_START: f32 = 4.0 / 128.0;
const TEX_END: f32 = 10.0 / 128.0;

#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD_VERTEX_DATA: &[f32] = &[
    0.0, 0.0,  TEX_START,
    1.0, 0.0,  TEX_START,
    0.0, 1.0,  TEX_END,
    1.0, 1.0,  TEX_END,
];

//===========================================================================//

pub struct CircuitView {
    width: u32,
    height: u32,
    varray: VertexArray,
    vbuffer: VertexBuffer<f32>,
    edit_grid: EditGridView,
}

impl CircuitView {
    pub fn new(size: (u32, u32)) -> CircuitView {
        CircuitView {
            width: size.0,
            height: size.1,
            varray: VertexArray::new(2),
            vbuffer: VertexBuffer::new(QUAD_VERTEX_DATA),
            edit_grid: EditGridView::new(size.0, size.1),
        }
    }

    pub fn draw(&self, resources: &Resources, grid: &EditGrid) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.4, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.edit_grid.draw(resources, grid);
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
            Matrix4::from_translation(cgmath::vec3(200.0, 350.0, 0.0)) *
                Matrix4::from_nonuniform_scale(100.0, 50.0, 1.0);
        let shader = resources.shaders().wire();
        shader.bind();
        shader.set_mvp(&(projection * model_mtx));
        shader.set_wire_color((0.0, 1.0, 1.0));
        resources.textures().wire().bind();
        self.varray.bind();
        self.vbuffer.attribf(0, 3, 3, 0);
        self.vbuffer.attribf(1, 1, 3, 2);
        self.varray.draw(Primitive::TriangleStrip, 0, 4);
    }

    pub fn handle_event(&mut self, event: &Event, _grid: &EditGrid) -> bool {
        match event {
            Event::MouseDown(mouse) => {
                if mouse.left &&
                    Rect::new(200, 350, 100, 50).contains_point(mouse.pt)
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

const GRID_CELL_SIZE: i32 = 64;

struct EditGridView {
    width: u32,
    height: u32,
    wire_model: WireModel,
}

impl EditGridView {
    pub fn new(width: u32, height: u32) -> EditGridView {
        EditGridView {
            width: width,
            height: height,
            wire_model: WireModel::new(),
        }
    }

    pub fn draw(&self, resources: &Resources, grid: &EditGrid) {
        let matrix = cgmath::ortho(0.0,
                                   self.width as f32,
                                   self.height as f32,
                                   0.0,
                                   -1.0,
                                   1.0);
        // TODO: translate based on current scrolling
        for (coords, dir, shape, size, color) in grid.wire_fragments() {
            match (shape, dir) {
                (WireShape::Stub, _) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model.draw_stub(resources, &matrix, color, size);
                }
                (WireShape::Straight, Direction::East) |
                (WireShape::Straight, Direction::North) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model
                        .draw_straight(resources, &matrix, color, size);
                }
                (WireShape::TurnLeft, _) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model
                        .draw_corner(resources, &matrix, color, size);
                }
                (WireShape::SplitTee, _) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model.draw_tee(resources, &matrix, color, size);
                }
                (WireShape::Cross, Direction::East) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model
                        .draw_cross(resources, &matrix, color, size);
                }
                _ => {}
            }
        }
        for (coords, ctype, orient) in grid.chips() {
            self.draw_chip(resources,
                           &coords_matrix(&matrix, coords, Direction::East),
                           ctype,
                           orient);
        }
    }

    fn draw_chip(&self, resources: &Resources, matrix: &Matrix4<f32>,
                 ctype: ChipType, _orient: Orientation) {
        resources
            .shaders()
            .solid()
            .fill_rect(matrix, (1.0, 0.0, 0.5), (-0.9, -0.9, 1.8, 1.8));
        resources.fonts().roman().draw(matrix,
                                       (0.5, 1.0),
                                       Align::Center,
                                       (0.0, -0.5),
                                       &format!("{:?}", ctype));
    }
}

fn coords_matrix(matrix: &Matrix4<f32>, coords: Coords, dir: Direction)
                 -> Matrix4<f32> {
    let angle = match dir {
        Direction::East => Deg(0.0),
        Direction::South => Deg(90.0),
        Direction::West => Deg(180.0),
        Direction::North => Deg(-90.0),
    };
    let cx = (coords.x * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32;
    let cy = (coords.y * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32;
    matrix * Matrix4::from_translation(vec3(cx, cy, 0.0)) *
        Matrix4::from_axis_angle(vec3(0.0, 0.0, 1.0), angle) *
        Matrix4::from_scale((GRID_CELL_SIZE / 2) as f32)
}

//===========================================================================//
