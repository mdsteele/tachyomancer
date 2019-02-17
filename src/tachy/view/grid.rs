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

use super::chip::ChipModel;
use super::tooltip::Tooltip;
use super::wire::WireModel;
use cgmath::{self, Matrix4, Point2, Vector2, vec4};
use num_integer::{div_floor, mod_floor};
use std::u32;
use tachy::geom::{Color4, Coords, CoordsRect, Direction, MatrixExt,
                  Orientation, Rect, RectSize};
use tachy::gui::{AudioQueue, Event, Keycode, Resources, Sound};
use tachy::save::{Hotkey, Prefs, WireShape};
use tachy::state::{ChipType, EditGrid, GridChange, WireColor};

//===========================================================================//

const BOUNDS_MARGIN: i32 = 30;
const GRID_CELL_SIZE: i32 = 64;
const SCROLL_PER_KEYDOWN: i32 = 40;
const ZONE_CENTER_SEMI_SIZE: i32 = 12;

//===========================================================================//

pub enum EditGridAction {
    EditConst(Coords, u32),
}

//===========================================================================//

pub struct EditGridView {
    width: f32,
    height: f32,
    scroll: Vector2<i32>,
    chip_model: ChipModel,
    wire_model: WireModel,
    bounds_drag: Option<BoundsDrag>,
    chip_drag: Option<ChipDrag>,
    wire_drag: Option<WireDrag>,
    tooltip: Tooltip<GridTooltipTag>,
}

