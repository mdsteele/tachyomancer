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

use super::super::chip::{
    chip_grid_rect, interface_grid_rect, ChipModel, CHIP_MARGIN,
};
use super::super::tooltip::TooltipSink;
use super::super::wire::WireModel;
use super::bounds::{BoundsDrag, BoundsHandle, BOUNDS_MARGIN};
use super::chipdrag::ChipDrag;
use super::manip::{ManipulationAction, ManipulationButtons};
use super::select::{self, SelectingDrag, Selection, SelectionDrag};
use super::tooltip::GridTooltipTag;
use super::tutorial::TutorialBubble;
use super::wiredrag::WireDrag;
use crate::mancer::gl::Depth;
use crate::mancer::gui::{
    ClockEventData, Cursor, Event, Keycode, MouseEventData, NextCursor,
    Resources, Sound, Ui,
};
use crate::mancer::save::{Hotkey, HotkeyCodeExt, Prefs};
use cgmath::{self, vec2, Matrix4, Point2, Vector2};
use std::collections::HashSet;
use std::mem;
use tachy::geom::{
    AsFloat, AsInt, Color3, Color4, Coords, CoordsRect, Direction, MatrixExt,
    Orientation, Rect, RectSize,
};
use tachy::save::{ChipType, HotkeyCode, WireSize};
use tachy::state::{EditGrid, GridChange, WireColor};

//===========================================================================//

// The size, in screen pixels, of a grid cell at 1x zoom:
const GRID_CELL_SIZE: i32 = 64;

// How far we scroll per second while holding down a scroll hotkey, in grid
// cells, at max zoom:
const SCROLL_GRID_CELLS_PER_SECOND: f64 = 12.0;

// The minimum zoom multiplier (i.e. how far zoomed out you can be):
const ZOOM_MIN: f32 = 0.25;
// The default zoom multiplier:
const ZOOM_DEFAULT: f32 = 1.0;
// The maximum zoom multiplier (i.e. how far zoomed in you can be):
const ZOOM_MAX: f32 = 2.0;
// How much to multiply/divide the zoom by when pressing a zoom hotkey:
const ZOOM_PER_KEYDOWN: f32 = 1.415; // slightly more than sqrt(2)

//===========================================================================//

pub enum EditGridAction {
    EditButton(Coords, Option<HotkeyCode>),
    EditCoerce(Coords, WireSize),
    EditComment(Coords, String),
    EditConst(Coords, u16),
}

//===========================================================================//

pub struct EditGridView {
    size: RectSize<f32>,
    scroll: Vector2<i32>,
    zoom: f32,
    interaction: Interaction,
    tutorial_bubbles: Vec<(Direction, TutorialBubble)>,
    hover_wire: Option<usize>,
    manip_buttons: ManipulationButtons,
}

impl EditGridView {
    pub fn new(
        window_size: RectSize<i32>,
        init_circuit_bounds: CoordsRect,
        tutorial_bubbles: Vec<(Direction, TutorialBubble)>,
    ) -> EditGridView {
        let pixel_bounds = init_circuit_bounds * GRID_CELL_SIZE;
        EditGridView {
            size: window_size.as_f32(),
            scroll: Vector2::new(
                pixel_bounds.x + pixel_bounds.width / 2,
                pixel_bounds.y + pixel_bounds.height / 2,
            ),
            zoom: ZOOM_DEFAULT,
            interaction: Interaction::Nothing,
            tutorial_bubbles,
            hover_wire: None,
            manip_buttons: ManipulationButtons::new(),
        }
    }

    fn draw_background(&self, resources: &Resources) {
        let size = self.size / self.zoom;
        let parallax = 0.9;
        let texel_rect = Rect::new(
            parallax * (self.scroll.x as f32) - 0.5 * size.width,
            parallax * (self.scroll.y as f32) - 0.5 * size.height,
            size.width,
            size.height,
        );
        resources.shaders().diagram().draw(
            &cgmath::ortho(0.0, 1.0, 1.0, 0.0, -1.0, 1.0),
            Rect::new(0.0, 0.0, 1.0, 1.0),
            texel_rect / 512.0,
            resources.textures().diagram_background(),
        );
    }

