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

use super::control::{ControlsAction, ControlsTray};
use super::parts::{PartsAction, PartsTray};
use super::wire::WireModel;
use cgmath::{self, Matrix4, Point2, Vector2, vec3, vec4};
use num_integer::{div_floor, mod_floor};
use tachy::font::Align;
use tachy::gui::{AudioQueue, Event, Keycode, Resources, Sound};
use tachy::state::{ChipType, Coords, Direction, EditGrid, GridChange,
                   Orientation, PortColor, PortFlow, RectSize, WireShape};

//===========================================================================//

const SCROLL_PER_KEYDOWN: i32 = 40;
const SECONDS_PER_TIME_STEP: f64 = 1.0;

//===========================================================================//

pub struct CircuitView {
    width: f32,
    height: f32,
    edit_grid: EditGridView,
    controls_tray: ControlsTray,
    parts_tray: PartsTray,
    seconds_since_time_step: f64,
}

impl CircuitView {
    pub fn new(window_size: RectSize<u32>) -> CircuitView {
        CircuitView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            edit_grid: EditGridView::new(window_size),
            controls_tray: ControlsTray::new(window_size, true),
            parts_tray: PartsTray::new(window_size),
            seconds_since_time_step: 0.0,
        }
    }

    pub fn draw(&self, resources: &Resources, grid: &EditGrid) {
        self.edit_grid.draw_board(resources, grid);
        let projection =
            cgmath::ortho(0.0, self.width, self.height, 0.0, -1.0, 1.0);
        self.parts_tray.draw(resources, &projection);
        self.controls_tray.draw(resources, &projection);
        self.edit_grid.draw_dragged(resources);
    }

    pub fn handle_event(&mut self, event: &Event, grid: &mut EditGrid,
                        audio: &mut AudioQueue)
                        -> bool {
        match event {
            Event::ClockTick(tick) => {
                if let Some(eval) = grid.eval_mut() {
                    // TODO: only if not paused
                    self.seconds_since_time_step += tick.elapsed;
                    while self.seconds_since_time_step >=
                        SECONDS_PER_TIME_STEP
                    {
                        self.seconds_since_time_step -= SECONDS_PER_TIME_STEP;
                        eval.eval_time_step();
                    }
                }
            }
            Event::KeyDown(key) => {
                if key.command && key.shift && key.code == Keycode::F {
                    return true;
                }
            }
            _ => {}
        }

        if let Some(opt_action) = self.controls_tray.handle_event(event) {
            match opt_action {
                None => {}
                Some(ControlsAction::Reset) => {
                    if grid.eval().is_some() {
                        audio.play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        grid.stop_eval();
                    }
                }
                Some(ControlsAction::RunOrPause) => {
                    if grid.eval().is_none() {
                        audio.play_sound(Sound::Beep);
                        self.seconds_since_time_step = 0.0;
                        grid.start_eval();
                    } else {
                        // TODO: pause
                    }
                }
                Some(ControlsAction::StepTime) => {
                    if let Some(eval) = grid.eval_mut() {
                        eval.eval_time_step();
                    }
                }
                Some(action) => {
                    debug_log!("pressed button: {:?}", action);
                }
            }
            return false;
        }

        let (opt_action, stop) = self.parts_tray.handle_event(event);
        match opt_action {
            Some(PartsAction::Grab(ctype, pt)) => {
                self.edit_grid.grab_from_parts_tray(ctype, pt);
            }
            Some(PartsAction::Drop) => {
                self.edit_grid.drop_into_parts_tray(grid);
            }
            None => {}
        }
        if stop {
            return false;
        }

        self.edit_grid.handle_event(event, grid);
        return false;
    }
}

//===========================================================================//

const GRID_CELL_SIZE: i32 = 64;
const ZONE_CENTER_SEMI_SIZE: i32 = 12;

