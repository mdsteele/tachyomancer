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

use super::super::chip::ChipModel;
use super::super::wire::WireModel;
use crate::mancer::gl::Depth;
use crate::mancer::gui::{Clipboard, Resources, Sound, Ui};
use cgmath::{vec2, Matrix4, MetricSpace, Point2, Vector2};
use std::collections::{HashMap, HashSet};
use tachy::geom::{
    AsFloat, AsInt, Color3, Color4, Coords, CoordsDelta, CoordsRect,
    CoordsSize, Direction, MatrixExt, Orientation, Rect,
};
use tachy::save::{ChipSet, ChipType, CircuitData, WireShape, WireSize};
use tachy::state::{ChipExt, EditGrid, GridChange, WireColor};

//===========================================================================//

// How close, in grid cells, the mouse must be to a grid vertex to start a
// selection rect.
const SELECTING_VERTEX_MAX_DIST: f32 = 0.2;

const SELECTION_BOX_COLOR1: Color4 = Color3::CYAN5.with_alpha(0.75);
const SELECTION_BOX_COLOR2: Color4 = Color3::CYAN4.with_alpha(0.75);
const SELECTION_BOX_COLOR3: Color4 = Color3::CYAN4.with_alpha(0.1);

//===========================================================================//

pub struct SelectingDrag {
    bounds: CoordsRect,
    start: Coords,
    rect: CoordsRect,
}

impl SelectingDrag {
    pub fn is_near_vertex(grid_pt: Point2<f32>, bounds: CoordsRect) -> bool {
        let coords = grid_pt.as_i32_round();
        let expanded =
            Rect::new(bounds.x, bounds.y, bounds.width + 1, bounds.height + 1);
        expanded.contains_point(coords)
            && grid_pt.distance(coords.as_f32()) <= SELECTING_VERTEX_MAX_DIST
    }

    pub fn new(bounds: CoordsRect, start: Coords) -> SelectingDrag {
        let rect = Rect::new(start.x, start.y, 0, 0);
        SelectingDrag { bounds, start, rect }
    }

    pub fn selected_rect(&self) -> CoordsRect {
        self.rect
    }

    pub fn move_to(&mut self, grid_pt: Point2<f32>, ui: &mut Ui) {
        let coords = grid_pt.as_i32_round();
        let new_rect = Rect::new(
            self.start.x.min(coords.x),
            self.start.y.min(coords.y),
            (self.start.x - coords.x).abs(),
            (self.start.y - coords.y).abs(),
        );
        let new_rect = new_rect.intersection(self.bounds);
        if self.rect != new_rect {
            self.rect = new_rect;
            ui.request_redraw();
        }
    }

    pub fn draw_box(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        grid_cell_size: f32,
    ) {
        draw_selection_box(
            resources,
            matrix,
            self.rect,
            grid_cell_size,
            vec2(0.0, 0.0),
        );
    }
}

//===========================================================================//

pub struct SelectionDrag {
    selection: Selection,
    grab_rel: Vector2<f32>,
    grid_pt: Point2<f32>,
    original_selected_rect: Option<CoordsRect>,
}

impl SelectionDrag {
    pub fn new(
        selection: Selection,
        grab_rel: Vector2<f32>,
        grid_pt: Point2<f32>,
        original_selected_rect: Option<CoordsRect>,
    ) -> SelectionDrag {
        SelectionDrag { selection, grab_rel, grid_pt, original_selected_rect }
    }

    fn top_left_grid_pt(&self) -> Point2<f32> {
        self.grid_pt - self.grab_rel
    }

    pub fn move_to(&mut self, grid_pt: Point2<f32>, ui: &mut Ui) {
        self.grid_pt = grid_pt;
        ui.request_redraw();
    }

    pub fn flip_horz(&mut self, ui: &mut Ui) {
        self.selection.reorient(Orientation::default().flip_horz());
        ui.request_redraw();
    }

    pub fn flip_vert(&mut self, ui: &mut Ui) {
        self.selection.reorient(Orientation::default().flip_vert());
        ui.request_redraw();
    }

    pub fn rotate_cw(&mut self, ui: &mut Ui) {
        self.selection.reorient(Orientation::default().rotate_cw());
        ui.request_redraw();
    }

    pub fn rotate_ccw(&mut self, ui: &mut Ui) {
        self.selection.reorient(Orientation::default().rotate_ccw());
        ui.request_redraw();
    }

