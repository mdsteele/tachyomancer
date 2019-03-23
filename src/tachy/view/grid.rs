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

use super::bounds::{BOUNDS_MARGIN, BoundsDrag, BoundsHandle};
use super::chip::ChipModel;
use super::chipdrag::ChipDrag;
use super::select::{self, SelectingDrag, Selection, SelectionDrag};
use super::tooltip::Tooltip;
use super::wire::WireModel;
use super::wiredrag::{WireDrag, Zone};
use cgmath::{self, Matrix4, Point2, Vector2, vec2, vec4};
use std::mem;
use tachy::geom::{AsFloat, AsInt, Color4, Coords, CoordsRect, Direction,
                  MatrixExt, Orientation, Rect, RectSize};
use tachy::gui::{AudioQueue, Clipboard, Cursor, Event, Keycode, NextCursor,
                 Resources, Sound};
use tachy::save::{ChipType, Hotkey, Prefs, WireShape};
use tachy::state::{EditGrid, GridChange, WireColor};

//===========================================================================//

// The size, in screen pixels, of a grid cell at 1x zoom:
const GRID_CELL_SIZE: i32 = 64;

// Number of screen pixels to scroll by when pressing a scroll hotkey:
const SCROLL_PER_KEYDOWN: i32 = 40;

const SELECTION_BOX_COLOR1: Color4 = Color4::CYAN5.with_alpha(0.75);
const SELECTION_BOX_COLOR2: Color4 = Color4::CYAN4.with_alpha(0.75);
const SELECTION_BOX_COLOR3: Color4 = Color4::CYAN4.with_alpha(0.1);

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
        let (bounds, is_acceptable) = match self.interaction {
            Interaction::DraggingBounds(ref drag) => {
                (drag.bounds(), drag.is_acceptable())
            }
            _ => (grid.bounds(), true),
        };
        let x = (bounds.x * GRID_CELL_SIZE) as f32;
        let y = (bounds.y * GRID_CELL_SIZE) as f32;
        let width = (bounds.width * GRID_CELL_SIZE) as f32;
        let height = (bounds.height * GRID_CELL_SIZE) as f32;
        let thick = BOUNDS_MARGIN * (GRID_CELL_SIZE as f32);
        let color = if is_acceptable {
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
            Interaction::DraggingBounds(ref drag) => drag.bounds(),
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
            Interaction::DraggingChip(ref drag) => drag.old_coords(),
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
                                        drag.selected_rect(),
                                        vec2(0.0, 0.0));
            }
            Interaction::RectSelected(rect) => {
                self.draw_selection_box(resources,
                                        &matrix,
                                        rect,
                                        vec2(0.0, 0.0));
            }
            Interaction::DraggingSelection(ref drag) => {
                // TODO: Draw selected chips/wires
                let rect = Rect::with_size(Coords::new(0, 0),
                                           drag.selection_size());
                let offset = drag.top_left_grid_pt() - Point2::new(0.0, 0.0);
                // TODO: color box red if we cannot drop the selection here
                self.draw_selection_box(resources, &matrix, rect, offset);
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
                                      drag.chip_type(),
                                      drag.new_orient(),
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

    pub fn request_interaction_cursor(&self, event: &Event,
                                      next_cursor: &mut NextCursor) {
        match event {
            Event::MouseUp(mouse) if mouse.left => return,
            _ => {}
        }
        match self.interaction {
            Interaction::DraggingBounds(ref drag) => {
                drag.request_cursor(next_cursor);
            }
            Interaction::DraggingChip(_) |
            Interaction::DraggingSelection(_) => {
                next_cursor.request(Cursor::HandClosed);
            }
            Interaction::DraggingWires(_) => {
                next_cursor.request(Cursor::Wire);
            }
            Interaction::SelectingRect(_) => {
                next_cursor.request(Cursor::Crosshair);
            }
            Interaction::Nothing |
            Interaction::RectSelected(_) => {}
        }
    }

    fn cursor_for_grid_pt(&self, grid_pt: Point2<f32>, grid: &EditGrid)
                          -> Cursor {
        let coords = grid_pt.as_i32_floor();
        match self.interaction {
            Interaction::Nothing => {
                // TODO: Check whether evaluation is active
                if SelectingDrag::is_near_vertex(grid_pt, grid.bounds()) {
                    return Cursor::Crosshair;
                } else if let Some(handle) =
                    BoundsHandle::for_grid_pt(grid_pt, grid)
                {
                    return handle.cursor();
                } else if grid.chip_at(coords).is_some() {
                    return Cursor::HandOpen;
                } else if grid.bounds().contains_point(coords) {
                    return Cursor::Wire;
                }
            }
            Interaction::RectSelected(rect) => {
                if rect.contains_point(grid_pt.as_i32_floor()) {
                    return Cursor::HandOpen;
                }
            }
            _ => {}
        }
        return Cursor::default();
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
                    prefs: &Prefs, audio: &mut AudioQueue,
                    clipboard: &Clipboard, next_cursor: &mut NextCursor)
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
                        Keycode::C => {
                            if let Interaction::RectSelected(rect) =
                                self.interaction
                            {
                                select::copy(grid, rect, clipboard);
                            }
                        }
                        Keycode::V => {
                            if let Some(selection) =
                                Selection::from_clipboard(clipboard)
                            {
                                self.cancel_interaction(grid);
                                let size = selection.size().as_f32();
                                let rel = vec2(size.width, size.height) * 0.5;
                                let grid_pt =
                                    self.screen_pt_to_grid_pt(key.mouse_pt);
                                let drag = SelectionDrag::new(selection,
                                                              rel,
                                                              grid_pt,
                                                              None);
                                self.interaction =
                                    Interaction::DraggingSelection(drag);
                            }
                        }
                        Keycode::X => {
                            if let Interaction::RectSelected(rect) =
                                self.interaction
                            {
                                select::cut(grid, rect, clipboard);
                                self.interaction = Interaction::Nothing;
                            }
                        }
                        Keycode::Z if key.shift => {
                            self.cancel_interaction(grid);
                            grid.redo();
                        }
                        Keycode::Z => {
                            if !self.cancel_interaction(grid) {
                                grid.undo();
                            }
                        }
                        _ => {}
                    }
                } else if let Some(hotkey) = prefs.hotkey_for_code(key.code) {
                    self.on_hotkey(hotkey);
                }
            }
            Event::MouseDown(mouse) => {
                let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                self.tooltip.stop_hover_all();
                match self.interaction.take() {
                    Interaction::RectSelected(rect) => {
                        if mouse.left &&
                            rect.contains_point(grid_pt.as_i32_floor())
                        {
                            let selection = select::cut_provisionally(grid,
                                                                      rect);
                            let grab_rel = grid_pt - rect.top_left().as_f32();
                            let drag = SelectionDrag::new(selection,
                                                          grab_rel,
                                                          grid_pt,
                                                          Some(rect));
                            self.interaction =
                                Interaction::DraggingSelection(drag);
                            return None;
                        } else {
                            self.interaction = Interaction::RectSelected(rect);
                        }
                    }
                    Interaction::DraggingSelection(drag) => {
                        if let Some(rect) = drag.finish(grid) {
                            self.interaction = Interaction::RectSelected(rect);
                        }
                        return None;
                    }
                    _ => {}
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
                    // TODO: Don't allow starting selection on a vertex that is
                    //   e.g. the center of a 2x2 chip.
                    if SelectingDrag::is_near_vertex(grid_pt, grid.bounds()) {
                        let drag = SelectingDrag::new(grid.bounds(),
                                                      grid_pt.as_i32_round());
                        self.interaction = Interaction::SelectingRect(drag);
                        return None;
                    }
                    let mouse_coords = grid_pt.as_i32_floor();
                    if !grid.bounds().contains_point(mouse_coords) {
                        if let Some(handle) =
                            BoundsHandle::for_grid_pt(grid_pt, grid)
                        {
                            let drag = BoundsDrag::new(handle, grid_pt, grid);
                            self.interaction =
                                Interaction::DraggingBounds(drag);
                        }
                    } else if let Some((coords, ctype, orient)) =
                        grid.chip_at(mouse_coords)
                    {
                        // TODO: If mouse is within chip cell but near edge of
                        //   chip, allow for wire dragging.
                        let change =
                            GridChange::RemoveChip(coords, ctype, orient);
                        if grid.try_mutate_provisionally(vec![change]) {
                            let drag = ChipDrag::new(ctype,
                                                     orient,
                                                     Some(coords),
                                                     grid_pt);
                            self.interaction = Interaction::DraggingChip(drag);
                            audio.play_sound(Sound::GrabChip);
                        }
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
                next_cursor.request(self.cursor_for_grid_pt(grid_pt, grid));
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
                            let rect = drag.selected_rect();
                            if !rect.is_empty() {
                                self.interaction =
                                    Interaction::RectSelected(rect);
                            }
                        }
                        Interaction::RectSelected(rect) => {
                            self.interaction = Interaction::RectSelected(rect);
                        }
                        Interaction::DraggingSelection(drag) => {
                            if let Some(rect) = drag.finish(grid) {
                                self.interaction =
                                    Interaction::RectSelected(rect);
                            }
                        }
                        Interaction::DraggingWires(drag) => {
                            drag.finish(grid);
                        }
                    }
                    let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                    let cursor = self.cursor_for_grid_pt(grid_pt, grid);
                    next_cursor.request(cursor);
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

    fn cancel_interaction(&mut self, grid: &mut EditGrid) -> bool {
        match self.interaction.take() {
            Interaction::DraggingChip(drag) => drag.cancel(grid),
            Interaction::DraggingSelection(drag) => drag.cancel(grid),
            _ => false,
        }
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

#[derive(Eq, PartialEq)]
enum GridTooltipTag {
    Chip(Coords, ChipType),
    Interface(usize),
    Wire(usize),
}

//===========================================================================//