impl EditGridView {
    pub fn new(window_size: RectSize<i32>) -> EditGridView {
        EditGridView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            scroll: Vector2::new(-window_size.width / 2,
                                 -window_size.height / 2),
            chip_model: ChipModel::new(),
            wire_model: WireModel::new(),
            bounds_drag: None,
            chip_drag: None,
            wire_drag: None,
            tooltip: Tooltip::new(window_size),
        }
    }

    fn draw_background_grid(&self, resources: &Resources) {
        let matrix = cgmath::ortho(0.0, 1.0, 1.0, 0.0, -1.0, 1.0);
        let pixel_rect = vec4(self.scroll.x as f32,
                              self.scroll.y as f32,
                              self.width,
                              self.height);
        let coords_rect = pixel_rect / (GRID_CELL_SIZE as f32);
        resources.shaders().board().draw(&matrix, coords_rect);
    }

    fn draw_bounds(&self, resources: &Resources, grid: &EditGrid) {
        let matrix = self.vp_matrix();
        let (bounds, acceptable) = if let Some(ref drag) = self.bounds_drag {
            (drag.bounds, drag.acceptable)
        } else {
            (grid.bounds(), true)
        };
        let x = (bounds.x * GRID_CELL_SIZE) as f32;
        let y = (bounds.y * GRID_CELL_SIZE) as f32;
        let width = (bounds.width * GRID_CELL_SIZE) as f32;
        let height = (bounds.height * GRID_CELL_SIZE) as f32;
        let thick = BOUNDS_MARGIN as f32;
        let color = if acceptable {
            Color4::PURPLE2.rgb()
        } else {
            (1.0, 0.0, 0.0)
        };
        let rect = Rect::new(x - thick, y, thick, height);
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        let rect = Rect::new(x, y - thick, width, thick);
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        let rect = Rect::new(x + width, y, thick, height);
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        let rect = Rect::new(x, y + height, width, thick);
        resources.shaders().solid().fill_rect(&matrix, color, rect);
    }

    fn draw_interfaces(&self, resources: &Resources, matrix: &Matrix4<f32>,
                       grid: &EditGrid) {
        let bounds = if let Some(ref drag) = self.bounds_drag {
            drag.bounds
        } else {
            grid.bounds()
        };
        for interface in grid.interfaces() {
            let coords = interface.top_left(bounds);
            let x = (coords.x * GRID_CELL_SIZE) as f32;
            let y = (coords.y * GRID_CELL_SIZE) as f32;
            let mat = matrix * Matrix4::trans2(x, y) *
                Matrix4::from_scale(GRID_CELL_SIZE as f32);
            self.chip_model.draw_interface(resources, &mat, interface);
        }
    }

    pub fn draw_board(&self, resources: &Resources, grid: &EditGrid) {
        self.draw_background_grid(resources);
        self.draw_bounds(resources, grid);

        // Draw wires:
        let matrix = self.vp_matrix();
        for (coords, dir, shape, size, color, has_error) in
            grid.wire_fragments()
        {
            let color = if has_error {
                WireColor::Ambiguous
            } else {
                color
            };
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

        self.draw_interfaces(resources, &matrix, grid);

        // Draw chips (except the one being dragged, if any):
        for (coords, ctype, orient) in grid.chips() {
            if let Some(ref drag) = self.chip_drag {
                if Some(coords) == drag.old_coords {
                    continue;
                }
            }
            let x = (coords.x * GRID_CELL_SIZE) as f32;
            let y = (coords.y * GRID_CELL_SIZE) as f32;
            let mat = matrix * Matrix4::trans2(x, y) *
                Matrix4::from_scale(GRID_CELL_SIZE as f32);
            self.chip_model.draw_chip(resources,
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
            let matrix = self.vp_matrix() * Matrix4::trans2(x, y) *
                Matrix4::from_scale(GRID_CELL_SIZE as f32);
            self.chip_model.draw_chip(resources,
                                      &matrix,
                                      drag.chip_type,
                                      drag.reorient * drag.old_orient,
                                      None);
        }
    }

    pub fn draw_tooltip(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        self.tooltip.draw(resources, matrix);
    }

    fn vp_matrix(&self) -> Matrix4<f32> {
        cgmath::ortho(0.0, self.width, self.height, 0.0, -1.0, 1.0) *
            Matrix4::trans2(-self.scroll.x as f32, -self.scroll.y as f32)
    }

    pub fn on_event(&mut self, event: &Event, grid: &mut EditGrid,
                    prefs: &Prefs, audio: &mut AudioQueue)
                    -> Option<EditGridAction> {
        match event {
            Event::ClockTick(tick) => {
                self.tooltip
                    .tick(tick, prefs, |tag| tooltip_format(grid, tag));
            }
            Event::KeyDown(key) => {
                self.tooltip.stop_hover_all();
                if key.command {
                    match key.code {
                        Keycode::Z if key.shift => grid.redo(),
                        Keycode::Z => grid.undo(),
                        _ => {}
                    }
                } else {
                    match prefs.hotkey_for_code(key.code) {
                        Some(Hotkey::ScrollUp) => {
                            self.scroll.y -= SCROLL_PER_KEYDOWN
                        }
                        Some(Hotkey::ScrollDown) => {
                            self.scroll.y += SCROLL_PER_KEYDOWN
                        }
                        Some(Hotkey::ScrollLeft) => {
                            self.scroll.x -= SCROLL_PER_KEYDOWN
                        }
                        Some(Hotkey::ScrollRight) => {
                            self.scroll.x += SCROLL_PER_KEYDOWN
                        }
                        Some(hotkey) => {
                            if let Some(ref mut drag) = self.chip_drag {
                                match hotkey {
                                    Hotkey::FlipHorz => drag.flip_horz(),
                                    Hotkey::FlipVert => drag.flip_vert(),
                                    Hotkey::RotateCcw => drag.rotate_ccw(),
                                    Hotkey::RotateCw => drag.rotate_cw(),
                                    _ => {}
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
            Event::MouseDown(mouse) => {
                self.tooltip.stop_hover_all();
                if grid.eval().is_some() {
                    if mouse.left {
                        if let Some((coords, ctype, _)) =
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
                    return None;
                }
                if mouse.left {
                    let mouse_coords = self.coords_for_point(mouse.pt);
                    if !grid.bounds().contains_point(mouse_coords) {
                        if let Some(octant) =
                            self.octant_for_point(mouse.pt, grid)
                        {
                            self.bounds_drag =
                                Some(BoundsDrag::new(octant, mouse.pt, grid));
                        }
                    } else if let Some((coords, ctype, orient)) =
                        grid.chip_at(mouse_coords)
                    {
                        // TODO: If mouse is within chip cell but near edge of
                        //   chip, allow for wire dragging.
                        self.chip_drag = Some(ChipDrag::new(ctype,
                                                            orient,
                                                            Some(coords),
                                                            mouse.pt));
                        audio.play_sound(Sound::GrabChip);
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
                    if let Some((_, ChipType::Const(value), _)) =
                        grid.chip_at(coords)
                    {
                        return Some(EditGridAction::EditConst(coords, value));
                    }
                    let east = grid.wire_shape_at(coords, Direction::East);
                    if east == Some(WireShape::Cross) ||
                        (east == Some(WireShape::Straight) &&
                             grid.wire_shape_at(coords, Direction::South) ==
                                 Some(WireShape::Straight))
                    {
                        grid.mutate(vec![GridChange::ToggleCrossWire(coords)]);
                    }
                }
            }
            Event::MouseMove(mouse) => {
                if let Some(ref mut drag) = self.bounds_drag {
                    drag.move_to(mouse.pt, grid);
                } else if let Some(ref mut drag) = self.chip_drag {
                    drag.move_to(mouse.pt);
                } else if let Some(mut drag) = self.wire_drag.take() {
                    if drag.move_to(self.zone_for_point(mouse.pt), grid) {
                        self.wire_drag = Some(drag);
                    } else {
                        debug_log!("wire drag done (move)");
                    }
                } else if !mouse.left && !mouse.right {
                    if let Some(tag) =
                        self.tooltip_tag_for_pt(grid, mouse.pt)
                    {
                        self.tooltip.start_hover(tag, mouse.pt);
                    } else {
                        self.tooltip.stop_hover_all();
                    }
                }
            }
            Event::MouseUp(mouse) => {
                if mouse.left {
                    if let Some(drag) = self.bounds_drag.take() {
                        drag.finish(grid);
                    }
                    if let Some(drag) = self.chip_drag.take() {
                        drag.drop_onto_board(grid);
                        audio.play_sound(Sound::DropChip);
                    }
                    if let Some(drag) = self.wire_drag.take() {
                        drag.finish(grid);
                    }
                }
            }
            Event::Scroll(scroll) => {
                self.tooltip.stop_hover_all();
                self.scroll += scroll.delta;
            }
            Event::Unfocus => {
                self.tooltip.stop_hover_all();
            }
            _ => {}
        }
        return None;
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

    fn octant_for_point(&self, pt: Point2<i32>, grid: &EditGrid)
                        -> Option<Octant> {
        let scrolled = pt + self.scroll;
        let bounds = grid.bounds();
        let inner = bounds * GRID_CELL_SIZE;
        if inner.contains_point(scrolled) {
            return None;
        }
        let outer = Rect::new(inner.x - BOUNDS_MARGIN,
                              inner.y - BOUNDS_MARGIN,
                              inner.width + 2 * BOUNDS_MARGIN,
                              inner.height + 2 * BOUNDS_MARGIN);
        if !outer.contains_point(scrolled) {
            return None;
        }
        let at_top = scrolled.y < inner.y;
        let at_bottom = scrolled.y >= inner.bottom();
        if scrolled.x < inner.x {
            if at_top {
                Some(Octant::TopLeft)
            } else if at_bottom {
                Some(Octant::BottomLeft)
            } else {
                Some(Octant::Left)
            }
        } else if scrolled.x >= inner.right() {
            if at_top {
                Some(Octant::TopRight)
            } else if at_bottom {
                Some(Octant::BottomRight)
            } else {
                Some(Octant::Right)
            }
        } else if at_top {
            Some(Octant::Top)
        } else if at_bottom {
            Some(Octant::Bottom)
        } else {
            None
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

    fn tooltip_tag_for_pt(&self, grid: &EditGrid, pt: Point2<i32>)
                          -> Option<GridTooltipTag> {
        let scrolled = pt + self.scroll;
        let coords = Coords::new(div_floor(scrolled.x, GRID_CELL_SIZE),
                                 div_floor(scrolled.y, GRID_CELL_SIZE));
        if let Some((coords, ctype, _)) = grid.chip_at(coords) {
            return Some(GridTooltipTag::Chip(coords, ctype));
        }
        if let Some((index, _)) = grid.interface_at(coords) {
            return Some(GridTooltipTag::Interface(index));
        }
        // TODO: Figure out if we're actually visibly over the wire shape,
        //   rather than just treating each wire shape as a triangle.
        let x = mod_floor(scrolled.x, GRID_CELL_SIZE);
        let y = mod_floor(scrolled.y, GRID_CELL_SIZE);
        let dir = if x > y {
            if x > (GRID_CELL_SIZE - y) {
                Direction::East
            } else {
                Direction::North
            }
        } else {
            if y > (GRID_CELL_SIZE - x) {
                Direction::South
            } else {
                Direction::West
            }
        };
        if let Some(index) = grid.wire_index_at(coords, dir) {
            // TODO: When hovering over a wire with an error, we should hilight
            //   the causes of the error (e.g. the two sender ports, or the
            //   wire loop, or whatever).
            return Some(GridTooltipTag::Wire(index));
        }
        return None;
    }
}

fn coords_matrix(matrix: &Matrix4<f32>, coords: Coords, dir: Direction)
                 -> Matrix4<f32> {
    let cx = (coords.x * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32;
    let cy = (coords.y * GRID_CELL_SIZE + GRID_CELL_SIZE / 2) as f32;
    matrix * Matrix4::trans2(cx, cy) *
        Matrix4::from_angle_z(dir.angle_from_east()) *
        Matrix4::from_scale((GRID_CELL_SIZE / 2) as f32)
}

fn tooltip_format(grid: &EditGrid, tag: &GridTooltipTag) -> String {
    match *tag {
        GridTooltipTag::Chip(_, ctype) => ctype.tooltip_format(),
        GridTooltipTag::Interface(index) => {
            grid.interfaces()[index].tooltip_format()
        }
        GridTooltipTag::Wire(index) => grid.wire_tooltip_format(index),
    }
}

//===========================================================================//

#[derive(Clone, Copy)]
enum Octant {
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    Top,
    TopRight,
}

//===========================================================================//

struct BoundsDrag {
    octant: Octant,
    drag_start: Point2<i32>,
    drag_current: Point2<i32>,
    bounds: CoordsRect,
    acceptable: bool,
}

impl BoundsDrag {
    pub fn new(octant: Octant, mouse_pt: Point2<i32>, grid: &mut EditGrid)
               -> BoundsDrag {
        BoundsDrag {
            octant,
            drag_start: mouse_pt,
            drag_current: mouse_pt,
            bounds: grid.bounds(),
            acceptable: true,
        }
    }

    pub fn move_to(&mut self, mouse_pt: Point2<i32>, grid: &EditGrid) {
        self.drag_current = mouse_pt;
        let delta = (self.drag_current - self.drag_start) / GRID_CELL_SIZE;
        let old_bounds = grid.bounds();
        let mut left = old_bounds.x;
        let mut right = old_bounds.x + old_bounds.width;
        match self.octant {
            Octant::TopLeft | Octant::Left | Octant::BottomLeft => {
                left = (left + delta.x).min(right - 1);
            }
            Octant::TopRight | Octant::Right | Octant::BottomRight => {
                right = (right + delta.x).max(left + 1);
            }
            Octant::Top | Octant::Bottom => {}
        }
        let mut top = old_bounds.y;
        let mut bottom = old_bounds.y + old_bounds.height;
        match self.octant {
            Octant::TopLeft | Octant::Top | Octant::TopRight => {
                top = (top + delta.y).min(bottom - 1);
            }
            Octant::BottomLeft | Octant::Bottom | Octant::BottomRight => {
                bottom = (bottom + delta.y).max(top + 1);
            }
            Octant::Left | Octant::Right => {}
        }
        self.bounds = Rect::new(left, top, right - left, bottom - top);
        self.acceptable = grid.can_have_bounds(self.bounds);
    }

    pub fn finish(self, grid: &mut EditGrid) {
        debug_assert_eq!(self.acceptable, grid.can_have_bounds(self.bounds));
        if self.acceptable {
            let old_bounds = grid.bounds();
            grid.mutate(vec![GridChange::SwapBounds(old_bounds, self.bounds)]);
        }
    }
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
            grid.mutate(changes);
        }
    }

    pub fn drop_into_parts_tray(self, grid: &mut EditGrid) {
        if let Some(old_coords) = self.old_coords {
            grid.mutate(vec![
                GridChange::ToggleChip(
                    old_coords,
                    self.old_orient,
                    self.chip_type
                ),
            ]);
        }
    }
}

//===========================================================================//

struct WireDrag {
    curr: Option<Zone>,
    prev: Option<Zone>,
    changed: bool,
}

// TODO: enforce wires must be in bounds
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
                grid.mutate(vec![GridChange::ToggleStubWire(coords, dir)]);
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
                grid.mutate(vec![GridChange::ToggleStubWire(coords, dir)]);
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
                grid.mutate(vec![GridChange::ToggleCrossWire(coords)]);
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
                grid.mutate(vec![
                    GridChange::ToggleStubWire(coords, dir),
                    GridChange::ToggleCenterWire(coords, dir, -dir),
                ]);
                self.changed = true;
                true
            }
            (Some(WireShape::Stub), Some(WireShape::Stub)) => {
                grid.mutate(
                    vec![GridChange::ToggleCenterWire(coords, dir, -dir)],
                );
                self.changed = true;
                true
            }
            (Some(WireShape::Straight), Some(WireShape::Straight)) => {
                if grid.wire_shape_at(coords - dir, dir) ==
                    Some(WireShape::Stub)
                {
                    grid.mutate(vec![
                        GridChange::ToggleCenterWire(coords, dir, -dir),
                        GridChange::ToggleStubWire(coords, -dir),
                    ]);
                } else {
                    grid.mutate(
                        vec![GridChange::ToggleCenterWire(coords, dir, -dir)],
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
                    vec![GridChange::ToggleCenterWire(coords, dir, dir2)],
                );
                self.changed = true;
                true
            }
            (Some(WireShape::Stub), None) => {
                grid.mutate(vec![
                    GridChange::ToggleStubWire(coords, dir2),
                    GridChange::ToggleCenterWire(coords, dir, dir2),
                ]);
                self.changed = true;
                true
            }
            (None, Some(WireShape::Stub)) => {
                grid.mutate(vec![
                    GridChange::ToggleStubWire(coords, dir),
                    GridChange::ToggleCenterWire(coords, dir, dir2),
                ]);
                self.changed = true;
                true
            }
            (Some(WireShape::TurnLeft), Some(WireShape::TurnRight)) => {
                grid.mutate(
                    vec![GridChange::ToggleCenterWire(coords, dir, dir2)],
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
                grid.mutate(vec![GridChange::ToggleSplitWire(coords, dir)]);
                self.changed = true;
                true
            }
            (None, Some(WireShape::TurnLeft), _) |
            (None, Some(WireShape::TurnRight), _) |
            (None, Some(WireShape::SplitTee), _) |
            (None, _, Some(WireShape::Straight)) => {
                grid.mutate(vec![
                    GridChange::ToggleStubWire(coords, dir),
                    GridChange::ToggleSplitWire(coords, dir),
                ]);
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

#[derive(Eq, PartialEq)]
enum GridTooltipTag {
    Chip(Coords, ChipType),
    Interface(usize),
    Wire(usize),
}

//===========================================================================//
