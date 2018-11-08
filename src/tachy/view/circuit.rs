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
use cgmath::{self, Deg, Matrix4, Point2, vec3};
use gl;
use num_integer::mod_floor;
use tachy::font::Align;
use tachy::gl::{Primitive, VertexArray, VertexBuffer};
use tachy::gui::{Event, Keycode, Rect, Resources};
use tachy::state::{ChipType, Coords, Direction, EditGrid, GridChange,
                   Orientation, WireShape};

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

    pub fn handle_event(&mut self, event: &Event, grid: &mut EditGrid)
                        -> bool {
        self.edit_grid.handle_event(event, grid);
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
const ZONE_CENTER_SEMI_SIZE: i32 = 12;

struct EditGridView {
    width: u32,
    height: u32,
    wire_model: WireModel,
    drag: Option<WireDrag>,
}

impl EditGridView {
    pub fn new(width: u32, height: u32) -> EditGridView {
        EditGridView {
            width: width,
            height: height,
            wire_model: WireModel::new(),
            drag: None,
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
                 ctype: ChipType, orient: Orientation) {
        let size = orient * ctype.size();
        let (width, height) = (size.width as f32, size.height as f32);
        let rect = (-0.9, -0.9, 2.0 * width - 0.2, 2.0 * height - 0.2);
        resources.shaders().solid().fill_rect(matrix, (1.0, 0.0, 0.5), rect);
        resources.fonts().roman().draw(matrix,
                                       (0.5, 1.0),
                                       Align::Center,
                                       (width - 1.0, height - 1.5),
                                       &format!("{:?}", ctype));
        // TODO: draw ports
    }

    fn handle_event(&mut self, event: &Event, grid: &mut EditGrid) {
        match event {
            Event::MouseDown(mouse) => {
                if mouse.left {
                    let mut drag = WireDrag::new();
                    if drag.move_to(self.zone_for_point(mouse.pt), grid) {
                        self.drag = Some(drag);
                    } else {
                        debug_log!("drag done (down)");
                    }
                } else if mouse.right {
                    let coords = self.coords_for_point(mouse.pt);
                    let east = grid.wire_shape_at(coords, Direction::East);
                    if east == Some(WireShape::Cross) ||
                        (east == Some(WireShape::Straight) &&
                             grid.wire_shape_at(coords, Direction::South) ==
                                 Some(WireShape::Straight))
                    {
                        grid.mutate(&[GridChange::ToggleCrossWire(coords)]);
                    }
                }
            }
            Event::MouseMove(mouse) => {
                if let Some(mut drag) = self.drag.take() {
                    if drag.move_to(self.zone_for_point(mouse.pt), grid) {
                        self.drag = Some(drag);
                    } else {
                        debug_log!("drag done (move)");
                    }
                }
            }
            Event::MouseUp(mouse) => {
                if mouse.left {
                    if let Some(mut drag) = self.drag.take() {
                        drag.finish(grid);
                    }
                }
            }
            _ => {}
        }
    }

    fn coords_for_point(&self, pt: Point2<i32>) -> Coords {
        // TODO: translate based on current scrolling
        pt / GRID_CELL_SIZE
    }

    fn zone_for_point(&self, pt: Point2<i32>) -> Zone {
        // TODO: translate based on current scrolling
        let coords = pt / GRID_CELL_SIZE;
        let x = mod_floor(pt.x, GRID_CELL_SIZE) - GRID_CELL_SIZE / 2;
        let y = mod_floor(pt.y, GRID_CELL_SIZE) - GRID_CELL_SIZE / 2;
        if x.abs() <= ZONE_CENTER_SEMI_SIZE &&
            y.abs() <= ZONE_CENTER_SEMI_SIZE
        {
            Zone::Center(coords)
        } else if x.abs() > y.abs() {
            Zone::East(if x > 0 {
                           coords
                       } else {
                           coords + Direction::West
                       })
        } else {
            Zone::South(if y > 0 {
                            coords
                        } else {
                            coords + Direction::North
                        })
        }
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

struct WireDrag {
    curr: Option<Zone>,
    prev: Option<Zone>,
    changed: bool,
}

impl WireDrag {
    pub fn new() -> WireDrag {
        WireDrag {
            curr: None,
            prev: None,
            changed: false,
        }
    }

    pub fn move_to(&mut self, zone: Zone, grid: &mut EditGrid) -> bool {
        if self.curr == Some(zone) {
            return true;
        }
        let more = match (self.prev, self.curr, zone) {
            (_, None, Zone::East(coords)) => {
                self.try_start_stub(coords, Direction::East, grid)
            }
            (_, None, Zone::South(coords)) => {
                self.try_start_stub(coords, Direction::South, grid)
            }
            (None, Some(Zone::Center(coords1)), Zone::East(coords2)) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::East, grid)
                } else if coords1 + Direction::West == coords2 {
                    self.try_split(coords1, Direction::West, grid)
                } else {
                    true
                }
            }
            (None, Some(Zone::Center(coords1)), Zone::South(coords2)) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::South, grid)
                } else if coords1 + Direction::North == coords2 {
                    self.try_split(coords1, Direction::North, grid)
                } else {
                    true
                }
            }
            (Some(Zone::East(coords1)), _, Zone::East(coords2)) => {
                if coords1 + Direction::East == coords2 {
                    self.try_straight(coords2, Direction::East, grid)
                } else if coords1 + Direction::West == coords2 {
                    self.try_straight(coords1, Direction::West, grid)
                } else {
                    true
                }
            }
            (Some(Zone::South(coords1)), _, Zone::South(coords2)) => {
                if coords1 + Direction::South == coords2 {
                    self.try_straight(coords2, Direction::South, grid)
                } else if coords1 + Direction::North == coords2 {
                    self.try_straight(coords1, Direction::North, grid)
                } else {
                    true
                }
            }
            // TODO: other cases
            (_, _, _) => true,
        };
        self.prev = self.curr;
        self.curr = Some(zone);
        more
    }

    pub fn finish(&mut self, grid: &mut EditGrid) {
        if self.changed {
            return;
        }
        match self.curr {
            Some(Zone::Center(coords)) => {
                self.try_toggle_cross(coords, grid);
            }
            Some(Zone::East(coords)) => {
                self.try_remove_stub(coords, Direction::East, grid);
            }
            Some(Zone::South(coords)) => {
                self.try_remove_stub(coords, Direction::South, grid);
            }
            None => {}
        }
    }

    fn try_start_stub(&mut self, coords: Coords, dir: Direction,
                      grid: &mut EditGrid)
                      -> bool {
        match (grid.wire_shape_at(coords, dir),
                 grid.wire_shape_at(coords + dir, -dir)) {
            (None, _) => {
                grid.mutate(&[GridChange::ToggleStubWire(coords, dir)]);
                self.changed = true;
                true
            }
            (_, Some(WireShape::Stub)) |
            (Some(WireShape::Stub), _) => true,
            (_, _) => true,
        }
    }

    fn try_remove_stub(&mut self, coords: Coords, dir: Direction,
                       grid: &mut EditGrid) {
        match (grid.wire_shape_at(coords, dir),
                 grid.wire_shape_at(coords + dir, -dir)) {
            (Some(WireShape::Stub), Some(WireShape::Stub)) => {
                grid.mutate(&[GridChange::ToggleStubWire(coords, dir)]);
                self.changed = true;
            }
            (_, _) => {}
        }
    }

    fn try_toggle_cross(&mut self, coords: Coords, grid: &mut EditGrid) {
        match (grid.wire_shape_at(coords, Direction::East),
                 grid.wire_shape_at(coords, Direction::South)) {
            (Some(WireShape::Straight), Some(WireShape::Straight)) |
            (Some(WireShape::Cross), _) => {
                grid.mutate(&[GridChange::ToggleCrossWire(coords)]);
                self.changed = true;
            }
            (_, _) => {}
        }
    }

    fn try_straight(&mut self, coords: Coords, dir: Direction,
                    grid: &mut EditGrid)
                    -> bool {
        match (grid.wire_shape_at(coords, dir),
                 grid.wire_shape_at(coords, -dir)) {
            (None, Some(WireShape::Stub)) => {
                grid.mutate(
                    &[
                        GridChange::ToggleStubWire(coords, dir),
                        GridChange::ToggleCenterWire(coords, dir, -dir),
                    ],
                );
                self.changed = true;
                true
            }
            (Some(WireShape::Stub), Some(WireShape::Stub)) => {
                grid.mutate(
                    &[GridChange::ToggleCenterWire(coords, dir, -dir)],
                );
                self.changed = true;
                true
            }
            (Some(WireShape::Straight), Some(WireShape::Straight)) => {
                if grid.wire_shape_at(coords - dir, dir) ==
                    Some(WireShape::Stub)
                {
                    grid.mutate(
                        &[
                            GridChange::ToggleCenterWire(coords, dir, -dir),
                            GridChange::ToggleStubWire(coords, -dir),
                        ],
                    );
                } else {
                    grid.mutate(
                        &[GridChange::ToggleCenterWire(coords, dir, -dir)],
                    );
                }
                self.changed = true;
                true
            }
            (_, _) => false,
        }
    }

    fn try_split(&mut self, coords: Coords, dir: Direction,
                 grid: &mut EditGrid)
                 -> bool {
        match (grid.wire_shape_at(coords, dir),
                 grid.wire_shape_at(coords, -dir),
                 grid.wire_shape_at(coords, dir.rotate_cw())) {
            (Some(WireShape::SplitLeft), _, _) |
            (Some(WireShape::SplitRight), _, _) |
            (Some(WireShape::SplitTee), _, _) |
            (Some(WireShape::Cross), _, _) |
            (Some(WireShape::Stub), Some(WireShape::TurnLeft), _) |
            (Some(WireShape::Stub), Some(WireShape::TurnRight), _) |
            (Some(WireShape::Stub), Some(WireShape::SplitTee), _) |
            (Some(WireShape::Stub), _, Some(WireShape::Straight)) => {
                grid.mutate(&[GridChange::ToggleSplitWire(coords, dir)]);
                self.changed = true;
                true
            }
            (None, Some(WireShape::TurnLeft), _) |
            (None, Some(WireShape::TurnRight), _) |
            (None, Some(WireShape::SplitTee), _) |
            (None, _, Some(WireShape::Straight)) => {
                grid.mutate(
                    &[
                        GridChange::ToggleStubWire(coords, dir),
                        GridChange::ToggleSplitWire(coords, dir),
                    ],
                );
                self.changed = true;
                true
            }
            (_, _, _) => false,
        }
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Zone {
    Center(Coords),
    East(Coords),
    South(Coords),
}

//===========================================================================//
