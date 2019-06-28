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
use super::chipdrag::ChipDrag;
use super::select::{self, SelectingDrag, Selection, SelectionDrag};
use super::super::chip::{CHIP_MARGIN, ChipModel, chip_grid_rect,
                         interface_grid_rect};
use super::super::tooltip::Tooltip;
use super::super::wire::WireModel;
use super::tooltip::GridTooltipTag;
use super::tutorial::TutorialBubble;
use super::wiredrag::WireDrag;
use cgmath::{self, Matrix4, Point2, Vector2, vec2, vec4};
use std::mem;
use tachy::geom::{AsFloat, AsInt, Color3, Coords, CoordsRect, Direction,
                  MatrixExt, Orientation, Rect, RectSize};
use tachy::gui::{ClockEventData, Cursor, Event, Keycode, NextCursor,
                 Resources, Sound, Ui};
use tachy::save::{ChipType, Hotkey, Prefs, WireShape};
use tachy::state::{EditGrid, GridChange, WireColor};

//===========================================================================//

// The size, in screen pixels, of a grid cell at 1x zoom:
const GRID_CELL_SIZE: i32 = 64;

// How far we scroll per second while holding down a scroll hotkey, in grid
// cells, at max zoom:
const SCROLL_GRID_CELLS_PER_SECOND: f64 = 12.0;

// The minimum zoom multiplier (i.e. how far zoomed out you can be):
const ZOOM_MIN: f32 = 0.25;
// The maximum zoom multiplier (i.e. how far zoomed in you can be):
const ZOOM_MAX: f32 = 1.0;
// How much to multiply/divide the zoom by when pressing a zoom hotkey:
const ZOOM_PER_KEYDOWN: f32 = 1.415; // slightly more than sqrt(2)

//===========================================================================//

pub enum EditGridAction {
    EditConst(Coords, u16),
}

//===========================================================================//

pub struct EditGridView {
    size: RectSize<f32>,
    scroll: Vector2<i32>,
    zoom: f32,
    wire_model: WireModel,
    interaction: Interaction,
    tutorial_bubbles: Vec<(Direction, TutorialBubble)>,
    hover_wire: Option<usize>,
    tooltip: Tooltip<GridTooltipTag>,
}

impl EditGridView {
    pub fn new(window_size: RectSize<i32>,
               tutorial_bubbles: Vec<(Direction, TutorialBubble)>)
               -> EditGridView {
        EditGridView {
            size: window_size.as_f32(),
            scroll: Vector2::new(0, 0),
            zoom: ZOOM_MAX,
            wire_model: WireModel::new(),
            interaction: Interaction::Nothing,
            tutorial_bubbles,
            hover_wire: None,
            tooltip: Tooltip::new(window_size),
        }
    }

    fn draw_background_grid(&self, resources: &Resources) {
        let matrix = cgmath::ortho(0.0, 1.0, 1.0, 0.0, -1.0, 1.0);
        let size = self.size * self.zoom.recip();
        let pixel_rect = vec4((self.scroll.x as f32) - 0.5 * size.width,
                              (self.scroll.y as f32) - 0.5 * size.height,
                              size.width,
                              size.height);
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
            Color3::PURPLE2
        } else {
            Color3::new(1.0, 0.0, 0.0)
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

    fn draw_tutorial_bubbles(&self, resources: &Resources, grid: &EditGrid) {
        let matrix = self.unzoomed_matrix();
        let bounds =
            if let Interaction::DraggingBounds(ref drag) = self.interaction {
                drag.bounds()
            } else {
                grid.bounds()
            };
        let bounds = bounds.as_f32().expand(BOUNDS_MARGIN) *
            ((GRID_CELL_SIZE as f32) * self.zoom);
        let margin: i32 = 8;
        for &(dir, ref bubble) in self.tutorial_bubbles.iter() {
            let topleft = match dir {
                Direction::East => {
                    Point2::new((bounds.right().round() as i32) + margin,
                                ((bounds.y + 0.5 * bounds.height).round() as
                                     i32) -
                                    bubble.height() / 2)
                }
                Direction::South => {
                    Point2::new(((bounds.x + 0.5 * bounds.width).round() as
                                     i32) -
                                    bubble.width() / 2,
                                (bounds.bottom().round() as i32) + margin)
                }
                Direction::West => {
                    Point2::new((bounds.x.round() as i32) - margin -
                                    bubble.width(),
                                ((bounds.y + 0.5 * bounds.height).round() as
                                     i32) -
                                    bubble.height() / 2)
                }
                Direction::North => {
                    Point2::new(((bounds.x + 0.5 * bounds.width).round() as
                                     i32) -
                                    bubble.width() / 2,
                                (bounds.y.round() as i32) - margin -
                                    bubble.height())
                }
            };
            bubble.draw(resources, &matrix, topleft);
        }
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
            ChipModel::draw_interface(resources, &mat, interface);
        }
    }

