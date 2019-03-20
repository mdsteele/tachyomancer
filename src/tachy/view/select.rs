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
use tachy::save::{ChipType, WireShape};
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
    selected_rect: CoordsRect,
    start_grid_pt: Point2<f32>,
    current_grid_pt: Point2<f32>,
    reorient: Orientation,
}

impl SelectionDrag {
    pub fn new(selected_rect: CoordsRect, grid_pt: Point2<f32>)
               -> SelectionDrag {
        SelectionDrag {
            selected_rect,
            start_grid_pt: grid_pt,
            current_grid_pt: grid_pt,
            reorient: Orientation::default(),
        }
    }

    pub fn reoriented_selected_rect(&self) -> CoordsRect {
        Rect::with_size(self.selected_rect.top_left(),
                        self.reorient * self.selected_rect.size())
    }

    pub fn delta(&self) -> Vector2<f32> {
        self.current_grid_pt - self.start_grid_pt
    }

    pub fn flip_horz(&mut self) { self.reorient = self.reorient.flip_horz(); }

    pub fn flip_vert(&mut self) { self.reorient = self.reorient.flip_vert(); }

    pub fn rotate_cw(&mut self) { self.reorient = self.reorient.rotate_cw(); }

    pub fn rotate_ccw(&mut self) {
        self.reorient = self.reorient.rotate_ccw();
    }

    pub fn move_to(&mut self, grid_pt: Point2<f32>) {
        self.current_grid_pt = grid_pt;
    }

    pub fn finish(self, grid: &mut EditGrid) -> CoordsRect {
        let drag_delta: CoordsDelta = self.delta().as_i32_round();
        let new_selected_rect: CoordsRect = self.reoriented_selected_rect() +
            drag_delta;
        if !grid.bounds().contains_rect(new_selected_rect) {
            return self.selected_rect;
        }
        let (changes, selection) = changes_for_cut(grid, self.selected_rect);
        let success = grid.try_mutate_then(changes, |grid| {
            let top_left = new_selected_rect.top_left();
            changes_for_paste(grid, &selection, self.reorient, top_left)
        });
        if success {
            return new_selected_rect;
        } else {
            return self.selected_rect;
        }
    }
}

//===========================================================================//

struct Selection {
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
}

//===========================================================================//

pub fn cut(grid: &mut EditGrid, selected_rect: CoordsRect) {
    let (changes, _selection) = changes_for_cut(grid, selected_rect);
    // TODO: save selection to clipboard
    grid.do_mutate(changes);
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
                     reorient: Orientation, top_left: Coords)
                     -> Vec<GridChange> {
    let paste_rect = Rect::with_size(top_left, reorient * selection.size);
    let mut old_wires = HashMap::<(Coords, Direction), WireShape>::new();
    let mut new_wires = HashMap::<(Coords, Direction), WireShape>::new();
    for (&(delta, dir), &shape) in selection.wires.iter() {
        let coords = top_left +
            reorient.transform_in_size(delta, selection.size);
        let dir = reorient * dir;
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
    for (&old_delta, &(ctype, old_orient)) in selection.chips.iter() {
        let new_coords = top_left +
            reorient.transform_in_size(old_delta, selection.size) -
            reorient.transform_in_size(vec2(0, 0), old_orient * ctype.size());
        let new_orient = reorient * old_orient;
        let new_rect = Rect::with_size(new_coords, new_orient * ctype.size());
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
        new_chips.insert(new_coords, (ctype, new_orient));
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
    use tachy::geom::{Coords, CoordsRect, CoordsSize, Direction, Orientation};
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
        let changes = changes_for_paste(&grid,
                                        &selection,
                                        Orientation::default(),
                                        Coords::new(4, 5));
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