    fn draw_bounds(
        &self,
        resources: &Resources,
        grid_matrix: &Matrix4<f32>,
        grid: &EditGrid,
    ) {
        let (bounds, is_acceptable) = match self.interaction {
            Interaction::DraggingBounds(ref drag) => {
                (drag.bounds(), drag.is_acceptable())
            }
            _ => (grid.bounds(), true),
        };
        resources.shaders().board().draw(grid_matrix, bounds.as_f32());

        let bounds = bounds.as_f32();
        let color = if is_acceptable {
            Color3::PURPLE2
        } else {
            Color3::new(1.0, 0.0, 0.0)
        };
        let rect = Rect::new(
            bounds.x - BOUNDS_MARGIN,
            bounds.y - BOUNDS_MARGIN,
            BOUNDS_MARGIN,
            bounds.height + 2.0 * BOUNDS_MARGIN,
        );
        resources.shaders().solid().fill_rect(&grid_matrix, color, rect);
        let rect = Rect::new(
            bounds.x,
            bounds.y - BOUNDS_MARGIN,
            bounds.width,
            BOUNDS_MARGIN,
        );
        resources.shaders().solid().fill_rect(&grid_matrix, color, rect);
        let rect = Rect::new(
            bounds.x + bounds.width,
            bounds.y - BOUNDS_MARGIN,
            BOUNDS_MARGIN,
            bounds.height + 2.0 * BOUNDS_MARGIN,
        );
        resources.shaders().solid().fill_rect(&grid_matrix, color, rect);
        let rect = Rect::new(
            bounds.x,
            bounds.y + bounds.height,
            bounds.width,
            BOUNDS_MARGIN,
        );
        resources.shaders().solid().fill_rect(&grid_matrix, color, rect);
    }

    fn draw_tutorial_bubbles(&self, resources: &Resources, grid: &EditGrid) {
        let matrix = self.unzoomed_matrix();
        let bounds =
            if let Interaction::DraggingBounds(ref drag) = self.interaction {
                drag.bounds()
            } else {
                grid.bounds()
            };
        let bounds = bounds.as_f32().expand(BOUNDS_MARGIN)
            * ((GRID_CELL_SIZE as f32) * self.zoom);
        let margin: i32 = 8;
        for &(dir, ref bubble) in self.tutorial_bubbles.iter() {
            let topleft = match dir {
                Direction::East => Point2::new(
                    (bounds.right().round() as i32) + margin,
                    ((bounds.y + 0.5 * bounds.height).round() as i32)
                        - bubble.height() / 2,
                ),
                Direction::South => Point2::new(
                    ((bounds.x + 0.5 * bounds.width).round() as i32)
                        - bubble.width() / 2,
                    (bounds.bottom().round() as i32) + margin,
                ),
                Direction::West => Point2::new(
                    (bounds.x.round() as i32) - margin - bubble.width(),
                    ((bounds.y + 0.5 * bounds.height).round() as i32)
                        - bubble.height() / 2,
                ),
                Direction::North => Point2::new(
                    ((bounds.x + 0.5 * bounds.width).round() as i32)
                        - bubble.width() / 2,
                    (bounds.y.round() as i32) - margin - bubble.height(),
                ),
            };
            bubble.draw(resources, &matrix, topleft);
        }
    }

    fn draw_chips(
        &self,
        resources: &Resources,
        grid_matrix: &Matrix4<f32>,
        grid: &EditGrid,
    ) {
        let dragged_chip_coords = match self.interaction {
            Interaction::DraggingChip(ref drag) => drag.old_coords(),
            _ => None,
        };
        for (coords, ctype, orient) in grid.chips() {
            if Some(coords) == dragged_chip_coords {
                continue;
            }
            ChipModel::draw_chip(
                resources,
                &grid_matrix,
                coords,
                ctype,
                orient,
                Some(grid),
            );
        }
    }

