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
use cgmath::{self, Matrix4, MetricSpace, Point2, Vector2, vec2, vec4};
use std::collections::{HashMap, HashSet};
use std::mem;
use tachy::geom::{AsFloat, AsInt, Color4, Coords, CoordsDelta, CoordsRect,
                  Direction, MatrixExt, Orientation, Rect, RectSize};
use tachy::gui::{AudioQueue, Event, Keycode, Resources, Sound};
use tachy::save::{ChipType, Hotkey, Prefs, WireShape};
use tachy::state::{EditGrid, GridChange, WireColor};

//===========================================================================//

// The thickness, in grid cells, of the bounds margin:
const BOUNDS_MARGIN: f32 = 0.45;

// The size, in screen pixels, of a grid cell at 1x zoom:
const GRID_CELL_SIZE: i32 = 64;

// Number of screen pixels to scroll by when pressing a scroll hotkey:
const SCROLL_PER_KEYDOWN: i32 = 40;

// How close, in grid cells, the mouse must be to a grid vertex to start a
// selection rect.
const SELECTING_VERTEX_MAX_DIST: f32 = 0.2;

const SELECTION_BOX_COLOR1: Color4 = Color4::CYAN5.with_alpha(0.75);
const SELECTION_BOX_COLOR2: Color4 = Color4::CYAN4.with_alpha(0.75);
const SELECTION_BOX_COLOR3: Color4 = Color4::CYAN4.with_alpha(0.1);

const ZONE_CENTER_SEMI_SIZE: f32 = 0.1875;

// The minimum zoom multiplier (i.e. how far zoomed out you can be):
const ZOOM_MIN: f32 = 0.25;
// The maximum zoom multiplier (i.e. how far zoomed in you can be):
const ZOOM_MAX: f32 = 1.0;
// How much to multiply/divide the zoom by when pressing a zoom hotkey:
const ZOOM_PER_KEYDOWN: f32 = 1.415; // slightly more than sqrt(2)

//===========================================================================//

pub enum EditGridAction {
    EditConst(Coords, u32),
}

//===========================================================================//

pub struct EditGridView {
    width: f32,
    height: f32,
    scroll: Vector2<i32>,
    zoom: f32,
    chip_model: ChipModel,
    wire_model: WireModel,
    interaction: Interaction,
    tooltip: Tooltip<GridTooltipTag>,
}