    pub fn draw_board(&self, resources: &Resources, grid: &EditGrid) {
        self.draw_background_grid(resources);
        self.draw_bounds(resources, grid);
        self.draw_tutorial_bubbles(resources, grid);

        // Draw wires:
        let matrix = self.vp_matrix();
        for (coords, dir, shape, size, color, has_error) in
            grid.wire_fragments()
        {
            let selected = grid.wire_index_at(coords, dir) == self.hover_wire;
            // TODO: When a wire with an error is selected, we should hilight
            //   the causes of the error (e.g. the two sender ports, or the
            //   wire loop, or whatever).
            let color = if has_error {
                WireColor::Ambiguous
            } else {
                color
            };
            match (shape, dir) {
                (WireShape::Stub, _) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model
                        .draw_stub(resources, &matrix, color, size, selected);
                }
                (WireShape::Straight, Direction::East) |
                (WireShape::Straight, Direction::North) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model.draw_straight(resources,
                                                  &matrix,
                                                  color,
                                                  size,
                                                  selected);
                }
                (WireShape::TurnLeft, _) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model
                        .draw_turn(resources, &matrix, color, size, selected);
                }
                (WireShape::SplitTee, _) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model
                        .draw_tee(resources, &matrix, color, size, selected);
                }
                (WireShape::Cross, Direction::East) => {
                    let matrix = coords_matrix(&matrix, coords, dir);
                    self.wire_model
                        .draw_cross(resources, &matrix, color, size, selected);
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
            ChipModel::draw_chip(resources,
                                 &mat,
                                 ctype,
                                 orient,
                                 Some((coords, grid)));
        }

        // Draw selection box (if any):
        match self.interaction {
            Interaction::SelectingRect(ref drag) => {
                drag.draw_box(resources,
                              &self.unzoomed_matrix(),
                              (GRID_CELL_SIZE as f32) * self.zoom);
            }
            Interaction::RectSelected(rect) => {
                Selection::draw_box(resources,
                                    &self.unzoomed_matrix(),
                                    rect,
                                    (GRID_CELL_SIZE as f32) * self.zoom);
            }
            Interaction::DraggingSelection(ref drag) => {
                drag.draw_selection(resources,
                                    &self.unzoomed_matrix(),
                                    &self.wire_model,
                                    (GRID_CELL_SIZE as f32) * self.zoom);
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
            ChipModel::draw_chip(resources,
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
        cgmath::ortho(0.0, self.size.width, self.size.height, 0.0, -1.0, 1.0) *
            Matrix4::trans2(0.5 * self.size.width, 0.5 * self.size.height) *
            Matrix4::from_scale(self.zoom) *
            Matrix4::trans2(-self.scroll.x as f32, -self.scroll.y as f32)
    }

    fn unzoomed_matrix(&self) -> Matrix4<f32> {
        cgmath::ortho(0.0, self.size.width, self.size.height, 0.0, -1.0, 1.0) *
            Matrix4::trans2(0.5 * self.size.width, 0.5 * self.size.height) *
            Matrix4::trans2((-self.scroll.x as f32) * self.zoom,
                            (-self.scroll.y as f32) * self.zoom)
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
                if grid.eval().is_some() {
                    if let Some((_, ChipType::Button, _)) =
                        grid.chip_at(coords)
                    {
                        return Cursor::HandPointing;
                    }
                    return Cursor::NoSign;
                }
                if let Some((chip_coords, ctype, orient)) =
                    grid.chip_at(coords)
                {
                    let chip_rect = chip_grid_rect(chip_coords, ctype, orient);
                    if chip_rect.contains_point(grid_pt) {
                        return Cursor::HandOpen;
                    }
                }
                if let Some((iface_coords, _, iface)) =
                    grid.interface_at(coords)
                {
                    if interface_grid_rect(iface_coords, iface)
                        .contains_point(grid_pt)
                    {
                        return Cursor::HandOpen;
                    }
                }
                if SelectingDrag::is_near_vertex(grid_pt, grid.bounds()) {
                    return Cursor::Crosshair;
                }
                if grid.bounds()
                    .as_f32()
                    .expand(CHIP_MARGIN)
                    .contains_point(grid_pt)
                {
                    return Cursor::Wire;
                }
                if let Some(bh) = BoundsHandle::for_grid_pt(grid_pt, grid) {
                    return bh.cursor();
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

    fn on_hotkey(&mut self, hotkey: Hotkey, ui: &mut Ui) {
        if hotkey == Hotkey::ZoomIn {
            self.zoom_by(ZOOM_PER_KEYDOWN, ui);
        } else if hotkey == Hotkey::ZoomOut {
            self.zoom_by(1.0 / ZOOM_PER_KEYDOWN, ui);
        } else {
            match self.interaction {
                Interaction::DraggingChip(ref mut drag) => {
                    match hotkey {
                        Hotkey::FlipHorz => drag.flip_horz(ui),
                        Hotkey::FlipVert => drag.flip_vert(ui),
                        Hotkey::RotateCcw => drag.rotate_ccw(ui),
                        Hotkey::RotateCw => drag.rotate_cw(ui),
                        _ => {}
                    }
                }
                Interaction::DraggingSelection(ref mut drag) => {
                    match hotkey {
                        Hotkey::FlipHorz => drag.flip_horz(ui),
                        Hotkey::FlipVert => drag.flip_vert(ui),
                        Hotkey::RotateCcw => drag.rotate_ccw(ui),
                        Hotkey::RotateCw => drag.rotate_cw(ui),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui,
                    grid: &mut EditGrid, prefs: &Prefs)
                    -> Option<EditGridAction> {
        match event {
            Event::ClockTick(tick) => {
                self.tooltip
                    .tick(tick, ui, prefs, |tag| tag.tooltip_format(grid));
                // Scroll if we're holding down any scroll key(s):
                let (left, right, up, down) = {
                    let keyboard = ui.keyboard();
                    (keyboard.is_held(prefs.hotkey_code(Hotkey::ScrollLeft)),
                     keyboard.is_held(prefs.hotkey_code(Hotkey::ScrollRight)),
                     keyboard.is_held(prefs.hotkey_code(Hotkey::ScrollUp)),
                     keyboard.is_held(prefs.hotkey_code(Hotkey::ScrollDown)))
                };
                let dist = ((SCROLL_GRID_CELLS_PER_SECOND * tick.elapsed) *
                                (GRID_CELL_SIZE as f64))
                    .round() as i32;
                if left && !right {
                    self.scroll_by_screen_dist(-dist, 0, ui);
                } else if right && !left {
                    self.scroll_by_screen_dist(dist, 0, ui);
                }
                if up && !down {
                    self.scroll_by_screen_dist(0, -dist, ui);
                } else if down && !up {
                    self.scroll_by_screen_dist(0, dist, ui);
                }
                // Spring back to scroll bounds:
                let expand = (self.size * (0.25 / self.zoom)).as_i32_round();
                let scroll_limit = (grid.bounds() * GRID_CELL_SIZE)
                    .expand2(expand.width, expand.height);
                if self.scroll.x < scroll_limit.x {
                    self.scroll.x =
                        track_towards(self.scroll.x, scroll_limit.x, tick);
                    ui.request_redraw();
                } else if self.scroll.x > scroll_limit.right() {
                    self.scroll.x = track_towards(self.scroll.x,
                                                  scroll_limit.right(),
                                                  tick);
                    ui.request_redraw();
                }
                if self.scroll.y < scroll_limit.y {
                    self.scroll.y =
                        track_towards(self.scroll.y, scroll_limit.y, tick);
                    ui.request_redraw();
                } else if self.scroll.y > scroll_limit.bottom() {
                    self.scroll.y = track_towards(self.scroll.y,
                                                  scroll_limit.bottom(),
                                                  tick);
                    ui.request_redraw();
                }
            }
            Event::KeyDown(key) => {
                if key.code == Keycode::Backspace ||
                    key.code == Keycode::Delete
                {
                    match self.interaction {
                        Interaction::Nothing => {
                            if let Some(wire) = self.hover_wire {
                                select::delete_wire(grid, wire);
                                self.hover_wire = None;
                                ui.request_redraw();
                            }
                        }
                        Interaction::RectSelected(rect) => {
                            select::delete(grid, rect);
                            self.interaction = Interaction::Nothing;
                            ui.request_redraw();
                        }
                        _ => {}
                    }
                } else if key.command {
                    match key.code {
                        Keycode::A => {
                            self.interaction =
                                Interaction::RectSelected(grid.bounds());
                            ui.request_redraw();
                        }
                        Keycode::C => {
                            if let Interaction::RectSelected(rect) =
                                self.interaction
                            {
                                select::copy(grid, rect, ui.clipboard());
                            }
                        }
                        Keycode::V => {
                            if let Some(selection) =
                                Selection::from_clipboard(ui.clipboard(),
                                                          grid.allowed_chips())
                            {
                                self.cancel_interaction(ui, grid);
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
                                ui.request_redraw();
                            }
                        }
                        Keycode::X => {
                            if let Interaction::RectSelected(rect) =
                                self.interaction
                            {
                                select::cut(grid, rect, ui.clipboard());
                                self.interaction = Interaction::Nothing;
                                ui.request_redraw();
                            }
                        }
                        Keycode::Z if key.shift => {
                            self.cancel_interaction(ui, grid);
                            if grid.redo() {
                                ui.request_redraw();
                            }
                        }
                        Keycode::Z => {
                            if !self.cancel_interaction(ui, grid) {
                                if grid.undo() {
                                    ui.request_redraw();
                                }
                            }
                        }
                        _ => {}
                    }
                } else if let Some(hotkey) = prefs.hotkey_for_code(key.code) {
                    self.on_hotkey(hotkey, ui);
                }
                self.stop_hover(ui);
            }
            Event::MouseDown(mouse) if mouse.left => {
                let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                self.stop_hover(ui);
                if grid.eval().is_some() {
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
                    return None;
                }
                match self.interaction.take() {
                    Interaction::Nothing => {}
                    Interaction::RectSelected(rect) => {
                        if rect.contains_point(grid_pt.as_i32_floor()) {
                            let selection = select::cut_provisionally(grid,
                                                                      rect);
                            let grab_rel = grid_pt - rect.top_left().as_f32();
                            let drag = SelectionDrag::new(selection,
                                                          grab_rel,
                                                          grid_pt,
                                                          Some(rect));
                            self.interaction =
                                Interaction::DraggingSelection(drag);
                        }
                        return None;
                    }
                    Interaction::DraggingSelection(drag) => {
                        if let Some(rect) = drag.finish(ui, grid) {
                            self.interaction = Interaction::RectSelected(rect);
                        }
                        return None;
                    }
                    _ => return None,
                }
                let mouse_coords = grid_pt.as_i32_floor();
                if let Some((coords, _, iface)) =
                    grid.interface_at(mouse_coords)
                {
                    if interface_grid_rect(coords, iface)
                        .contains_point(grid_pt)
                    {
                        let handle = BoundsHandle::for_side(iface.side());
                        let drag = BoundsDrag::new(handle, grid_pt, grid);
                        self.interaction = Interaction::DraggingBounds(drag);
                        return None;
                    }
                }
                if let Some(handle) = BoundsHandle::for_grid_pt(grid_pt,
                                                                grid)
                {
                    let drag = BoundsDrag::new(handle, grid_pt, grid);
                    self.interaction = Interaction::DraggingBounds(drag);
                    return None;
                }
                if let Some((coords, ctype, orient)) =
                    grid.chip_at(mouse_coords)
                {
                    let chip_rect = chip_grid_rect(coords, ctype, orient);
                    if chip_rect.contains_point(grid_pt) {
                        let change =
                            GridChange::RemoveChip(coords, ctype, orient);
                        if grid.try_mutate_provisionally(vec![change]) {
                            let drag = ChipDrag::new(ctype,
                                                     orient,
                                                     Some(coords),
                                                     grid_pt);
                            self.interaction = Interaction::DraggingChip(drag);
                            ui.audio().play_sound(Sound::GrabChip);
                        }
                        return None;
                    }
                }
                if SelectingDrag::is_near_vertex(grid_pt, grid.bounds()) {
                    let drag = SelectingDrag::new(grid.bounds(),
                                                  grid_pt.as_i32_round());
                    self.interaction = Interaction::SelectingRect(drag);
                    ui.request_redraw();
                    return None;
                }
                if grid.bounds()
                    .as_f32()
                    .expand(CHIP_MARGIN)
                    .contains_point(grid_pt)
                {
                    let mut drag = WireDrag::new();
                    if drag.move_to(grid_pt, ui, grid) {
                        self.interaction = Interaction::DraggingWires(drag);
                    } else {
                        debug_log!("wire drag done (down)");
                    }
                }
            }
            Event::MouseDown(mouse) if mouse.right => {
                if grid.eval().is_some() {
                    return None;
                }
                let coords = self.coords_for_screen_pt(mouse.pt);
                if let Some((_, ChipType::Const(value), _)) =
                    grid.chip_at(coords)
                {
                    return Some(EditGridAction::EditConst(coords, value));
                }
                let change = GridChange::ToggleCrossWire(coords);
                if grid.try_mutate(vec![change]) {
                    // TODO: Play sound for toggling cross wire.
                    ui.request_redraw();
                }
            }
            Event::MouseMove(mouse) => {
                let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                ui.cursor().request(self.cursor_for_grid_pt(grid_pt, grid));
                let mut should_stop_interaction = false;
                match self.interaction {
                    Interaction::Nothing |
                    Interaction::RectSelected(_) => {
                        if !mouse.left && !mouse.right {
                            if let Some(tag) =
                                GridTooltipTag::for_grid_pt(grid, grid_pt)
                            {
                                if let GridTooltipTag::Wire(wire) = tag {
                                    if self.interaction.is_nothing() &&
                                        self.hover_wire != Some(wire)
                                    {
                                        self.hover_wire = Some(wire);
                                        ui.request_redraw();
                                    }
                                } else if self.hover_wire.is_some() {
                                    self.hover_wire = None;
                                    ui.request_redraw();
                                }
                                self.tooltip.start_hover(mouse.pt, ui, tag);
                            } else {
                                self.stop_hover(ui);
                            }
                        }
                    }
                    Interaction::DraggingBounds(ref mut drag) => {
                        drag.move_to(grid_pt, ui, grid);
                    }
                    Interaction::DraggingChip(ref mut drag) => {
                        drag.move_to(grid_pt, ui);
                    }
                    Interaction::SelectingRect(ref mut drag) => {
                        drag.move_to(grid_pt, ui);
                    }
                    Interaction::DraggingSelection(ref mut drag) => {
                        drag.move_to(grid_pt, ui);
                    }
                    Interaction::DraggingWires(ref mut drag) => {
                        if !drag.move_to(grid_pt, ui, grid) {
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
                            drag.drop_onto_board(ui, grid);
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
                            if let Some(rect) = drag.finish(ui, grid) {
                                self.interaction =
                                    Interaction::RectSelected(rect);
                            }
                        }
                        Interaction::DraggingWires(drag) => {
                            drag.finish(ui, grid);
                        }
                    }
                    let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                    let cursor = self.cursor_for_grid_pt(grid_pt, grid);
                    ui.cursor().request(cursor);
                }
            }
            Event::Multitouch(touch) => {
                self.stop_hover(ui);
                self.zoom_by(touch.scale, ui);
            }
            Event::Scroll(scroll) => {
                self.stop_hover(ui);
                self.scroll_by_screen_dist(scroll.delta.x, scroll.delta.y, ui);
            }
            Event::Unfocus => {
                self.stop_hover(ui);
            }
            _ => {}
        }
        return None;
    }

    fn stop_hover(&mut self, ui: &mut Ui) {
        self.tooltip.stop_hover_all(ui);
        if self.hover_wire.is_some() {
            self.hover_wire = None;
            ui.request_redraw();
        }
    }

    pub fn grab_from_parts_tray(&mut self, screen_pt: Point2<i32>,
                                ui: &mut Ui, ctype: ChipType) {
        let size = ctype.size();
        let start = 0.5 * Point2::new(size.width, size.height).as_f32();
        let mut drag =
            ChipDrag::new(ctype, Orientation::default(), None, start);
        drag.move_to(self.screen_pt_to_grid_pt(screen_pt), ui);
        self.interaction = Interaction::DraggingChip(drag);
        ui.request_redraw();
        ui.audio().play_sound(Sound::GrabChip);
    }

    pub fn drop_into_parts_tray(&mut self, ui: &mut Ui, grid: &mut EditGrid) {
        match self.interaction.take() {
            Interaction::DraggingChip(drag) => {
                drag.drop_into_parts_tray(ui, grid);
            }
            other => self.interaction = other,
        }
    }

    /// Ceases the current interaction (if any) and sets `self.interaction` to
    /// `Nothing`.  Returns true if any provisional changes were rolled back.
    fn cancel_interaction(&mut self, ui: &mut Ui, grid: &mut EditGrid)
                          -> bool {
        match self.interaction.take() {
            Interaction::DraggingChip(drag) => drag.cancel(ui, grid),
            Interaction::DraggingSelection(drag) => drag.cancel(ui, grid),
            Interaction::DraggingWires(drag) => {
                drag.finish(ui, grid);
                false
            }
            _ => false,
        }
    }

    fn zoom_by(&mut self, factor: f32, ui: &mut Ui) {
        self.zoom = ZOOM_MIN.max(self.zoom * factor).min(ZOOM_MAX);
        ui.request_redraw();
    }

    fn scroll_by_screen_dist(&mut self, x: i32, y: i32, ui: &mut Ui) {
        self.scroll += (vec2(x, y).as_f32() / self.zoom).as_i32_round();
        ui.request_redraw();
    }

    fn screen_pt_to_grid_pt(&self, screen_pt: Point2<i32>) -> Point2<f32> {
        let half_size = self.size * 0.5;
        (((screen_pt.as_f32() - vec2(half_size.width, half_size.height)) /
              self.zoom)
             .as_i32_round() + self.scroll)
            .as_f32() / (GRID_CELL_SIZE as f32)
    }

    fn coords_for_screen_pt(&self, screen_pt: Point2<i32>) -> Coords {
        self.screen_pt_to_grid_pt(screen_pt).as_i32_floor()
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

fn track_towards(current: i32, goal: i32, tick: &ClockEventData) -> i32 {
    let tracking_base: f64 = 0.0001; // smaller = faster tracking
    let difference = (goal - current) as f64;
    let change = difference * (1.0 - tracking_base.powf(tick.elapsed));
    current + (change.round() as i32)
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
    fn is_nothing(&self) -> bool {
        match self {
            Interaction::Nothing => true,
            _ => false,
        }
    }

    fn take(&mut self) -> Interaction {
        mem::replace(self, Interaction::Nothing)
    }
}

//===========================================================================//