    fn draw_interfaces(
        &self,
        resources: &Resources,
        grid_matrix: &Matrix4<f32>,
        grid: &EditGrid,
    ) {
        let bounds = match self.interaction {
            Interaction::DraggingBounds(ref drag) => drag.bounds(),
            _ => grid.bounds(),
        };
        for interface in grid.interfaces() {
            let coords = interface.top_left(bounds);
            ChipModel::draw_interface(
                resources,
                &grid_matrix,
                coords,
                interface,
            );
        }
    }

    fn draw_wires(
        &self,
        resources: &Resources,
        grid_matrix: &Matrix4<f32>,
        grid: &EditGrid,
    ) {
        let subcycle_wires: HashSet<usize> = if let Some(eval) = grid.eval() {
            if eval.subcycle() > 0 {
                grid.wire_index_group(eval.subcycle() - 1)
                    .iter()
                    .copied()
                    .filter(|&wire_index| eval.wire_has_change(wire_index))
                    .collect()
            } else {
                HashSet::new()
            }
        } else {
            HashSet::new()
        };
        let half_wire =
            if let Interaction::DraggingWires(ref drag) = self.interaction {
                drag.half_wire()
            } else {
                None
            };
        for (coords, dir, shape, size, color, has_error) in
            grid.wire_fragments()
        {
            let wire_index = grid.wire_index_at(coords, dir).unwrap();
            let hilight = if self.hover_wire == Some(wire_index) {
                &Color4::CYAN5
            } else if subcycle_wires.contains(&wire_index) {
                &Color4::YELLOW5
            } else {
                &Color4::TRANSPARENT
            };
            // TODO: When a wire with an error is selected, we should hilight
            //   the causes of the error (e.g. the two sender ports, or the
            //   wire loop, or whatever).
            let color = if has_error { WireColor::Ambiguous } else { color };
            if half_wire == Some((coords, dir)) {
                WireModel::draw_half_straight(
                    resources,
                    &grid_matrix,
                    coords,
                    dir,
                    color,
                    size,
                    hilight,
                );
            } else {
                WireModel::draw_fragment(
                    resources,
                    &grid_matrix,
                    coords,
                    dir,
                    shape,
                    color,
                    size,
                    hilight,
                );
            }
        }
    }

    fn draw_selection_box_if_any(&self, resources: &Resources) {
        let grid_cell_size = (GRID_CELL_SIZE as f32) * self.zoom;
        match self.interaction {
            Interaction::SelectingRect(ref drag) => {
                drag.draw_box(
                    resources,
                    &self.unzoomed_matrix(),
                    grid_cell_size,
                );
            }
            Interaction::RectSelected(rect) => {
                let matrix = self.unzoomed_matrix();
                Selection::draw_box(resources, &matrix, rect, grid_cell_size);
                self.manip_buttons.draw(
                    resources,
                    &matrix,
                    rect,
                    grid_cell_size,
                );
            }
            Interaction::DraggingSelection(ref drag) => {
                drag.draw_selection(
                    resources,
                    &self.unzoomed_matrix(),
                    grid_cell_size,
                );
            }
            _ => {}
        }
    }

    pub fn draw_board(&self, resources: &Resources, grid: &EditGrid) {
        let grid_matrix =
            self.vp_matrix() * Matrix4::from_scale(GRID_CELL_SIZE as f32);
        self.draw_background(resources);
        self.draw_bounds(resources, &grid_matrix, grid);
        self.draw_tutorial_bubbles(resources, grid);

        let depth = Depth::enable_with_face_culling(false);
        self.draw_chips(resources, &grid_matrix, grid);
        self.draw_interfaces(resources, &grid_matrix, grid);
        self.draw_wires(resources, &grid_matrix, grid);
        depth.disable();

        self.draw_selection_box_if_any(resources);
    }