    pub fn draw_selection(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        grid_cell_size: f32,
    ) {
        let offset = self.top_left_grid_pt() - Point2::new(0.0, 0.0);

        {
            let grid_matrix = matrix
                * Matrix4::from_scale(grid_cell_size)
                * Matrix4::trans2v(offset);
            let depth = Depth::enable_with_face_culling(false);
            // Draw chips:
            for (&delta, &(ctype, orient)) in self.selection.chips.iter() {
                let coords = Coords::new(0, 0) + delta;
                ChipModel::draw_chip(
                    resources,
                    &grid_matrix,
                    coords,
                    ctype,
                    orient,
                    None,
                );
            }
            // Draw wires:
            let color = WireColor::Unknown;
            let size = WireSize::One;
            let hilight = &Color4::TRANSPARENT;
            for (&(delta, dir), &shape) in self.selection.wires.iter() {
                let coords = Coords::new(0, 0) + delta;
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
            depth.disable();
        }

        // Draw selection box:
        let rect = Rect::with_size(Coords::new(0, 0), self.selection.size());
        // TODO: color box red if we cannot drop the selection here
        draw_selection_box(resources, matrix, rect, grid_cell_size, offset);
    }

    /// Cancels/undoes the in-progress selection drag.  Returns true if any
    /// provisional changes were rolled back.
    pub fn cancel(self, ui: &mut Ui, grid: &mut EditGrid) -> bool {
        ui.request_redraw();
        grid.roll_back_provisional_changes()
    }

    pub fn finish(
        self,
        ui: &mut Ui,
        grid: &mut EditGrid,
    ) -> Option<CoordsRect> {
        ui.request_redraw();
        let top_left_coords = self.top_left_grid_pt().as_i32_round();
        let changes =
            changes_for_paste(grid, &self.selection, top_left_coords);
        if grid.try_mutate_provisionally(changes) {
            grid.commit_provisional_changes();
            ui.audio().play_sound(Sound::DropChip);
            if self.original_selected_rect.is_some() {
                Some(Rect::with_size(top_left_coords, self.selection.size()))
            } else {
                None
            }
        } else {
            grid.roll_back_provisional_changes();
            self.original_selected_rect
        }
    }
}

//===========================================================================//

pub struct Selection {
    size: CoordsSize,
    chips: HashMap<CoordsDelta, (ChipType, Orientation)>,
    wires: HashMap<(CoordsDelta, Direction), WireShape>,
}

impl Selection {
    fn new(
        rect: CoordsRect,
        chips: &HashMap<Coords, (ChipType, Orientation)>,
        wires: &HashMap<(Coords, Direction), WireShape>,
    ) -> Selection {
        let origin = rect.top_left();
        let chips = chips
            .iter()
            .map(|(&coords, &chip)| (coords - origin, chip))
            .collect();
        let wires = wires
            .iter()
            .map(|(&(coords, dir), &shape)| ((coords - origin, dir), shape))
            .collect();
        Selection { size: rect.size(), chips, wires }
    }

    pub fn from_clipboard(
        clipboard: &Clipboard,
        allowed: &ChipSet,
    ) -> Option<Selection> {
        if let Some(text) = clipboard.get() {
            match CircuitData::deserialize_from_string(&text) {
                Ok(data) => {
                    let chips = data
                        .chips
                        .iter()
                        .filter(|&(_, ctype, _)| allowed.contains(ctype))
                        .map(|(delta, ctype, orient)| (delta, (ctype, orient)))
                        .collect();
                    let wires = data
                        .wires
                        .iter()
                        .map(|(delta, dir, shape)| ((delta, dir), shape))
                        .collect();
                    let selection =
                        Selection { size: data.size, chips, wires };
                    return Some(selection);
                }
                Err(err) => {
                    debug_log!("Could not deserialize selection: {}", err);
                }
            }
        }
        return None;
    }

    pub fn size(&self) -> CoordsSize {
        self.size
    }

    pub fn draw_box(
        resources: &Resources,
        matrix: &Matrix4<f32>,
        selected_rect: CoordsRect,
        grid_cell_size: f32,
    ) {
        draw_selection_box(
            resources,
            matrix,
            selected_rect,
            grid_cell_size,
            vec2(0.0, 0.0),
        );
    }

