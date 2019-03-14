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
use std::collections::{HashMap, HashSet};
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
    let mut wires = HashMap::<(Coords, Direction), WireShape>::new();
    let mut needs_mass_remove = false;
    let mut extra_stubs = HashSet::<(Coords, Direction)>::new();
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
                wires.insert((coords, dir), shape);
                let coords2 = coords + dir;
                let dir2 = -dir;
                let on_edge = !selected_rect.contains_point(coords2);
                needs_mass_remove = needs_mass_remove || !on_edge ||
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
    let selection = Selection::new(selected_rect, &chips, &wires);
    if needs_mass_remove {
        changes.push(GridChange::MassRemoveWires(selected_rect, wires));
    }
    for (coords, dir) in extra_stubs {
        changes.push(GridChange::RemoveStubWire(coords, dir));
    }
    (changes, selection)
}

//===========================================================================//

fn changes_for_paste(grid: &EditGrid, selection: &Selection,
                     reorient: Orientation, top_left: Coords)
                     -> Vec<GridChange> {
    let mut changes = Vec::<GridChange>::new();

    // Place chips:
    for (&old_delta, &(ctype, old_orient)) in selection.chips.iter() {
        let new_coords = top_left +
            reorient.transform_in_size(old_delta, selection.size) -
            reorient.transform_in_size(vec2(0, 0), old_orient * ctype.size());
        let new_orient = reorient * old_orient;
        let new_rect = Rect::with_size(new_coords, new_orient * ctype.size());

        // Remove wires from under new chip location:
        // TODO: eliminate code duplication with drop_onto_board
        let mut needs_mass_remove = false;
        let mut wires = HashMap::<(Coords, Direction), WireShape>::new();
        for coords in new_rect {
            for dir in Direction::all() {
                if let Some(shape) = grid.wire_shape_at(coords, dir) {
                    wires.insert((coords, dir), shape);
                    needs_mass_remove = needs_mass_remove ||
                        shape != WireShape::Stub ||
                        new_rect.contains_point(coords + dir);
                }
            }
        }
        if needs_mass_remove {
            changes.push(GridChange::MassRemoveWires(new_rect, wires));
        }

        changes.push(GridChange::AddChip(new_coords, ctype, new_orient));
    }

    // Place wires:
    let paste_rect = Rect::with_size(top_left, reorient * selection.size);
    let new_wires: HashMap<(Coords, Direction), WireShape> = selection
        .wires
        .iter()
        .map(|(&(delta, dir), &shape)| {
                 ((top_left +
                       reorient.transform_in_size(delta, selection.size),
                   reorient * dir),
                  shape)
             })
        .collect();
    let mut new_edge_stubs = HashSet::<(Coords, Direction)>::new();
    let mut old_wires = HashMap::<(Coords, Direction), WireShape>::new();
    let mut needs_mass_remove = false;
    for (&(coords, dir), _) in new_wires.iter() {
        if let Some(old_shape) = grid.wire_shape_at(coords, dir) {
            old_wires.insert((coords, dir), old_shape);
            needs_mass_remove = needs_mass_remove ||
                old_shape != WireShape::Stub ||
                paste_rect.contains_point(coords + dir);
        } else if !paste_rect.contains_point(coords + dir) {
            new_edge_stubs.insert((coords, dir));
        }
    }
    // TODO fix up old_wires
    if needs_mass_remove {
        changes.push(GridChange::MassRemoveWires(paste_rect, old_wires));
    }
    for (coords, dir) in new_edge_stubs {
        changes.push(GridChange::AddStubWire(coords, dir));
    }
    changes.push(GridChange::MassAddWires(paste_rect, new_wires));

    changes
}

//===========================================================================//