    pub fn draw_dragged(&self, resources: &Resources) {
        if let Interaction::DraggingChip(ref drag) = self.interaction {
            let pt = drag.chip_topleft();
            let grid_matrix = self.vp_matrix()
                * Matrix4::from_scale(GRID_CELL_SIZE as f32)
                * Matrix4::trans2(pt.x, pt.y);
            let depth = Depth::enable_with_face_culling(false);
            ChipModel::draw_chip(
                resources,
                &grid_matrix,
                Coords::new(0, 0),
                drag.chip_type(),
                drag.new_orient(),
                None,
            );
            depth.disable();
        }
    }

    fn ortho_matrix(&self) -> Matrix4<f32> {
        cgmath::ortho(
            -0.5 * self.size.width,
            0.5 * self.size.width,
            0.5 * self.size.height,
            -0.5 * self.size.height,
            -100.0,
            100.0,
        )
    }

    fn vp_matrix(&self) -> Matrix4<f32> {
        self.ortho_matrix()
            * Matrix4::from_scale(self.zoom)
            * Matrix4::trans2(-self.scroll.x as f32, -self.scroll.y as f32)
    }

    fn unzoomed_matrix(&self) -> Matrix4<f32> {
        self.ortho_matrix()
            * Matrix4::trans2(
                (-self.scroll.x as f32) * self.zoom,
                (-self.scroll.y as f32) * self.zoom,
            )
    }

    pub fn request_interaction_cursor(
        &self,
        event: &Event,
        next_cursor: &mut NextCursor,
    ) {
        match event {
            Event::MouseUp(mouse) if mouse.left => return,
            _ => {}
        }
        match self.interaction {
            Interaction::DraggingBounds(ref drag) => {
                drag.request_cursor(next_cursor);
            }
            Interaction::DraggingChip(_)
            | Interaction::DraggingSelection(_) => {
                next_cursor.request(Cursor::HandClosed);
            }
            Interaction::DraggingWires(_) => {
                next_cursor.request(Cursor::Wire);
            }
            Interaction::SelectingRect(_) => {
                next_cursor.request(Cursor::Crosshair);
            }
            Interaction::Nothing | Interaction::RectSelected(_) => {}
        }
    }