    fn reorient(&mut self, reorient: Orientation) {
        let new_size = reorient * self.size;
        let new_chips = self
            .chips
            .iter()
            .map(|(&old_delta, &(ctype, old_orient))| {
                let new_delta = reorient
                    .transform_in_size(old_delta, self.size)
                    - reorient.transform_in_size(
                        vec2(0, 0),
                        old_orient * ctype.size(),
                    );
                let new_orient = reorient * old_orient;
                (new_delta, (ctype, new_orient))
            })
            .collect();
        let new_wires = self
            .wires
            .iter()
            .map(|(&(old_delta, old_dir), &shape)| {
                let new_delta =
                    reorient.transform_in_size(old_delta, self.size);
                let new_dir = reorient * old_dir;
                let new_shape = if reorient.is_mirrored() {
                    match shape {
                        WireShape::TurnLeft => WireShape::TurnRight,
                        WireShape::TurnRight => WireShape::TurnLeft,
                        WireShape::SplitLeft => WireShape::SplitRight,
                        WireShape::SplitRight => WireShape::SplitLeft,
                        other => other,
                    }
                } else {
                    shape
                };
                ((new_delta, new_dir), new_shape)
            })
            .collect();
        self.size = new_size;
        self.chips = new_chips;
        self.wires = new_wires;
    }

    fn to_clipboard_text(&self) -> Result<String, String> {
        let mut data = CircuitData::new(self.size.width, self.size.height);
        for (&delta, &(ctype, orient)) in self.chips.iter() {
            data.chips.insert(delta, ctype, orient);
        }
        for (&(delta, dir), &shape) in self.wires.iter() {
            data.wires.insert(delta, dir, shape);
        }
        data.serialize_to_string()
    }

