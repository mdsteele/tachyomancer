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

use cgmath::{MetricSpace, Point2, Vector2, vec2};
use std::collections::HashMap;
use tachy::geom::{AsFloat, AsInt, Coords, CoordsDelta, CoordsRect,
                  CoordsSize, Direction, Orientation, Rect};
use tachy::gui::Clipboard;
use tachy::save::{ChipType, CircuitData, Puzzle, WireShape};
use tachy::state::{EditGrid, GridChange};

//===========================================================================//

// How close, in grid cells, the mouse must be to a grid vertex to start a
// selection rect.
const SELECTING_VERTEX_MAX_DIST: f32 = 0.2;

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
        expanded.contains_point(coords) &&
            grid_pt.distance(coords.as_f32()) <= SELECTING_VERTEX_MAX_DIST
    }

    pub fn new(bounds: CoordsRect, start: Coords) -> SelectingDrag {
        let rect = Rect::new(start.x, start.y, 0, 0);
        SelectingDrag {
            bounds,
            start,
            rect,
        }
    }

    pub fn selected_rect(&self) -> CoordsRect { self.rect }

    pub fn move_to(&mut self, grid_pt: Point2<f32>) {
        let coords = grid_pt.as_i32_round();
        self.rect = Rect::new(self.start.x.min(coords.x),
                              self.start.y.min(coords.y),
                              (self.start.x - coords.x).abs(),
                              (self.start.y - coords.y).abs());
        self.rect = self.rect.intersection(self.bounds);
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
    pub fn new(selection: Selection, grab_rel: Vector2<f32>,
               grid_pt: Point2<f32>,
               original_selected_rect: Option<CoordsRect>)
               -> SelectionDrag {
        SelectionDrag {
            selection,
            grab_rel,
            grid_pt,
            original_selected_rect,
        }
    }

    pub fn selection_size(&self) -> CoordsSize { self.selection.size() }

    pub fn top_left_grid_pt(&self) -> Point2<f32> {
        self.grid_pt - self.grab_rel
    }

    pub fn move_to(&mut self, grid_pt: Point2<f32>) { self.grid_pt = grid_pt; }

    pub fn flip_horz(&mut self) {
        self.selection.reorient(Orientation::default().flip_horz());
    }

    pub fn flip_vert(&mut self) {
        self.selection.reorient(Orientation::default().flip_vert());
    }

    pub fn rotate_cw(&mut self) {
        self.selection.reorient(Orientation::default().rotate_cw());
    }

    pub fn rotate_ccw(&mut self) {
        self.selection.reorient(Orientation::default().rotate_ccw());
    }

    pub fn cancel(self, grid: &mut EditGrid) -> bool {
        grid.roll_back_provisional_changes()
    }

    pub fn finish(self, grid: &mut EditGrid) -> Option<CoordsRect> {
        let top_left_coords = self.top_left_grid_pt().as_i32_round();
        let changes =
            changes_for_paste(grid, &self.selection, top_left_coords);
        if grid.try_mutate_provisionally(changes) {
            grid.commit_provisional_changes();
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
    fn new(rect: CoordsRect,
           chips: &HashMap<Coords, (ChipType, Orientation)>,
           wires: &HashMap<(Coords, Direction), WireShape>)
           -> Selection {
        let origin = rect.top_left();
        let chips = chips
            .iter()
            .map(|(&coords, &chip)| (coords - origin, chip))
            .collect();
        let wires = wires
            .iter()
            .map(|(&(coords, dir), &shape)| ((coords - origin, dir), shape))
            .collect();
        Selection {
            size: rect.size(),
            chips,
            wires,
        }
    }

    pub fn from_clipboard(clipboard: &Clipboard, puzzle: Puzzle)
                          -> Option<Selection> {
        if let Some(text) = clipboard.get() {
            match CircuitData::deserialize_from_string(&text) {
                Ok(data) => {
                    let (left, top, width, height) = data.bounds;
                    let origin = Coords::new(left, top);
                    let chips = data.chips
                        .iter()
                        .filter(|(_, ctype, _)| ctype.is_allowed_in(puzzle))
                        .map(|(coords, ctype, orient)| {
                                 (coords - origin, (ctype, orient))
                             })
                        .collect();
                    let wires = data.wires
                        .iter()
                        .map(|(coords, dir, shape)| {
                                 ((coords - origin, dir), shape)
                             })
                        .collect();
                    let selection = Selection {
                        size: CoordsSize::new(width, height),
                        chips,
                        wires,
                    };
                    return Some(selection);
                }
                Err(err) => {
                    debug_log!("Could not deserialize selection: {}", err);
                }
            }
        }
        return None;
    }

    pub fn size(&self) -> CoordsSize { self.size }

    fn reorient(&mut self, reorient: Orientation) {
        let new_size = reorient * self.size;
        let new_chips = self.chips
            .iter()
            .map(|(&old_delta, &(ctype, old_orient))| {
                let new_delta = reorient
                    .transform_in_size(old_delta, self.size) -
                    reorient.transform_in_size(vec2(0, 0),
                                               old_orient * ctype.size());
                let new_orient = reorient * old_orient;
                (new_delta, (ctype, new_orient))
            })
            .collect();
        let new_wires = self.wires
            .iter()
            .map(|(&(old_delta, old_dir), &shape)| {
                let new_delta = reorient
                    .transform_in_size(old_delta, self.size);
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
        let origin = Coords::new(0, 0);
        let mut data = CircuitData::new(origin.x,
                                        origin.y,
                                        self.size.width,
                                        self.size.height);
        for (&delta, &(ctype, orient)) in self.chips.iter() {
            data.chips.insert(origin + delta, ctype, orient);
        }
        for (&(delta, dir), &shape) in self.wires.iter() {
            data.wires.insert(origin + delta, dir, shape);
        }
        data.serialize_to_string()
    }

    fn copy_to_clipboard(&self, clipboard: &Clipboard) {
        match self.to_clipboard_text() {
            Ok(text) => clipboard.set(&text),
            Err(err) => {
                debug_log!("Could not serialize selection: {}", err);
            }
        }
    }
}

//===========================================================================//

pub fn copy(grid: &EditGrid, selected_rect: CoordsRect,
            clipboard: &Clipboard) {
    let (_, selection) = changes_for_cut(grid, selected_rect);
    selection.copy_to_clipboard(clipboard);
}

pub fn cut(grid: &mut EditGrid, selected_rect: CoordsRect,
           clipboard: &Clipboard) {
    let (changes, selection) = changes_for_cut(grid, selected_rect);
    if !grid.try_mutate(changes) {
        debug_log!("WARNING: cut mutation failed");
    }
    selection.copy_to_clipboard(clipboard);
}

pub fn cut_provisionally(grid: &mut EditGrid, selected_rect: CoordsRect)
                         -> Selection {
    let (changes, selection) = changes_for_cut(grid, selected_rect);
    if !grid.try_mutate_provisionally(changes) {
        debug_log!("WARNING: cut_provisionally mutation failed");
    }
    selection
}

fn changes_for_cut(grid: &EditGrid, selected_rect: CoordsRect)
                   -> (Vec<GridChange>, Selection) {
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
                    changes.push(GridChange::RemoveChip(chip_coords,
                                                        ctype,
                                                        orient));
                    chips.insert(chip_coords, (ctype, orient));
                }
            }
        }
        for dir in Direction::all() {
            if let Some(shape) = grid.wire_shape_at(coords, dir) {
                if selected_rect.contains_point(coords + dir) {
                    selection_wires.insert((coords, dir), shape);
                    old_wires.insert((coords, dir), shape);
                } else if grid.wire_shape_at(coords + dir, -dir) ==
                           Some(WireShape::Stub)
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

fn changes_for_paste(grid: &EditGrid, selection: &Selection,
                     top_left: Coords)
                     -> Vec<GridChange> {
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
                WireShape::SplitTee => {
                    vec![
                        (dir.rotate_ccw(), WireShape::SplitLeft),
                        (dir.rotate_cw(), WireShape::SplitRight),
                    ]
                }
                WireShape::SplitLeft => {
                    vec![
                        (dir.rotate_cw(), WireShape::SplitTee),
                        (-dir, WireShape::SplitRight),
                    ]
                }
                WireShape::SplitRight => {
                    vec![
                        (dir.rotate_ccw(), WireShape::SplitTee),
                        (-dir, WireShape::SplitLeft),
                    ]
                }
                WireShape::Cross => {
                    vec![
                        (dir.rotate_ccw(), WireShape::Cross),
                        (-dir, WireShape::Cross),
                        (dir.rotate_cw(), WireShape::Cross),
                    ]
                }
            };
            for (dir2, shape2) in others {
                if !old_wires.contains_key(&(coords, dir2)) {
                    debug_assert!(!new_wires.contains_key(&(coords, dir2)));
                    old_wires.insert((coords, dir2), shape2);
                    new_wires.insert((coords, dir2), WireShape::Stub);
                }
            }
        }
        new_wires.insert((coords, dir), shape);
        if !paste_rect.contains_point(coords + dir) &&
            grid.wire_shape_at(coords + dir, -dir).is_none()
        {
            new_wires.insert((coords + dir, -dir), WireShape::Stub);
        }
    }

    let mut new_chips = HashMap::<Coords, (ChipType, Orientation)>::new();
    for (&delta, &(ctype, orient)) in selection.chips.iter() {
        let new_coords = top_left + delta;
        let new_rect = Rect::with_size(new_coords, orient * ctype.size());
        for coords in new_rect {
            for dir in Direction::all() {
                if let Some(shape) = grid.wire_shape_at(coords, dir) {
                    // TODO: Remove lone stubs that aren't connected to a port
                    if new_rect.contains_point(coords + dir) {
                        old_wires.insert((coords, dir), shape);
                    } else if shape != WireShape::Stub {
                        old_wires.insert((coords, dir), shape);
                        new_wires.insert((coords, dir), WireShape::Stub);
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
    use super::{Selection, changes_for_cut, changes_for_paste};
    use cgmath::vec2;
    use std::collections::HashMap;
    use tachy::geom::{Coords, CoordsRect, CoordsSize, Direction};
    use tachy::save::{CircuitData, Puzzle, WireShape};
    use tachy::state::EditGrid;

    #[test]
    fn cut_removes_edge_stub() {
        let mut data = CircuitData::new(0, 0, 10, 10);
        data.wires.insert(Coords::new(3, 5), Direction::East, WireShape::Stub);
        data.wires.insert(Coords::new(4, 5), Direction::West, WireShape::Stub);
        let mut grid = EditGrid::from_circuit_data(Puzzle::TutorialOr, &data);
        assert_eq!(grid.wire_shape_at(Coords::new(3, 5), Direction::East),
                   Some(WireShape::Stub));
        let rect = CoordsRect::new(4, 5, 1, 1);
        let (changes, selection) = changes_for_cut(&grid, rect);
        assert_eq!(selection.size, rect.size());
        assert!(grid.try_mutate(changes));
        assert_eq!(grid.wire_shape_at(Coords::new(3, 5), Direction::East),
                   None);
    }

    #[test]
    fn paste_splices_wires() {
        let mut data = CircuitData::new(0, 0, 10, 10);
        data.wires.insert(Coords::new(3, 5), Direction::East, WireShape::Stub);
        data.wires
            .insert(Coords::new(4, 5), Direction::West, WireShape::Straight);
        data.wires
            .insert(Coords::new(4, 5), Direction::East, WireShape::Straight);
        data.wires.insert(Coords::new(5, 5), Direction::West, WireShape::Stub);
        let mut grid = EditGrid::from_circuit_data(Puzzle::TutorialOr, &data);
        assert_eq!(grid.wire_shape_at(Coords::new(4, 5), Direction::East),
                   Some(WireShape::Straight));
        assert_eq!(grid.wire_shape_at(Coords::new(5, 5), Direction::West),
                   Some(WireShape::Stub));
        let selection = Selection {
            size: CoordsSize::new(2, 1),
            chips: HashMap::new(),
            wires: vec![
                ((vec2(0, 0), Direction::East), WireShape::Stub),
                ((vec2(1, 0), Direction::West), WireShape::Straight),
                ((vec2(1, 0), Direction::East), WireShape::Straight),
            ].into_iter()
                .collect(),
        };
        let changes = changes_for_paste(&grid, &selection, Coords::new(4, 5));
        assert!(grid.try_mutate(changes));
        assert_eq!(grid.wire_shape_at(Coords::new(4, 5), Direction::East),
                   Some(WireShape::Straight));
        assert_eq!(grid.wire_shape_at(Coords::new(5, 5), Direction::West),
                   Some(WireShape::Straight));
        assert_eq!(grid.wire_shape_at(Coords::new(5, 5), Direction::East),
                   Some(WireShape::Straight));
        assert_eq!(grid.wire_shape_at(Coords::new(6, 5), Direction::West),
                   Some(WireShape::Stub));
    }

    // TODO: more tests
}

//===========================================================================//