    fn cursor_for_grid_pt(
        &self,
        grid_pt: Point2<f32>,
        grid: &EditGrid,
    ) -> Cursor {
        let coords = grid_pt.as_i32_floor();
        match self.interaction {
            Interaction::Nothing => {
                if grid.eval().is_some() {
                    match grid.chip_at(coords) {
                        Some((_, ChipType::Break(_), _))
                        | Some((_, ChipType::Button(_), _))
                        | Some((_, ChipType::Screen, _))
                        | Some((_, ChipType::Toggle(_), _)) => {
                            return Cursor::HandPointing;
                        }
                        _ => return Cursor::NoSign,
                    }
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
                if grid
                    .bounds()
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

    fn on_hotkey(&mut self, hotkey: Hotkey, ui: &mut Ui, grid: &mut EditGrid) {
        if hotkey == Hotkey::ZoomIn {
            self.zoom_by(ZOOM_PER_KEYDOWN, ui);
        } else if hotkey == Hotkey::ZoomOut {
            self.zoom_by(1.0 / ZOOM_PER_KEYDOWN, ui);
        } else if hotkey == Hotkey::ZoomDefault {
            if self.zoom != ZOOM_DEFAULT {
                self.zoom = ZOOM_DEFAULT;
                ui.request_redraw();
            }
        } else if let Some(action) = ManipulationAction::from_hotkey(hotkey) {
            self.apply_manipulation(action, ui, grid);
        }
    }

    fn apply_manipulation(
        &mut self,
        action: ManipulationAction,
        ui: &mut Ui,
        grid: &mut EditGrid,
    ) {
        match self.interaction {
            Interaction::DraggingChip(ref mut drag) => match action {
                ManipulationAction::FlipHorz => drag.flip_horz(ui),
                ManipulationAction::FlipVert => drag.flip_vert(ui),
                ManipulationAction::RotateCcw => drag.rotate_ccw(ui),
                ManipulationAction::RotateCw => drag.rotate_cw(ui),
            },
            Interaction::DraggingSelection(ref mut drag) => match action {
                ManipulationAction::FlipHorz => drag.flip_horz(ui),
                ManipulationAction::FlipVert => drag.flip_vert(ui),
                ManipulationAction::RotateCcw => drag.rotate_ccw(ui),
                ManipulationAction::RotateCw => drag.rotate_cw(ui),
            },
            Interaction::RectSelected(rect) => {
                match action {
                    ManipulationAction::FlipHorz => {
                        select::flip_horz(grid, rect);
                    }
                    ManipulationAction::FlipVert => {
                        select::flip_vert(grid, rect);
                    }
                    ManipulationAction::RotateCcw => {
                        select::rotate_ccw(grid, rect);
                    }
                    ManipulationAction::RotateCw => {
                        select::rotate_cw(grid, rect);
                    }
                }
                ui.request_redraw();
            }
            _ => {}
        }
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        grid: &mut EditGrid,
        tooltip: &mut dyn TooltipSink<GridTooltipTag>,
        prefs: &Prefs,
    ) -> Option<EditGridAction> {
        if let Interaction::RectSelected(rect) = self.interaction {
            let top_left = self.grid_pt_to_screen_pt(rect.top_left().as_f32());
            let act = self.manip_buttons.on_event(event, ui, rect, top_left);
            if let Some(action) = act {
                self.apply_manipulation(action, ui, grid);
                return None;
            }
        }
        match event {
            Event::ClockTick(tick) => {
                // Scroll if we're holding down any scroll key(s):
                let left = is_hotkey_held(Hotkey::ScrollLeft, ui, prefs);
                let right = is_hotkey_held(Hotkey::ScrollRight, ui, prefs);
                let up = is_hotkey_held(Hotkey::ScrollUp, ui, prefs);
                let down = is_hotkey_held(Hotkey::ScrollDown, ui, prefs);
                let dist = ((SCROLL_GRID_CELLS_PER_SECOND * tick.elapsed)
                    * (GRID_CELL_SIZE as f64))
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
                    self.scroll.x = track_towards(
                        self.scroll.x,
                        scroll_limit.right(),
                        tick,
                    );
                    ui.request_redraw();
                }
                if self.scroll.y < scroll_limit.y {
                    self.scroll.y =
                        track_towards(self.scroll.y, scroll_limit.y, tick);
                    ui.request_redraw();
                } else if self.scroll.y > scroll_limit.bottom() {
                    self.scroll.y = track_towards(
                        self.scroll.y,
                        scroll_limit.bottom(),
                        tick,
                    );
                    ui.request_redraw();
                }
            }
            Event::KeyDown(key) => {
                if key.code == Keycode::Backspace
                    || key.code == Keycode::Delete
                {
                    match self.interaction {
                        Interaction::Nothing => {
                            if let Some(wire) = self.hover_wire {
                                select::delete_wire(grid, wire);
                                self.hover_wire = None;
                                // TODO: play sound for delete
                                ui.request_redraw();
                            }
                        }
                        Interaction::RectSelected(rect) => {
                            self.manip_buttons.unfocus();
                            select::delete(grid, rect);
                            self.interaction = Interaction::Nothing;
                            // TODO: play sound for delete
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
                            if let Some(selection) = Selection::from_clipboard(
                                ui.clipboard(),
                                grid.allowed_chips(),
                            ) {
                                self.cancel_interaction(ui, grid);
                                let size = selection.size().as_f32();
                                let rel = vec2(size.width, size.height) * 0.5;
                                let grid_pt =
                                    self.screen_pt_to_grid_pt(key.mouse_pt);
                                let drag = SelectionDrag::new(
                                    selection, rel, grid_pt, None,
                                );
                                self.interaction =
                                    Interaction::DraggingSelection(drag);
                                ui.request_redraw();
                            }
                        }
                        Keycode::X => {
                            if let Interaction::RectSelected(rect) =
                                self.interaction
                            {
                                self.manip_buttons.unfocus();
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
                } else if let Some(code) = HotkeyCode::from_keycode(key.code) {
                    if let Some(hotkey) = prefs.hotkey_for_code(code) {
                        self.on_hotkey(hotkey, ui, grid);
                    } else if let Some(eval) = grid.eval_mut() {
                        eval.press_hotkey(code);
                    }
                }
                self.stop_hover(ui);
            }
            Event::MouseDown(mouse) if mouse.left => {
                let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                self.stop_hover(ui);
                if grid.eval().is_some() {
                    match grid.chip_at(grid_pt.as_i32_floor()) {
                        Some((coords, ChipType::Break(_), _)) => {
                            // TODO: Play sound for toggling breakpoint.
                            grid.press_button(coords, 0);
                            ui.request_redraw();
                        }
                        Some((coords, ChipType::Button(_), _)) => {
                            // TODO: Play sound for pressing button.
                            grid.press_button(coords, 0);
                        }
                        Some((coords, ChipType::Screen, _)) => {
                            if let Some(sublocation) =
                                ChipModel::chip_screen_cell(coords, grid_pt)
                            {
                                grid.press_button(coords, sublocation);
                            }
                        }
                        Some((coords, ChipType::Toggle(_), _)) => {
                            // TODO: Play sound for flipping toggle switch.
                            grid.press_button(coords, 0);
                        }
                        _ => {}
                    }
                    return None;
                }
                match self.interaction.take() {
                    Interaction::Nothing => {}
                    Interaction::RectSelected(rect) => {
                        self.manip_buttons.unfocus();
                        if rect.contains_point(grid_pt.as_i32_floor()) {
                            let selection =
                                select::cut_provisionally(grid, rect);
                            let grab_rel = grid_pt - rect.top_left().as_f32();
                            let drag = SelectionDrag::new(
                                selection,
                                grab_rel,
                                grid_pt,
                                Some(rect),
                            );
                            self.interaction =
                                Interaction::DraggingSelection(drag);
                        } else {
                            let cursor =
                                self.cursor_for_grid_pt(grid_pt, grid);
                            ui.cursor().request(cursor);
                            ui.request_redraw();
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
                if let Some(handle) = BoundsHandle::for_grid_pt(grid_pt, grid)
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
                            let drag = ChipDrag::new(
                                ctype,
                                orient,
                                Some(coords),
                                grid_pt,
                            );
                            self.interaction = Interaction::DraggingChip(drag);
                            ui.audio().play_sound(Sound::GrabChip);
                        }
                        return None;
                    }
                }
                if SelectingDrag::is_near_vertex(grid_pt, grid.bounds()) {
                    let drag = SelectingDrag::new(
                        grid.bounds(),
                        grid_pt.as_i32_round(),
                    );
                    self.interaction = Interaction::SelectingRect(drag);
                    ui.request_redraw();
                    return None;
                }
                if grid
                    .bounds()
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
                    // TODO: May as well let right-click continue to work for
                    //   Break/Toggle chips during evaluation.
                    return None;
                }
                let coords = self.coords_for_screen_pt(mouse.pt);
                match grid.chip_at(coords) {
                    Some((_, ChipType::Break(enabled), orient)) => {
                        if try_toggle_break(coords, enabled, orient, grid) {
                            // TODO: Play sound for toggling breakpoint.
                            ui.request_redraw();
                            return None;
                        }
                    }
                    Some((_, ChipType::Button(code), _)) => {
                        return Some(EditGridAction::EditButton(coords, code));
                    }
                    Some((_, ChipType::Coerce(size), _)) => {
                        return Some(EditGridAction::EditCoerce(coords, size));
                    }
                    Some((_, ChipType::Comment(bytes), _)) => {
                        let string: String =
                            bytes.iter().map(|&b| char::from(b)).collect();
                        let string = string.trim().to_string();
                        return Some(EditGridAction::EditComment(
                            coords, string,
                        ));
                    }
                    Some((_, ChipType::Const(value), _)) => {
                        return Some(EditGridAction::EditConst(coords, value));
                    }
                    Some((_, ChipType::Toggle(value), orient)) => {
                        if try_toggle_switch(coords, value, orient, grid) {
                            // TODO: Play sound for flipping toggle switch.
                            ui.request_redraw();
                            return None;
                        }
                    }
                    _ => {}
                }
                if WireDrag::try_toggle_cross(coords, grid) {
                    debug_assert!(grid.has_provisional_changes());
                    grid.commit_provisional_changes();
                    ui.audio().play_sound(Sound::DragWire);
                    ui.request_redraw();
                }
            }
            Event::MouseMove(mouse) => {
                let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
                ui.cursor().request(self.cursor_for_grid_pt(grid_pt, grid));
                match self.interaction {
                    Interaction::Nothing | Interaction::RectSelected(_) => {
                        self.apply_tooltip(mouse, ui, grid, tooltip);
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
                            self.interaction = Interaction::Nothing;
                        }
                    }
                }
            }
            Event::MouseUp(mouse) => {
                if mouse.left {
                    match self.interaction.take() {
                        Interaction::Nothing => {}
                        Interaction::DraggingBounds(drag) => {
                            drag.finish(ui, grid);
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
                            ui.request_redraw();
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

    fn apply_tooltip(
        &mut self,
        mouse: &MouseEventData,
        ui: &mut Ui,
        grid: &EditGrid,
        tooltip: &mut dyn TooltipSink<GridTooltipTag>,
    ) {
        if mouse.left || mouse.right {
            return;
        }
        if let Some(action) = self.manip_buttons.hovered_action() {
            if self.hover_wire.is_some() {
                self.hover_wire = None;
                ui.request_redraw();
            }
            tooltip.hover_tag(
                mouse.pt,
                ui,
                GridTooltipTag::Manipulation(action),
            );
            return;
        }
        let grid_pt = self.screen_pt_to_grid_pt(mouse.pt);
        if let Some(tag) = GridTooltipTag::for_grid_pt(grid, grid_pt) {
            if let GridTooltipTag::Wire(wire) = tag {
                if self.interaction.is_nothing()
                    && self.hover_wire != Some(wire)
                {
                    self.hover_wire = Some(wire);
                    ui.request_redraw();
                }
            } else if self.hover_wire.is_some() {
                self.hover_wire = None;
                ui.request_redraw();
            }
            tooltip.hover_tag(mouse.pt, ui, tag);
            return;
        }
        tooltip.hover_none(ui);
        self.stop_hover(ui);
    }

    fn stop_hover(&mut self, ui: &mut Ui) {
        if self.hover_wire.is_some() {
            self.hover_wire = None;
            ui.request_redraw();
        }
    }

    pub fn grab_from_parts_tray(
        &mut self,
        screen_pt: Point2<i32>,
        ui: &mut Ui,
        ctype: ChipType,
    ) {
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

    pub fn has_interaction(&self) -> bool {
        match self.interaction {
            Interaction::Nothing => false,
            _ => true,
        }
    }

    pub fn is_dragging(&self) -> bool {
        match self.interaction {
            Interaction::Nothing | Interaction::RectSelected(_) => false,
            _ => true,
        }
    }

    /// Ceases the current interaction (if any) and sets `self.interaction` to
    /// `Nothing`.  Returns true if any provisional changes were rolled back.
    pub fn cancel_interaction(
        &mut self,
        ui: &mut Ui,
        grid: &mut EditGrid,
    ) -> bool {
        match self.interaction.take() {
            Interaction::Nothing => false,
            Interaction::DraggingChip(drag) => drag.cancel(ui, grid),
            Interaction::DraggingSelection(drag) => drag.cancel(ui, grid),
            Interaction::DraggingWires(drag) => {
                drag.finish(ui, grid);
                false
            }
            Interaction::RectSelected(_) => {
                self.manip_buttons.unfocus();
                ui.request_redraw();
                false
            }
            Interaction::DraggingBounds(_) | Interaction::SelectingRect(_) => {
                ui.request_redraw();
                false
            }
        }
    }

    fn zoom_by(&mut self, factor: f32, ui: &mut Ui) {
        if factor < 1.0 {
            let minimum =
                if self.zoom > ZOOM_DEFAULT { ZOOM_DEFAULT } else { ZOOM_MIN };
            self.zoom = (self.zoom * factor).max(minimum);
            ui.request_redraw();
        } else if factor > 1.0 {
            let maximum =
                if self.zoom < ZOOM_DEFAULT { ZOOM_DEFAULT } else { ZOOM_MAX };
            self.zoom = (self.zoom * factor).min(maximum);
            ui.request_redraw();
        }
    }

    fn scroll_by_screen_dist(&mut self, x: i32, y: i32, ui: &mut Ui) {
        self.scroll += (vec2(x, y).as_f32() / self.zoom).as_i32_round();
        ui.request_redraw();
    }

    fn screen_pt_to_grid_pt(&self, screen_pt: Point2<i32>) -> Point2<f32> {
        let half_size = self.size * 0.5;
        let relative_to_center =
            screen_pt.as_f32() - vec2(half_size.width, half_size.height);
        let zoomed = relative_to_center / self.zoom;
        let scrolled = zoomed.as_i32_round() + self.scroll;
        scrolled.as_f32() / (GRID_CELL_SIZE as f32)
    }

    fn grid_pt_to_screen_pt(&self, grid_pt: Point2<f32>) -> Point2<i32> {
        let scrolled = (grid_pt * (GRID_CELL_SIZE as f32)).as_i32_round();
        let zoomed = (scrolled - self.scroll).as_f32();
        let relative_to_center = zoomed * self.zoom;
        let half_size = self.size * 0.5;
        (relative_to_center + vec2(half_size.width, half_size.height))
            .as_i32_round()
    }

    fn coords_for_screen_pt(&self, screen_pt: Point2<i32>) -> Coords {
        self.screen_pt_to_grid_pt(screen_pt).as_i32_floor()
    }
}

fn is_hotkey_held(hotkey: Hotkey, ui: &Ui, prefs: &Prefs) -> bool {
    ui.keyboard().is_held(prefs.hotkey_code(hotkey).to_keycode())
}

fn track_towards(current: i32, goal: i32, tick: &ClockEventData) -> i32 {
    let tracking_base: f64 = 0.0001; // smaller = faster tracking
    let difference = (goal - current) as f64;
    let change = difference * (1.0 - tracking_base.powf(tick.elapsed));
    current + (change.round() as i32)
}

fn try_toggle_break(
    coords: Coords,
    enabled: bool,
    orient: Orientation,
    grid: &mut EditGrid,
) -> bool {
    let changes = vec![
        GridChange::RemoveChip(coords, ChipType::Break(enabled), orient),
        GridChange::AddChip(coords, ChipType::Break(!enabled), orient),
    ];
    grid.try_mutate(changes)
}

fn try_toggle_switch(
    coords: Coords,
    value: bool,
    orient: Orientation,
    grid: &mut EditGrid,
) -> bool {
    let changes = vec![
        GridChange::RemoveChip(coords, ChipType::Toggle(value), orient),
        GridChange::AddChip(coords, ChipType::Toggle(!value), orient),
    ];
    grid.try_mutate(changes)
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