    fn copy_to_clipboard(&self, clipboard: &mut Clipboard) {
        match self.to_clipboard_text() {
            Ok(text) => clipboard.set(&text),
            Err(err) => {
                debug_log!("Could not serialize selection: {}", err);
            }
        }
    }
}

//===========================================================================//

fn draw_selection_box(
    resources: &Resources,
    matrix: &Matrix4<f32>,
    selected_rect: CoordsRect,
    grid_cell_size: f32,
    delta: Vector2<f32>,
) {
    let ui = resources.shaders().ui();
    let rect = (selected_rect.as_f32() * grid_cell_size).expand(4.0)
        + delta * grid_cell_size;
    ui.draw_selection_box(
        matrix,
        &rect,
        &SELECTION_BOX_COLOR1,
        &SELECTION_BOX_COLOR2,
        &SELECTION_BOX_COLOR3,
    );
}

//===========================================================================//

pub fn flip_horz(grid: &mut EditGrid, selected_rect: CoordsRect) {
    reorient(grid, selected_rect, Orientation::default().flip_horz());
}

pub fn flip_vert(grid: &mut EditGrid, selected_rect: CoordsRect) {
    reorient(grid, selected_rect, Orientation::default().flip_vert());
}

pub fn rotate_ccw(grid: &mut EditGrid, selected_rect: CoordsRect) {
    reorient(grid, selected_rect, Orientation::default().rotate_ccw());
}

pub fn rotate_cw(grid: &mut EditGrid, selected_rect: CoordsRect) {
    reorient(grid, selected_rect, Orientation::default().rotate_cw());
}

fn reorient(
    grid: &mut EditGrid,
    selected_rect: CoordsRect,
    orient: Orientation,
) {
    let mut selection = cut_provisionally(grid, selected_rect);
    selection.reorient(orient);
    let changes =
        changes_for_paste(grid, &selection, selected_rect.top_left());
    if grid.try_mutate_provisionally(changes) {
        grid.commit_provisional_changes();
    } else {
        debug_warn!("reorient paste mutation failed");
        grid.roll_back_provisional_changes();
    }
}

//===========================================================================//

pub fn copy(
    grid: &EditGrid,
    selected_rect: CoordsRect,
    clipboard: &mut Clipboard,
) {
    let (_, selection) = changes_for_cut(grid, selected_rect);
    selection.copy_to_clipboard(clipboard);
}

pub fn cut(
    grid: &mut EditGrid,
    selected_rect: CoordsRect,
    clipboard: &mut Clipboard,
) {
    let (changes, selection) = changes_for_cut(grid, selected_rect);
    if !grid.try_mutate(changes) {
        debug_warn!("cut mutation failed");
    }
    selection.copy_to_clipboard(clipboard);
}

pub fn delete(grid: &mut EditGrid, selected_rect: CoordsRect) {
    let (changes, _) = changes_for_cut(grid, selected_rect);
    if !grid.try_mutate(changes) {
        debug_warn!("delete mutation failed");
    }
}

pub fn delete_wire(grid: &mut EditGrid, wire_index: usize) {
    let old_wires: HashMap<(Coords, Direction), WireShape> =
        grid.wire_fragments_for_wire_index(wire_index).collect();
    let new_wires = HashMap::<(Coords, Direction), WireShape>::new();
    let changes = vec![GridChange::ReplaceWires(old_wires, new_wires)];
    if !grid.try_mutate(changes) {
        debug_warn!("delete_wire mutation failed");
    }
}

pub fn cut_provisionally(
    grid: &mut EditGrid,
    selected_rect: CoordsRect,
) -> Selection {
    let (changes, selection) = changes_for_cut(grid, selected_rect);
    if !grid.try_mutate_provisionally(changes) {
        debug_warn!("cut_provisionally mutation failed");
    }
    selection
}

fn changes_for_cut(
    grid: &EditGrid,
    selected_rect: CoordsRect,
) -> (Vec<GridChange>, Selection) {
    let mut changes = Vec::<GridChange>::new();
    let mut chips = HashMap::<Coords, (ChipType, Orientation)>::new();
    let mut old_wires = HashMap::<(Coords, Direction), WireShape>::new();
    let mut new_wires = HashMap::<(Coords, Direction), WireShape>::new();
    let mut selection_wires = HashMap::<(Coords, Direction), WireShape>::new();
    for coords in selected_rect {
        if let Some((chip_coords, ctype, orient)) = grid.chip_at(coords) {
            if chip_coords == coords {
                let chip_size = orient * ctype.size();
                let chip_rect = Rect::with_size(chip_coords, chip_size);
                if selected_rect.contains_rect(chip_rect) {
                    changes.push(GridChange::RemoveChip(
                        chip_coords,
                        ctype,
                        orient,
                    ));
                    chips.insert(chip_coords, (ctype, orient));
                }
            }
        }
        for dir in Direction::all() {
            if let Some(shape) = grid.wire_shape_at(coords, dir) {
                if selected_rect.contains_point(coords + dir) {
                    selection_wires.insert((coords, dir), shape);
                    old_wires.insert((coords, dir), shape);
                } else if grid.wire_shape_at(coords + dir, -dir)
                    == Some(WireShape::Stub)
                {
                    selection_wires.insert((coords, dir), shape);
                    old_wires.insert((coords, dir), shape);
                    old_wires.insert((coords + dir, -dir), WireShape::Stub);
                } else if shape != WireShape::Stub {
                    selection_wires.insert((coords, dir), shape);
                    old_wires.insert((coords, dir), shape);
                    new_wires.insert((coords, dir), WireShape::Stub);
                }
            }
        }
    }
    let selection = Selection::new(selected_rect, &chips, &selection_wires);
    if !old_wires.is_empty() {
        changes.push(GridChange::ReplaceWires(old_wires, new_wires));
    }
    (changes, selection)
}

//===========================================================================//

fn changes_for_paste(
    grid: &EditGrid,
    selection: &Selection,
    top_left: Coords,
) -> Vec<GridChange> {
    let paste_rect = Rect::with_size(top_left, selection.size);
    let mut old_wires = HashMap::<(Coords, Direction), WireShape>::new();
    let mut new_wires = HashMap::<(Coords, Direction), WireShape>::new();
    for (&(delta, dir), &shape) in selection.wires.iter() {
        let coords = top_left + delta;
        if let Some(old_shape) = grid.wire_shape_at(coords, dir) {
            if old_shape == shape || shape == WireShape::Stub {
                continue;
            }
            old_wires.insert((coords, dir), old_shape);
            let others = match old_shape {
                WireShape::Stub => vec![],
                WireShape::Straight => vec![(-dir, WireShape::Straight)],
                WireShape::TurnLeft => {
                    vec![(dir.rotate_cw(), WireShape::TurnRight)]
                }
                WireShape::TurnRight => {
                    vec![(dir.rotate_ccw(), WireShape::TurnLeft)]
                }
                WireShape::SplitTee => vec![
                    (dir.rotate_ccw(), WireShape::SplitLeft),
                    (dir.rotate_cw(), WireShape::SplitRight),
                ],
                WireShape::SplitLeft => vec![
                    (dir.rotate_cw(), WireShape::SplitTee),
                    (-dir, WireShape::SplitRight),
                ],
                WireShape::SplitRight => vec![
                    (dir.rotate_ccw(), WireShape::SplitTee),
                    (-dir, WireShape::SplitLeft),
                ],
                WireShape::Cross => vec![
                    (dir.rotate_ccw(), WireShape::Cross),
                    (-dir, WireShape::Cross),
                    (dir.rotate_cw(), WireShape::Cross),
                ],
            };
            for (dir2, shape2) in others {
                if !old_wires.contains_key(&(coords, dir2)) {
                    debug_assert!(!new_wires.contains_key(&(coords, dir2)));
                    old_wires.insert((coords, dir2), shape2);
                    if grid.wire_shape_at(coords + dir2, -dir2)
                        == Some(WireShape::Stub)
                    {
                        old_wires
                            .insert((coords + dir2, -dir2), WireShape::Stub);
                    } else {
                        new_wires.insert((coords, dir2), WireShape::Stub);
                    }
                }
            }
        }
        new_wires.insert((coords, dir), shape);
        if !paste_rect.contains_point(coords + dir)
            && grid.wire_shape_at(coords + dir, -dir).is_none()
        {
            new_wires.insert((coords + dir, -dir), WireShape::Stub);
        }
    }

    let mut new_chips = HashMap::<Coords, (ChipType, Orientation)>::new();
    for (&delta, &(ctype, orient)) in selection.chips.iter() {
        let new_coords = top_left + delta;
        let ports: HashSet<(Coords, Direction)> = ctype
            .ports(new_coords, orient)
            .into_iter()
            .map(|port| (port.coords, port.dir))
            .collect();
        let new_rect = Rect::with_size(new_coords, orient * ctype.size());
        for coords in new_rect {
            for dir in Direction::all() {
                if let Some(shape) = grid.wire_shape_at(coords, dir) {
                    if new_rect.contains_point(coords + dir) {
                        old_wires.insert((coords, dir), shape);
                    } else if shape != WireShape::Stub {
                        old_wires.insert((coords, dir), shape);
                        if !ports.contains(&(coords, dir))
                            && grid.wire_shape_at(coords + dir, -dir)
                                == Some(WireShape::Stub)
                        {
                            old_wires
                                .insert((coords + dir, -dir), WireShape::Stub);
                        } else {
                            new_wires.insert((coords, dir), WireShape::Stub);
                        }
                    }
                }
            }
        }
        new_chips.insert(new_coords, (ctype, orient));
    }

    let mut changes = Vec::<GridChange>::new();
    if !old_wires.is_empty() || !new_wires.is_empty() {
        changes.push(GridChange::ReplaceWires(old_wires, new_wires));
    }
    for (coords, (ctype, orient)) in new_chips.into_iter() {
        changes.push(GridChange::AddChip(coords, ctype, orient));
    }
    changes
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{changes_for_cut, changes_for_paste, Selection};
    use cgmath::vec2;
    use std::collections::{HashMap, HashSet};
    use tachy::geom::{
        Coords, CoordsDelta, CoordsRect, CoordsSize, Direction, Orientation,
    };
    use tachy::save::{ChipType, CircuitData, Puzzle, WireShape};
    use tachy::state::EditGrid;

    #[test]
    fn cut_removes_edge_stub() {
        let mut data = CircuitData::new(10, 10);
        data.wires.insert(
            CoordsDelta::new(3, 5),
            Direction::East,
            WireShape::Stub,
        );
        data.wires.insert(
            CoordsDelta::new(4, 5),
            Direction::West,
            WireShape::Stub,
        );
        let mut grid = EditGrid::from_circuit_data(
            Puzzle::TutorialOr,
            &HashSet::new(),
            &data,
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(3, 5), Direction::East),
            Some(WireShape::Stub)
        );
        let rect = CoordsRect::new(4, 5, 1, 1);
        let (changes, selection) = changes_for_cut(&grid, rect);
        assert_eq!(selection.size, rect.size());
        assert!(grid.try_mutate(changes));
        assert_eq!(
            grid.wire_shape_at(Coords::new(3, 5), Direction::East),
            None
        );
    }