struct EditGridView {
    width: f32,
    height: f32,
    scroll: Vector2<i32>,
    wire_model: WireModel,
    chip_drag: Option<ChipDrag>,
    wire_drag: Option<WireDrag>,
}

impl EditGridView {
    pub fn new(window_size: RectSize<u32>) -> EditGridView {
        EditGridView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            scroll: Vector2::new(0, 0),
            wire_model: WireModel::new(),
            chip_drag: None,
            wire_drag: None,
        }
    }

    pub fn draw_board(&self, resources: &Resources, grid: &EditGrid) {
        // Draw grid:
        {
            let matrix = cgmath::ortho(0.0, 1.0, 1.0, 0.0, -1.0, 1.0);
            let pixel_rect = vec4(self.scroll.x as f32,
                                  self.scroll.y as f32,
                                  self.width,
                                  self.height);
            let coords_rect = pixel_rect / (GRID_CELL_SIZE as f32);
            resources.shaders().board().draw(&matrix, coords_rect);
        }

        // Draw wires:
        let matrix = self.vp_matrix();
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

        // Draw chips (except the one being dragged, if any):
        for (coords, ctype, orient) in grid.chips() {
            if let Some(ref drag) = self.chip_drag {
                if Some(coords) == drag.old_coords {
                    continue;
                }
            }
            let x = (coords.x * GRID_CELL_SIZE) as f32;
            let y = (coords.y * GRID_CELL_SIZE) as f32;
            let mat = matrix * Matrix4::from_translation(vec3(x, y, 0.0)) *
                Matrix4::from_scale(GRID_CELL_SIZE as f32);
            self.draw_chip(resources,
                           &mat,
                           ctype,
                           orient,
                           Some((coords, grid)));
        }
    }

    pub fn draw_dragged(&self, resources: &Resources) {
        if let Some(ref drag) = self.chip_drag {
            let pt = drag.chip_topleft();
            let x = pt.x as f32;
            let y = pt.y as f32;
            let matrix = self.vp_matrix() *
                Matrix4::from_translation(vec3(x, y, 0.0)) *
                Matrix4::from_scale(GRID_CELL_SIZE as f32);
            self.draw_chip(resources,
                           &matrix,
                           drag.chip_type,
                           drag.reorient * drag.old_orient,
                           None);
        }
    }

    fn vp_matrix(&self) -> Matrix4<f32> {
        cgmath::ortho(0.0, self.width, self.height, 0.0, -1.0, 1.0) *
            Matrix4::from_translation(vec3(-self.scroll.x as f32,
                                           -self.scroll.y as f32,
                                           0.0))
    }

    fn draw_chip(&self, resources: &Resources, matrix: &Matrix4<f32>,
                 ctype: ChipType, orient: Orientation,
                 coords_and_grid: Option<(Coords, &EditGrid)>) {
        let size = orient * ctype.size();
        let (width, height) = (size.width as f32, size.height as f32);
        let margin = 0.1;
        let rect = (margin, margin, width - 2. * margin, height - 2. * margin);
        resources.shaders().solid().fill_rect(matrix, (0.3, 0.3, 0.3), rect);
        for port in ctype.ports((0, 0).into(), orient) {
            let x = port.pos.x as f32 + 0.5;
            let y = port.pos.y as f32 + 0.5;
            let angle = port.dir.angle_from_east();
            let mat = matrix * Matrix4::from_translation(vec3(x, y, 0.0)) *
                Matrix4::from_axis_angle(vec3(0.0, 0.0, 1.0), angle);
            let color = match (port.color, port.flow) {
                (PortColor::Behavior, PortFlow::Send) => (1.0, 0.5, 0.0),
                (PortColor::Behavior, PortFlow::Recv) => (0.75, 0.0, 0.0),
                (PortColor::Event, PortFlow::Send) => (0.0, 1.0, 1.0),
                (PortColor::Event, PortFlow::Recv) => (0.0, 0.0, 1.0),
            };
            let rect = (0.5 - margin, -0.3, 0.5 * margin, 0.6);
            resources.shaders().solid().fill_rect(&mat, color, rect);
        }
        match ctype {
            ChipType::Display => {
                if let Some((coords, grid)) = coords_and_grid {
                    if grid.eval().is_some() {
                        let ports = ctype.ports(coords, orient);
                        debug_assert_eq!(ports.len(), 1);
                        if let Some(value) = grid.port_value(ports[0].loc()) {
                            resources.fonts().roman().draw(matrix,
                                                           (0.15, 0.3),
                                                           Align::Center,
                                                           (0.5 * width,
                                                            0.5 * height -
                                                                0.15),
                                                           &format!("{}",
                                                                    value));
                            return;
                        }
                    }
                }
            }
            _ => {}
        }
        resources.fonts().roman().draw(matrix,
                                       (0.15, 0.3),
                                       Align::Center,
                                       (0.5 * width, 0.5 * height - 0.15),
                                       &format!("{:?}", ctype));
    }

    fn handle_event(&mut self, event: &Event, grid: &mut EditGrid) {
        match event {
            Event::KeyDown(key) => {
                match key.code {
                    Keycode::Up => self.scroll.y -= SCROLL_PER_KEYDOWN,
                    Keycode::Down => self.scroll.y += SCROLL_PER_KEYDOWN,
                    Keycode::Left => self.scroll.x -= SCROLL_PER_KEYDOWN,
                    Keycode::Right => self.scroll.x += SCROLL_PER_KEYDOWN,
                    _ => {}
                }
                // TODO: Make these hotkeys customizable by prefs.
                if let Some(ref mut drag) = self.chip_drag {
                    if !key.command && !key.shift {
                        match key.code {
                            Keycode::A => drag.flip_horz(),
                            Keycode::E => drag.rotate_cw(),
                            Keycode::Q => drag.rotate_ccw(),
                            Keycode::W => drag.flip_vert(),
                            _ => {}
                        }
                    }
                }
            }
            Event::MouseDown(mouse) => {
                if grid.eval().is_some() {
                    if mouse.left {
                        if let Some((ctype, _, coords)) =
                            grid.chip_at(self.coords_for_point(mouse.pt))
                        {
                            if ctype == ChipType::Button {
                                grid.eval_mut()
                                    .unwrap()
                                    .interaction()
                                    .press_button(coords);
                            }
                        }
                    }
                    return;
                }
                if mouse.left {
                    if let Some((ctype, orient, coords)) =
                        grid.chip_at(self.coords_for_point(mouse.pt))
                    {
                        // TODO: If mouse is within chip cell but near edge of
                        //   chip, allow for wire dragging.
                        self.chip_drag = Some(ChipDrag::new(ctype,
                                                            orient,
                                                            Some(coords),
                                                            mouse.pt));
                    } else {
                        let mut drag = WireDrag::new();
                        if drag.move_to(self.zone_for_point(mouse.pt), grid) {
                            self.wire_drag = Some(drag);
                        } else {
                            debug_log!("wire drag done (down)");
                        }
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
                if let Some(ref mut drag) = self.chip_drag {
                    drag.move_to(mouse.pt);
                }
                if let Some(mut drag) = self.wire_drag.take() {
                    if drag.move_to(self.zone_for_point(mouse.pt), grid) {
                        self.wire_drag = Some(drag);
                    } else {
                        debug_log!("wire drag done (move)");
                    }
                }
            }
            Event::MouseUp(mouse) => {
                if mouse.left {
                    if let Some(drag) = self.chip_drag.take() {
                        drag.drop_onto_board(grid);
                    }
                    if let Some(drag) = self.wire_drag.take() {
                        drag.finish(grid);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn grab_from_parts_tray(&mut self, ctype: ChipType, pt: Point2<i32>) {
        let size = ctype.size();
        let start = Point2::new(size.width, size.height) *
            (GRID_CELL_SIZE / 2) - self.scroll;
        let mut drag =
            ChipDrag::new(ctype, Orientation::default(), None, start);
        drag.move_to(pt);
        self.chip_drag = Some(drag);
    }

    pub fn drop_into_parts_tray(&mut self, grid: &mut EditGrid) {
        if let Some(drag) = self.chip_drag.take() {
            drag.drop_into_parts_tray(grid);
        }
    }

    fn coords_for_point(&self, pt: Point2<i32>) -> Coords {
        let scrolled = pt + self.scroll;
        Coords::new(div_floor(scrolled.x, GRID_CELL_SIZE),
                    div_floor(scrolled.y, GRID_CELL_SIZE))
    }

    fn zone_for_point(&self, pt: Point2<i32>) -> Zone {
        let scrolled = pt + self.scroll;
        let coords = Coords::new(div_floor(scrolled.x, GRID_CELL_SIZE),
                                 div_floor(scrolled.y, GRID_CELL_SIZE));
        let x = mod_floor(scrolled.x, GRID_CELL_SIZE) - GRID_CELL_SIZE / 2;
        let y = mod_floor(scrolled.y, GRID_CELL_SIZE) - GRID_CELL_SIZE / 2;
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
    let cx = (coords.x * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32;
    let cy = (coords.y * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32;
    matrix * Matrix4::from_translation(vec3(cx, cy, 0.0)) *
        Matrix4::from_axis_angle(vec3(0.0, 0.0, 1.0), dir.angle_from_east()) *
        Matrix4::from_scale((GRID_CELL_SIZE / 2) as f32)
}

//===========================================================================//

struct ChipDrag {
    chip_type: ChipType,
    old_orient: Orientation,
    old_coords: Option<Coords>,
    drag_start: Point2<i32>,
    drag_current: Point2<i32>,
    reorient: Orientation,
}

impl ChipDrag {
    pub fn new(chip_type: ChipType, old_orient: Orientation,
               old_coords: Option<Coords>, drag_start: Point2<i32>)
               -> ChipDrag {
        ChipDrag {
            chip_type,
            old_orient,
            old_coords,
            drag_start,
            drag_current: drag_start,
            reorient: Orientation::default(),
        }
    }

    pub fn chip_topleft(&self) -> Point2<i32> {
        let coords_topleft = if let Some(coords) = self.old_coords {
            coords * GRID_CELL_SIZE
        } else {
            Point2::new(0, 0)
        };
        coords_topleft + (self.drag_current - self.drag_start)
    }

    pub fn flip_horz(&mut self) { self.reorient = self.reorient.flip_horz(); }

    pub fn flip_vert(&mut self) { self.reorient = self.reorient.flip_vert(); }

    pub fn rotate_cw(&mut self) { self.reorient = self.reorient.rotate_cw(); }

    pub fn rotate_ccw(&mut self) {
        self.reorient = self.reorient.rotate_ccw();
    }

    pub fn move_to(&mut self, mouse_pt: Point2<i32>) {
        self.drag_current = mouse_pt;
    }

    pub fn drop_onto_board(self, grid: &mut EditGrid) {
        let pt = self.chip_topleft();
        let new_coords =
            Coords::new(div_floor(pt.x + GRID_CELL_SIZE / 2, GRID_CELL_SIZE),
                        div_floor(pt.y + GRID_CELL_SIZE / 2, GRID_CELL_SIZE));
        let new_size = self.reorient * self.old_orient * self.chip_type.size();
        // TODO: Allow moving a large-size chip onto a position that overlaps
        //   its old position.
        if grid.can_place_chip(new_coords, new_size) {
            let mut changes = Vec::<GridChange>::new();
            if let Some(old_coords) = self.old_coords {
                changes.push(GridChange::ToggleChip(old_coords,
                                                    self.old_orient,
                                                    self.chip_type));
            }
            changes.push(GridChange::ToggleChip(new_coords,
                                                self.reorient *
                                                    self.old_orient,
                                                self.chip_type));
            grid.mutate(&changes);
        }
    }

    pub fn drop_into_parts_tray(self, grid: &mut EditGrid) {
        if let Some(old_coords) = self.old_coords {
            grid.mutate(
                &[
                    GridChange::ToggleChip(
                        old_coords,
                        self.old_orient,
                        self.chip_type,
                    ),
                ],
            );
        }
    }
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
            (_, Some(Zone::East(coords1)), Zone::South(coords2)) => {
                if coords1 == coords2 {
                    self.try_turn_left(coords1, Direction::East, grid)
                } else if coords1 + Direction::North == coords2 {
                    self.try_turn_left(coords1, Direction::North, grid)
                } else if coords1 + Direction::East == coords2 {
                    self.try_turn_left(coords2, Direction::South, grid)
                } else if coords1 + Direction::East ==
                           coords2 + Direction::South
                {
                    self.try_turn_left(coords1 + Direction::East,
                                       Direction::West,
                                       grid)
                } else {
                    true
                }
            }
            (_, Some(Zone::South(coords1)), Zone::East(coords2)) => {
                if coords1 == coords2 {
                    self.try_turn_left(coords1, Direction::East, grid)
                } else if coords1 + Direction::South == coords2 {
                    self.try_turn_left(coords2, Direction::North, grid)
                } else if coords1 + Direction::West == coords2 {
                    self.try_turn_left(coords1, Direction::South, grid)
                } else if coords1 + Direction::South ==
                           coords2 + Direction::East
                {
                    self.try_turn_left(coords1 + Direction::South,
                                       Direction::West,
                                       grid)
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

    pub fn finish(mut self, grid: &mut EditGrid) {
        match (self.changed, self.prev, self.curr) {
            (_, Some(Zone::East(coords1)), Some(Zone::Center(coords2))) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::East, grid);
                } else if coords1 + Direction::East == coords2 {
                    self.try_split(coords2, Direction::West, grid);
                }
            }
            (_, Some(Zone::South(coords1)), Some(Zone::Center(coords2))) => {
                if coords1 == coords2 {
                    self.try_split(coords1, Direction::South, grid);
                } else if coords1 + Direction::South == coords2 {
                    self.try_split(coords2, Direction::North, grid);
                }
            }
            (false, None, Some(Zone::Center(coords))) => {
                self.try_toggle_cross(coords, grid);
            }
            (false, None, Some(Zone::East(coords))) => {
                self.try_remove_stub(coords, Direction::East, grid);
            }
            (false, None, Some(Zone::South(coords))) => {
                self.try_remove_stub(coords, Direction::South, grid);
            }
            (_, _, _) => {}
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

    fn try_turn_left(&mut self, coords: Coords, dir: Direction,
                     grid: &mut EditGrid)
                     -> bool {
        let dir2 = dir.rotate_cw();
        match (grid.wire_shape_at(coords, dir),
                 grid.wire_shape_at(coords, dir2)) {
            (Some(WireShape::Stub), Some(WireShape::Stub)) => {
                grid.mutate(
                    &[GridChange::ToggleCenterWire(coords, dir, dir2)],
                );
                self.changed = true;
                true
            }
            (Some(WireShape::Stub), None) => {
                grid.mutate(
                    &[
                        GridChange::ToggleStubWire(coords, dir2),
                        GridChange::ToggleCenterWire(coords, dir, dir2),
                    ],
                );
                self.changed = true;
                true
            }
            (None, Some(WireShape::Stub)) => {
                grid.mutate(
                    &[
                        GridChange::ToggleStubWire(coords, dir),
                        GridChange::ToggleCenterWire(coords, dir, dir2),
                    ],
                );
                self.changed = true;
                true
            }
            (Some(WireShape::TurnLeft), Some(WireShape::TurnRight)) => {
                grid.mutate(
                    &[GridChange::ToggleCenterWire(coords, dir, dir2)],
                );
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