impl EditGridView {
    pub fn new(window_size: RectSize<i32>) -> EditGridView {
        EditGridView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            scroll: Vector2::new(0, 0),
            zoom: ZOOM_MAX,
            chip_model: ChipModel::new(),
            wire_model: WireModel::new(),
            interaction: Interaction::Nothing,
            tooltip: Tooltip::new(window_size),
        }
    }

    fn draw_background_grid(&self, resources: &Resources) {
        let matrix = cgmath::ortho(0.0, 1.0, 1.0, 0.0, -1.0, 1.0);
        let width = self.width / self.zoom;
        let height = self.height / self.zoom;
        let pixel_rect = vec4((self.scroll.x as f32) - 0.5 * width,
                              (self.scroll.y as f32) - 0.5 * height,
                              width,
                              height);
        let coords_rect = pixel_rect / (GRID_CELL_SIZE as f32);
        resources.shaders().board().draw(&matrix, coords_rect);
    }

    fn draw_bounds(&self, resources: &Resources, grid: &EditGrid) {
        let matrix = self.vp_matrix();
        let (bounds, acceptable) = match self.interaction {
            Interaction::DraggingBounds(ref drag) => {
                (drag.bounds, drag.acceptable)
            }
            _ => (grid.bounds(), true),
        };
        let x = (bounds.x * GRID_CELL_SIZE) as f32;
        let y = (bounds.y * GRID_CELL_SIZE) as f32;
        let width = (bounds.width * GRID_CELL_SIZE) as f32;
        let height = (bounds.height * GRID_CELL_SIZE) as f32;
        let thick = BOUNDS_MARGIN * (GRID_CELL_SIZE as f32);
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
        let bounds = match self.interaction {
            Interaction::DraggingBounds(ref drag) => drag.bounds,
            _ => grid.bounds(),
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

    fn draw_selection_box(&self, resources: &Resources,
                          matrix: &Matrix4<f32>, selected_rect: CoordsRect,
                          delta: Vector2<f32>) {
        let ui = resources.shaders().ui();
        let rect = (selected_rect * GRID_CELL_SIZE).expand(4).as_f32() +
            delta * (GRID_CELL_SIZE as f32);
        ui.draw_selection_box(matrix,
                              &rect,
                              &SELECTION_BOX_COLOR1,
                              &SELECTION_BOX_COLOR2,
                              &SELECTION_BOX_COLOR3);
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
        let dragged_chip_coords = match self.interaction {
            Interaction::DraggingChip(ref drag) => drag.old_coords,
            _ => None,
        };
        for (coords, ctype, orient) in grid.chips() {
            if Some(coords) == dragged_chip_coords {
                continue;
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

        // Draw selection box (if any):
        match self.interaction {
            Interaction::SelectingRect(ref drag) => {
                self.draw_selection_box(resources,
                                        &matrix,
                                        drag.rect,
                                        vec2(0.0, 0.0));
            }
            Interaction::RectSelected(rect) => {
                self.draw_selection_box(resources,
                                        &matrix,
                                        rect,
                                        vec2(0.0, 0.0));
            }
            Interaction::DraggingSelection(ref drag) => {
                self.draw_selection_box(resources,
                                        &matrix,
                                        drag.reoriented_selected_rect(),
                                        drag.delta());
            }
            _ => {}
        }
    }

    pub fn draw_dragged(&self, resources: &Resources) {
        if let Interaction::DraggingChip(ref drag) = self.interaction {
            let pt = drag.chip_topleft();
            let matrix = self.vp_matrix() *
                Matrix4::from_scale(GRID_CELL_SIZE as f32) *
                Matrix4::trans2(pt.x, pt.y);
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
            Matrix4::trans2(0.5 * self.width, 0.5 * self.height) *
            Matrix4::from_scale(self.zoom) *
            Matrix4::trans2(-self.scroll.x as f32, -self.scroll.y as f32)
    }

    fn on_hotkey(&mut self, hotkey: Hotkey) {
        match hotkey {
            Hotkey::ScrollUp => {
                self.scroll_by_screen_dist(0, -SCROLL_PER_KEYDOWN);
            }
            Hotkey::ScrollDown => {
                self.scroll_by_screen_dist(0, SCROLL_PER_KEYDOWN);
            }
            Hotkey::ScrollLeft => {
                self.scroll_by_screen_dist(-SCROLL_PER_KEYDOWN, 0);
            }
            Hotkey::ScrollRight => {
                self.scroll_by_screen_dist(SCROLL_PER_KEYDOWN, 0);
            }
            Hotkey::ZoomIn => {
                self.zoom_by(ZOOM_PER_KEYDOWN);
            }
            Hotkey::ZoomOut => {
                self.zoom_by(1.0 / ZOOM_PER_KEYDOWN);
            }
            _ => {
                match self.interaction {
                    Interaction::DraggingChip(ref mut drag) => {
                        match hotkey {
                            Hotkey::FlipHorz => drag.flip_horz(),
                            Hotkey::FlipVert => drag.flip_vert(),
                            Hotkey::RotateCcw => drag.rotate_ccw(),
                            Hotkey::RotateCw => drag.rotate_cw(),
                            _ => {}
                        }
                    }
                    Interaction::DraggingSelection(ref mut drag) => {
                        match hotkey {
                            Hotkey::FlipHorz => drag.flip_horz(),
                            Hotkey::FlipVert => drag.flip_vert(),
                            Hotkey::RotateCcw => drag.rotate_ccw(),
                            Hotkey::RotateCw => drag.rotate_cw(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
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
                        Keycode::A => {
                            self.interaction =
                                Interaction::RectSelected(grid.bounds());
                        }
                        Keycode::X => {
                            if let Interaction::RectSelected(rect) =
                                self.interaction
                            {
                                self.cut_region(grid, rect);
                                self.interaction = Interaction::Nothing;
                            }
                        }
                        Keycode::Z if key.shift => grid.redo(),
                        Keycode::Z => grid.undo(),
                        _ => {}
                    }
                } else if let Some(hotkey) = prefs.hotkey_for_code(key.code) {
                    self.on_hotkey(hotkey);
                }
            }
            Event::MouseDown(mouse) => {
                let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                self.tooltip.stop_hover_all();
                if let Interaction::RectSelected(rect) = self.interaction {
                    if mouse.left &&
                        rect.contains_point(grid_pt.as_i32_floor())
                    {
                        let drag = SelectionDrag::new(rect, grid_pt);
                        self.interaction =
                            Interaction::DraggingSelection(drag);
                        return None;
                    }
                    self.interaction = Interaction::Nothing;
                }
                if grid.eval().is_some() {
                    if mouse.left {
                        if let Some((coords, ctype, _)) =
                            grid.chip_at(grid_pt.as_i32_floor())
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
                    if SelectingDrag::is_near_vertex(grid_pt, grid.bounds()) {
                        let drag = SelectingDrag::new(grid.bounds(),
                                                      grid_pt.as_i32_round());
                        self.interaction = Interaction::SelectingRect(drag);
                        return None;
                    }
                    let mouse_coords = grid_pt.as_i32_floor();
                    if !grid.bounds().contains_point(mouse_coords) {
                        if let Some(octant) =
                            self.bounds_octant_for_screen_pt(mouse.pt, grid)
                        {
                            let drag = BoundsDrag::new(octant, grid_pt, grid);
                            self.interaction =
                                Interaction::DraggingBounds(drag);
                        }
                    } else if let Some((coords, ctype, orient)) =
                        grid.chip_at(mouse_coords)
                    {
                        // TODO: If mouse is within chip cell but near edge of
                        //   chip, allow for wire dragging.
                        let drag = ChipDrag::new(ctype,
                                                 orient,
                                                 Some(coords),
                                                 grid_pt);
                        self.interaction = Interaction::DraggingChip(drag);
                        audio.play_sound(Sound::GrabChip);
                    } else {
                        let mut drag = WireDrag::new();
                        if drag.move_to(Zone::from_grid_pt(grid_pt), grid) {
                            self.interaction =
                                Interaction::DraggingWires(drag);
                        } else {
                            debug_log!("wire drag done (down)");
                        }
                    }
                } else if mouse.right {
                    let coords = self.coords_for_screen_pt(mouse.pt);
                    if let Some((_, ChipType::Const(value), _)) =
                        grid.chip_at(coords)
                    {
                        return Some(EditGridAction::EditConst(coords, value));
                    }
                    let change = GridChange::ToggleCrossWire(coords);
                    if grid.try_mutate(vec![change]) {
                        // TODO: Play sound.
                    }
                }
            }
            Event::MouseMove(mouse) => {
                let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                let mut should_stop_interaction = false;
                match self.interaction {
                    Interaction::Nothing |
                    Interaction::RectSelected(_) => {
                        if !mouse.left && !mouse.right {
                            if let Some(tag) =
                                self.tooltip_tag_for_grid_pt(grid, grid_pt)
                            {
                                self.tooltip.start_hover(tag, mouse.pt);
                            } else {
                                self.tooltip.stop_hover_all();
                            }
                        }
                    }
                    Interaction::DraggingBounds(ref mut drag) => {
                        drag.move_to(grid_pt, grid);
                    }
                    Interaction::DraggingChip(ref mut drag) => {
                        drag.move_to(grid_pt);
                    }
                    Interaction::SelectingRect(ref mut drag) => {
                        drag.move_to(grid_pt);
                    }
                    Interaction::DraggingSelection(ref mut drag) => {
                        drag.move_to(grid_pt);
                    }
                    Interaction::DraggingWires(ref mut drag) => {
                        if !drag.move_to(Zone::from_grid_pt(grid_pt), grid) {
                            debug_log!("wire drag done (move)");
                            should_stop_interaction = true;
                        }
                    }
                }
                if should_stop_interaction {
                    self.interaction = Interaction::Nothing;
                }
            }
            Event::MouseUp(mouse) => {
                if mouse.left {
                    match self.interaction.take() {
                        Interaction::Nothing => {}
                        Interaction::DraggingBounds(drag) => {
                            drag.finish(grid);
                        }
                        Interaction::DraggingChip(drag) => {
                            drag.drop_onto_board(grid);
                            audio.play_sound(Sound::DropChip);
                        }
                        Interaction::SelectingRect(drag) => {
                            if !drag.rect.is_empty() {
                                self.interaction =
                                    Interaction::RectSelected(drag.rect);
                            }
                        }
                        Interaction::RectSelected(rect) => {
                            self.interaction = Interaction::RectSelected(rect);
                        }
                        Interaction::DraggingSelection(drag) => {
                            let rect = drag.finish(grid);
                            self.interaction = Interaction::RectSelected(rect);
                        }
                        Interaction::DraggingWires(drag) => {
                            drag.finish(grid);
                        }
                    }
                }
            }
            Event::Multitouch(touch) => {
                self.tooltip.stop_hover_all();
                self.zoom_by(touch.scale);
            }
            Event::Scroll(scroll) => {
                self.tooltip.stop_hover_all();
                self.scroll_by_screen_dist(scroll.delta.x, scroll.delta.y);
            }
            Event::Unfocus => {
                self.tooltip.stop_hover_all();
            }
            _ => {}
        }
        return None;
    }

    pub fn grab_from_parts_tray(&mut self, ctype: ChipType,
                                screen_pt: Point2<i32>) {
        let size = ctype.size();
        let start = 0.5 * Point2::new(size.width, size.height).as_f32();
        let mut drag =
            ChipDrag::new(ctype, Orientation::default(), None, start);
        drag.move_to(self.screen_pt_to_grid_pt(screen_pt));
        self.interaction = Interaction::DraggingChip(drag);
    }

    pub fn drop_into_parts_tray(&mut self, grid: &mut EditGrid) {
        match self.interaction.take() {
            Interaction::DraggingChip(drag) => {
                drag.drop_into_parts_tray(grid);
            }
            other => self.interaction = other,
        }
    }

    fn cut_region(&mut self, grid: &mut EditGrid, selected_rect: CoordsRect) {
        let mut changes = Vec::<GridChange>::new();
        let mut needs_mass_toggle = false;
        let mut wires = HashMap::<(Coords, Direction), WireShape>::new();
        let mut extra_stubs = HashSet::<(Coords, Direction)>::new();
        for coords in selected_rect {
            if let Some((chip_coords, ctype, orient)) = grid.chip_at(coords) {
                if chip_coords == coords {
                    let chip_size = orient * ctype.size();
                    let chip_rect = Rect::with_size(chip_coords, chip_size);
                    if selected_rect.contains_rect(chip_rect) {
                        changes.push(GridChange::ToggleChip(chip_coords,
                                                            orient,
                                                            ctype));
                    }
                }
            }
            for dir in Direction::all() {
                if let Some(shape) = grid.wire_shape_at(coords, dir) {
                    wires.insert((coords, dir), shape);
                    let coords2 = coords + dir;
                    let dir2 = -dir;
                    let on_edge = !selected_rect.contains_point(coords2);
                    needs_mass_toggle = needs_mass_toggle || !on_edge ||
                        shape != WireShape::Stub;
                    if on_edge &&
                        grid.wire_shape_at(coords2, dir2) ==
                            Some(WireShape::Stub)
                    {
                        extra_stubs.insert((coords2, dir2));
                    }
                }
            }
        }
        if needs_mass_toggle {
            changes.push(GridChange::MassToggleWires(selected_rect, wires));
        }
        for (coords, dir) in extra_stubs {
            changes.push(GridChange::ToggleStubWire(coords, dir));
        }
        grid.do_mutate(changes);
    }

    fn zoom_by(&mut self, factor: f32) {
        self.zoom = ZOOM_MIN.max(self.zoom * factor).min(ZOOM_MAX);
    }

    fn scroll_by_screen_dist(&mut self, x: i32, y: i32) {
        self.scroll += (vec2(x, y).as_f32() / self.zoom).as_i32_round();
    }

    fn screen_pt_to_grid_pt(&self, screen_pt: Point2<i32>) -> Point2<f32> {
        (((screen_pt.as_f32() - vec2(0.5 * self.width, 0.5 * self.height)) /
              self.zoom)
             .as_i32_round() + self.scroll)
            .as_f32() / (GRID_CELL_SIZE as f32)
    }

    fn bounds_octant_for_screen_pt(&self, screen_pt: Point2<i32>,
                                   grid: &EditGrid)
                                   -> Option<Octant> {
        let grid_pt = self.screen_pt_to_grid_pt(screen_pt);
        let inner = grid.bounds().as_f32();
        if inner.contains_point(grid_pt) {
            return None;
        }
        let outer = inner.expand(BOUNDS_MARGIN);
        if !outer.contains_point(grid_pt) {
            return None;
        }
        let at_top = grid_pt.y < inner.y;
        let at_bottom = grid_pt.y >= inner.bottom();
        if grid_pt.x < inner.x {
            if at_top {
                Some(Octant::TopLeft)
            } else if at_bottom {
                Some(Octant::BottomLeft)
            } else {
                Some(Octant::Left)
            }
        } else if grid_pt.x >= inner.right() {
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

    fn coords_for_screen_pt(&self, screen_pt: Point2<i32>) -> Coords {
        self.screen_pt_to_grid_pt(screen_pt).as_i32_floor()
    }

    fn tooltip_tag_for_grid_pt(&self, grid: &EditGrid, grid_pt: Point2<f32>)
                               -> Option<GridTooltipTag> {
        let coords: Coords = grid_pt.as_i32_floor();
        if let Some((coords, ctype, _)) = grid.chip_at(coords) {
            return Some(GridTooltipTag::Chip(coords, ctype));
        }
        if let Some((index, _)) = grid.interface_at(coords) {
            return Some(GridTooltipTag::Interface(index));
        }
        // TODO: Figure out if we're actually visibly over the wire shape,
        //   rather than just treating each wire shape as a triangle.
        let x = grid_pt.x - (coords.x as f32);
        let y = grid_pt.y - (coords.y as f32);
        let dir = if x > y {
            if x > (1.0 - y) {
                Direction::East
            } else {
                Direction::North
            }
        } else {
            if y > (1.0 - x) {
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

enum Interaction {
    Nothing,
    DraggingBounds(BoundsDrag),
    DraggingChip(ChipDrag),
    SelectingRect(SelectingDrag),
    RectSelected(CoordsRect),
    DraggingSelection(SelectionDrag),
    DraggingWires(WireDrag),
}

impl Interaction {
    fn take(&mut self) -> Interaction {
        mem::replace(self, Interaction::Nothing)
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
    drag_start_grid_pt: Point2<f32>,
    drag_current_grid_pt: Point2<f32>,
    bounds: CoordsRect,
    acceptable: bool,
}

impl BoundsDrag {
    pub fn new(octant: Octant, start_grid_pt: Point2<f32>,
               grid: &mut EditGrid)
               -> BoundsDrag {
        BoundsDrag {
            octant,
            drag_start_grid_pt: start_grid_pt,
            drag_current_grid_pt: start_grid_pt,
            bounds: grid.bounds(),
            acceptable: true,
        }
    }

    pub fn move_to(&mut self, grid_pt: Point2<f32>, grid: &EditGrid) {
        self.drag_current_grid_pt = grid_pt;
        let delta: CoordsDelta = (self.drag_current_grid_pt -
                                      self.drag_start_grid_pt)
            .as_i32_round();
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
            grid.do_mutate(
                vec![GridChange::SwapBounds(old_bounds, self.bounds)],
            );
        }
    }
}

//===========================================================================//

struct ChipDrag {
    chip_type: ChipType,
    old_orient: Orientation,
    old_coords: Option<Coords>,
    drag_start: Point2<f32>, // grid space
    drag_current: Point2<f32>, // grid space
    reorient: Orientation,
}

impl ChipDrag {
    pub fn new(chip_type: ChipType, old_orient: Orientation,
               old_coords: Option<Coords>, drag_start: Point2<f32>)
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

    pub fn chip_topleft(&self) -> Point2<f32> {
        let old_coords = if let Some(coords) = self.old_coords {
            coords
        } else {
            Point2::new(0, 0)
        };
        old_coords.as_f32() + (self.drag_current - self.drag_start)
    }

    pub fn flip_horz(&mut self) { self.reorient = self.reorient.flip_horz(); }

    pub fn flip_vert(&mut self) { self.reorient = self.reorient.flip_vert(); }

    pub fn rotate_cw(&mut self) { self.reorient = self.reorient.rotate_cw(); }

    pub fn rotate_ccw(&mut self) {
        self.reorient = self.reorient.rotate_ccw();
    }

    pub fn move_to(&mut self, grid_pt: Point2<f32>) {
        self.drag_current = grid_pt;
    }

    pub fn drop_onto_board(self, grid: &mut EditGrid) {
        let old_size = self.old_orient * self.chip_type.size();
        let old_rect =
            self.old_coords
                .map(|coords| CoordsRect::with_size(coords, old_size));
        let new_coords: Coords = self.chip_topleft().as_i32_round();
        let new_size = self.reorient * old_size;
        let new_rect = CoordsRect::with_size(new_coords, new_size);
        if grid.can_move_chip(old_rect, new_rect) {
            let mut changes = Vec::<GridChange>::new();
            if let Some(old_coords) = self.old_coords {
                changes.push(GridChange::ToggleChip(old_coords,
                                                    self.old_orient,
                                                    self.chip_type));
            }
            let mut needs_mass_toggle = false;
            let mut wires = HashMap::<(Coords, Direction), WireShape>::new();
            for coords in new_rect {
                for dir in Direction::all() {
                    if let Some(shape) = grid.wire_shape_at(coords, dir) {
                        wires.insert((coords, dir), shape);
                        needs_mass_toggle = needs_mass_toggle ||
                            shape != WireShape::Stub ||
                            new_rect.contains_point(coords + dir);
                    }
                }
            }
            if needs_mass_toggle {
                changes.push(GridChange::MassToggleWires(new_rect, wires));
            }
            changes.push(GridChange::ToggleChip(new_coords,
                                                self.reorient *
                                                    self.old_orient,
                                                self.chip_type));
            grid.do_mutate(changes);
        }
    }

    pub fn drop_into_parts_tray(self, grid: &mut EditGrid) {
        if let Some(old_coords) = self.old_coords {
            grid.do_mutate(vec![
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

struct SelectingDrag {
    bounds: CoordsRect,
    start: Coords,
    rect: CoordsRect,
}

impl SelectingDrag {
    fn is_near_vertex(grid_pt: Point2<f32>, bounds: CoordsRect) -> bool {
        let coords = grid_pt.as_i32_round();
        let expanded =
            Rect::new(bounds.x, bounds.y, bounds.width + 1, bounds.height + 1);
        expanded.contains_point(coords) &&
            grid_pt.distance(coords.as_f32()) <= SELECTING_VERTEX_MAX_DIST
    }

    fn new(bounds: CoordsRect, start: Coords) -> SelectingDrag {
        let rect = Rect::new(start.x, start.y, 0, 0);
        SelectingDrag {
            bounds,
            start,
            rect,
        }
    }

    fn move_to(&mut self, grid_pt: Point2<f32>) {
        let coords = grid_pt.as_i32_round();
        self.rect = Rect::new(self.start.x.min(coords.x),
                              self.start.y.min(coords.y),
                              (self.start.x - coords.x).abs(),
                              (self.start.y - coords.y).abs());
        self.rect = self.rect.intersection(self.bounds);
    }
}

//===========================================================================//

struct SelectionDrag {
    selected_rect: CoordsRect,
    start_grid_pt: Point2<f32>,
    current_grid_pt: Point2<f32>,
    reorient: Orientation,
}

impl SelectionDrag {
    fn new(selected_rect: CoordsRect, grid_pt: Point2<f32>) -> SelectionDrag {
        SelectionDrag {
            selected_rect,
            start_grid_pt: grid_pt,
            current_grid_pt: grid_pt,
            reorient: Orientation::default(),
        }
    }

    fn reoriented_selected_rect(&self) -> CoordsRect {
        Rect::with_size(self.selected_rect.top_left(),
                        self.reorient * self.selected_rect.size())
    }

    fn delta(&self) -> Vector2<f32> {
        self.current_grid_pt - self.start_grid_pt
    }

    fn flip_horz(&mut self) { self.reorient = self.reorient.flip_horz(); }

    fn flip_vert(&mut self) { self.reorient = self.reorient.flip_vert(); }

    fn rotate_cw(&mut self) { self.reorient = self.reorient.rotate_cw(); }

    fn rotate_ccw(&mut self) { self.reorient = self.reorient.rotate_ccw(); }

    fn move_to(&mut self, grid_pt: Point2<f32>) {
        self.current_grid_pt = grid_pt;
    }

    fn finish(self, grid: &mut EditGrid) -> CoordsRect {
        let drag_delta: CoordsDelta = self.delta().as_i32_round();
        let new_selected_rect: CoordsRect = self.reoriented_selected_rect() +
            drag_delta;
        if !grid.bounds().contains_rect(new_selected_rect) {
            return self.selected_rect;
        }
        let mut chips = Vec::<(Coords, ChipType, Orientation)>::new();
        for coords in self.selected_rect {
            if let Some((chip_coords, ctype, orient)) = grid.chip_at(coords) {
                if chip_coords == coords {
                    let chip_size = orient * ctype.size();
                    let chip_rect = Rect::with_size(chip_coords, chip_size);
                    if self.selected_rect.contains_rect(chip_rect) {
                        chips.push((chip_coords, ctype, orient));
                    }
                }
            }
        }
        let mut changes = Vec::<GridChange>::new();

        // Remove old chips:
        for &(coords, ctype, orient) in chips.iter() {
            changes.push(GridChange::ToggleChip(coords, orient, ctype));
        }

        // TODO: remove old wires

        // TODO: place new wires

        // Place new chips:
        for &(old_coords, ctype, old_orient) in chips.iter() {
            let new_coords =
                self.reorient
                    .transform_in_rect(old_coords, self.selected_rect) +
                    drag_delta;
            let new_orient = self.reorient * old_orient;
            // TODO: Remove wires from under new chip location
            changes
                .push(GridChange::ToggleChip(new_coords, new_orient, ctype));
        }

        if grid.try_mutate(changes) {
            return new_selected_rect;
        } else {
            return self.selected_rect;
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
// TODO: enforce wires can't be created under chips
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
                grid.do_mutate(vec![GridChange::ToggleStubWire(coords, dir)]);
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
                grid.do_mutate(vec![GridChange::ToggleStubWire(coords, dir)]);
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
                grid.do_mutate(vec![GridChange::ToggleCrossWire(coords)]);
                self.changed = true;
            }
            (_, _) => {}
        }
    }

    fn try_straight(&mut self, coords: Coords, dir: Direction,
                    grid: &mut EditGrid)
                    -> bool {
        let mut changes = Vec::<GridChange>::new();
        if grid.wire_shape_at(coords, dir).is_none() {
            changes.push(GridChange::ToggleStubWire(coords, dir));
        }
        if grid.wire_shape_at(coords, -dir).is_none() {
            changes.push(GridChange::ToggleStubWire(coords, -dir));
        }
        changes.push(GridChange::ToggleCenterWire(coords, dir, -dir));
        if grid.wire_shape_at(coords, dir) == Some(WireShape::Straight) &&
            grid.wire_shape_at(coords + dir, -dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::ToggleStubWire(coords, dir));
        }
        if grid.wire_shape_at(coords, -dir) == Some(WireShape::Straight) &&
            grid.wire_shape_at(coords - dir, dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::ToggleStubWire(coords, -dir));
        }
        let success = grid.try_mutate(changes);
        self.changed |= success;
        success
    }

    fn try_turn_left(&mut self, coords: Coords, dir: Direction,
                     grid: &mut EditGrid)
                     -> bool {
        let dir2 = dir.rotate_cw();
        let mut changes = Vec::<GridChange>::new();
        if grid.wire_shape_at(coords, dir).is_none() {
            changes.push(GridChange::ToggleStubWire(coords, dir));
        }
        if grid.wire_shape_at(coords, dir2).is_none() {
            changes.push(GridChange::ToggleStubWire(coords, dir2));
        }
        changes.push(GridChange::ToggleCenterWire(coords, dir, dir2));
        if grid.wire_shape_at(coords, dir) == Some(WireShape::TurnLeft) &&
            grid.wire_shape_at(coords + dir, -dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::ToggleStubWire(coords, dir));
        }
        if grid.wire_shape_at(coords, dir2) == Some(WireShape::TurnRight) &&
            grid.wire_shape_at(coords + dir2, -dir2) == Some(WireShape::Stub)
        {
            changes.push(GridChange::ToggleStubWire(coords, dir2));
        }
        let success = grid.try_mutate(changes);
        self.changed |= success;
        success
    }

    fn try_split(&mut self, coords: Coords, dir: Direction,
                 grid: &mut EditGrid)
                 -> bool {
        let mut changes = Vec::<GridChange>::new();
        let shape = grid.wire_shape_at(coords, dir);
        if shape.is_none() {
            changes.push(GridChange::ToggleStubWire(coords, dir));
        }
        changes.push(GridChange::ToggleSplitWire(coords, dir));
        if shape.is_some() && shape != Some(WireShape::Stub) &&
            grid.wire_shape_at(coords + dir, -dir) == Some(WireShape::Stub)
        {
            changes.push(GridChange::ToggleStubWire(coords, dir));
        }
        let success = grid.try_mutate(changes);
        self.changed |= success;
        success
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Zone {
    Center(Coords),
    East(Coords),
    South(Coords),
}

impl Zone {
    fn from_grid_pt(grid_pt: Point2<f32>) -> Zone {
        let coords: Coords = grid_pt.as_i32_floor();
        let x = grid_pt.x - (coords.x as f32) - 0.5;
        let y = grid_pt.y - (coords.y as f32) - 0.5;
        if x.abs() <= ZONE_CENTER_SEMI_SIZE &&
            y.abs() <= ZONE_CENTER_SEMI_SIZE
        {
            Zone::Center(coords)
        } else if x.abs() > y.abs() {
            Zone::East(if x > 0.0 {
                           coords
                       } else {
                           coords + Direction::West
                       })
        } else {
            Zone::South(if y > 0.0 {
                            coords
                        } else {
                            coords + Direction::North
                        })
        }
    }
}

//===========================================================================//

#[derive(Eq, PartialEq)]
enum GridTooltipTag {
    Chip(Coords, ChipType),
    Interface(usize),
    Wire(usize),
}

//===========================================================================//