    #[test]
    fn paste_splices_wires() {
        let mut data = CircuitData::new(10, 10);
        data.wires.insert(
            CoordsDelta::new(3, 5),
            Direction::East,
            WireShape::Stub,
        );
        data.wires.insert(
            CoordsDelta::new(4, 5),
            Direction::West,
            WireShape::Straight,
        );
        data.wires.insert(
            CoordsDelta::new(4, 5),
            Direction::East,
            WireShape::Straight,
        );
        data.wires.insert(
            CoordsDelta::new(5, 5),
            Direction::West,
            WireShape::Stub,
        );
        let mut grid = EditGrid::from_circuit_data(
            Puzzle::TutorialOr,
            &HashSet::new(),
            &data,
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(4, 5), Direction::East),
            Some(WireShape::Straight)
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(5, 5), Direction::West),
            Some(WireShape::Stub)
        );
        let selection = Selection {
            size: CoordsSize::new(2, 1),
            chips: HashMap::new(),
            wires: vec![
                ((vec2(0, 0), Direction::East), WireShape::Stub),
                ((vec2(1, 0), Direction::West), WireShape::Straight),
                ((vec2(1, 0), Direction::East), WireShape::Straight),
            ]
            .into_iter()
            .collect(),
        };
        let changes = changes_for_paste(&grid, &selection, Coords::new(4, 5));
        assert!(grid.try_mutate(changes));
        assert_eq!(
            grid.wire_shape_at(Coords::new(4, 5), Direction::East),
            Some(WireShape::Straight)
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(5, 5), Direction::West),
            Some(WireShape::Straight)
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(5, 5), Direction::East),
            Some(WireShape::Straight)
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(6, 5), Direction::West),
            Some(WireShape::Stub)
        );
    }

    #[test]
    fn pasting_a_chip_removes_stubs_not_on_port() {
        let mut data = CircuitData::new(10, 10);
        data.wires.insert(
            CoordsDelta::new(1, 1),
            Direction::East,
            WireShape::TurnLeft,
        );
        let mut grid = EditGrid::from_circuit_data(
            Puzzle::TutorialOr,
            &HashSet::new(),
            &data,
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(1, 1), Direction::East),
            Some(WireShape::TurnLeft)
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(1, 1), Direction::South),
            Some(WireShape::TurnRight)
        );
        let selection = Selection {
            size: CoordsSize::new(1, 1),
            chips: vec![(vec2(0, 0), (ChipType::Not, Orientation::default()))]
                .into_iter()
                .collect(),
            wires: HashMap::new(),
        };
        let changes = changes_for_paste(&grid, &selection, Coords::new(1, 1));
        assert!(grid.try_mutate(changes));
        assert_eq!(
            grid.wire_shape_at(Coords::new(1, 1), Direction::East),
            Some(WireShape::Stub)
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(1, 1), Direction::South),
            None
        );
    }

    #[test]
    fn pasting_a_wire_removes_new_stubs() {
        let mut data = CircuitData::new(10, 10);
        data.wires.insert(
            CoordsDelta::new(1, 1),
            Direction::East,
            WireShape::TurnLeft,
        );
        let mut grid = EditGrid::from_circuit_data(
            Puzzle::TutorialOr,
            &HashSet::new(),
            &data,
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(1, 1), Direction::East),
            Some(WireShape::TurnLeft)
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(1, 1), Direction::South),
            Some(WireShape::TurnRight)
        );
        let selection = Selection {
            size: CoordsSize::new(1, 1),
            chips: HashMap::new(),
            wires: vec![
                ((vec2(0, 0), Direction::West), WireShape::TurnRight),
                ((vec2(0, 0), Direction::South), WireShape::TurnLeft),
            ]
            .into_iter()
            .collect(),
        };
        let changes = changes_for_paste(&grid, &selection, Coords::new(1, 1));
        assert!(grid.try_mutate(changes));
        assert_eq!(
            grid.wire_shape_at(Coords::new(1, 1), Direction::South),
            Some(WireShape::TurnLeft)
        );
        assert_eq!(
            grid.wire_shape_at(Coords::new(1, 1), Direction::East),
            None
        );
    }
}

//===========================================================================//
