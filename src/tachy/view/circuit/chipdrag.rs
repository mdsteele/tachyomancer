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

use cgmath::Point2;
use std::collections::{HashMap, HashSet};
use tachy::geom::{AsFloat, AsInt, Coords, CoordsRect, Direction, Orientation};
use tachy::save::{ChipType, WireShape};
use tachy::state::{ChipExt, EditGrid, GridChange};

//===========================================================================//

pub struct ChipDrag {
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

    pub fn chip_type(&self) -> ChipType { self.chip_type }

    pub fn old_coords(&self) -> Option<Coords> { self.old_coords }

    pub fn new_orient(&self) -> Orientation { self.reorient * self.old_orient }

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

    pub fn cancel(self, grid: &mut EditGrid) -> bool {
        grid.roll_back_provisional_changes()
    }

    pub fn drop_onto_board(self, grid: &mut EditGrid) {
        let new_coords: Coords = self.chip_topleft().as_i32_round();
        let new_orient = self.reorient * self.old_orient;
        let new_size = new_orient * self.chip_type.size();
        let new_rect = CoordsRect::with_size(new_coords, new_size);
        if !grid.can_place_chip(new_rect) {
            grid.roll_back_provisional_changes();
            return;
        }
        let new_ports: HashSet<(Coords, Direction)> = self.chip_type
            .ports(new_coords, new_orient)
            .into_iter()
            .map(|port| (port.pos, port.dir))
            .collect();
        let mut changes = Vec::<GridChange>::new();
        let mut old_wires = HashMap::new();
        let mut new_wires = HashMap::new();
        for coords in new_rect {
            for dir in Direction::all() {
                if let Some(shape) = grid.wire_shape_at(coords, dir) {
                    let coords2 = coords + dir;
                    if new_rect.contains_point(coords2) {
                        old_wires.insert((coords, dir), shape);
                    } else if grid.wire_shape_at(coords2, -dir) ==
                               Some(WireShape::Stub) &&
                               !new_ports.contains(&(coords, dir))
                    {
                        old_wires.insert((coords, dir), shape);
                        old_wires.insert((coords2, -dir), WireShape::Stub);
                    } else if shape != WireShape::Stub {
                        old_wires.insert((coords, dir), shape);
                        new_wires.insert((coords, dir), WireShape::Stub);
                    }
                }
            }
        }
        if !old_wires.is_empty() {
            changes.push(GridChange::ReplaceWires(old_wires, new_wires));
        }
        changes.push(GridChange::AddChip(new_coords,
                                         self.chip_type,
                                         self.reorient * self.old_orient));
        if grid.try_mutate_provisionally(changes) {
            grid.commit_provisional_changes();
        } else {
            grid.roll_back_provisional_changes();
        }
    }

    pub fn drop_into_parts_tray(self, grid: &mut EditGrid) {
        grid.commit_provisional_changes();
    }
}

//===========================================================================//
