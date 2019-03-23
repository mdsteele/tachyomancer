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

use super::check::{self, WireColor, WireError, WireInfo};
use super::chip::{ChipExt, new_chip_evals};
use super::eval::{ChipEval, CircuitEval, CircuitInteraction};
use super::port::{PortColor, PortConstraint, PortDependency, PortFlow};
use super::puzzle::{Interface, new_puzzle_eval, puzzle_interfaces};
use super::size::WireSize;
use std::collections::{HashMap, hash_map};
use std::mem;
use std::time::{Duration, Instant};
use std::usize;
use tachy::geom::{Coords, CoordsDelta, CoordsRect, Direction, Orientation,
                  Rect};
use tachy::save::{ChipType, CircuitData, Puzzle, WireShape};

//===========================================================================//

#[derive(Clone, Copy, Debug)]
enum ChipCell {
    /// A chip.
    Chip(ChipType, Orientation),
    /// For chips larger than 1x1, cells other than the top-left corner use
    /// Ref with the delta to the top-left corner.
    Ref(CoordsDelta),
}

//===========================================================================//

#[derive(Debug)]
pub enum GridChange {
    /// Adds a wire stub between two adjacent cells.
    AddStubWire(Coords, Direction),
    /// Removes a wire stub between two adjacent cells.
    RemoveStubWire(Coords, Direction),
    /// Toggles whether two edges of a cell are connected.
    ToggleCenterWire(Coords, Direction, Direction),
    /// Toggles whether a wire is connected to the split in the middle of a
    /// cell.
    ToggleSplitWire(Coords, Direction),
    /// Toggles a cell between a four-way split and an overpass/underpass.
    ToggleCrossWire(Coords),
    ReplaceWires(HashMap<(Coords, Direction), WireShape>,
                 HashMap<(Coords, Direction), WireShape>),
    /// Places a chip onto the board.
    AddChip(Coords, ChipType, Orientation),
    /// Removes a chip from the board.
    RemoveChip(Coords, ChipType, Orientation),
    /// Change the bounds rect from the first rect to the second.
    SetBounds(CoordsRect, CoordsRect),
}

impl GridChange {
    pub fn invert_group(changes: Vec<GridChange>) -> Vec<GridChange> {
        changes.into_iter().rev().map(GridChange::invert).collect()
    }

    pub fn invert(self) -> GridChange {
        match self {
            GridChange::AddStubWire(c, d) => GridChange::RemoveStubWire(c, d),
            GridChange::RemoveStubWire(c, d) => GridChange::AddStubWire(c, d),
            GridChange::ReplaceWires(old, new) => {
                GridChange::ReplaceWires(new, old)
            }
            GridChange::AddChip(c, t, o) => GridChange::RemoveChip(c, t, o),
            GridChange::RemoveChip(c, t, o) => GridChange::AddChip(c, t, o),
            GridChange::SetBounds(old, new) => GridChange::SetBounds(new, old),
            other => other,
        }
    }
}

//===========================================================================//

pub struct EditGrid {
    puzzle: Puzzle,
    bounds: CoordsRect,
    interfaces: &'static [Interface],
    fragments: HashMap<(Coords, Direction), (WireShape, usize)>,
    chips: HashMap<Coords, ChipCell>,
    wires: Vec<WireInfo>,
    wires_for_ports: HashMap<(Coords, Direction), usize>,
    wire_groups: Vec<Vec<usize>>,
    errors: Vec<WireError>,
    eval: Option<CircuitEval>,
    undo_stack: Vec<Vec<GridChange>>,
    redo_stack: Vec<Vec<GridChange>>,
    provisional_changes: Vec<GridChange>,
    modified_since: Option<Instant>,
}

impl EditGrid {
    pub fn new(puzzle: Puzzle) -> EditGrid {
        let mut grid = EditGrid {
            puzzle,
            bounds: Rect::new(-4, -3, 8, 6),
            interfaces: puzzle_interfaces(puzzle),
            fragments: HashMap::new(),
            chips: HashMap::new(),
            wires: Vec::new(),
            wires_for_ports: HashMap::new(),
            wire_groups: Vec::new(),
            errors: Vec::new(),
            eval: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            provisional_changes: Vec::new(),
            modified_since: None,
        };
        grid.typecheck_wires();
        grid
    }

    pub fn from_circuit_data(puzzle: Puzzle, data: &CircuitData) -> EditGrid {
        let mut grid = EditGrid {
            puzzle,
            bounds: Rect::new(data.bounds.0,
                              data.bounds.1,
                              data.bounds.2,
                              data.bounds.3),
            interfaces: puzzle_interfaces(puzzle),
            fragments: HashMap::new(),
            chips: HashMap::new(),
            wires: Vec::new(),
            wires_for_ports: HashMap::new(),
            wire_groups: Vec::new(),
            errors: Vec::new(),
            eval: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            provisional_changes: Vec::new(),
            modified_since: None,
        };

        // Chips:
        for (coords, ctype, orient) in data.chips.iter() {
            let change = GridChange::AddChip(coords, ctype, orient);
            if !grid.mutate_one(&change) {
                debug_log!("from_circuit_data: {:?} had no effect", change);
            }
        }

        // Wires:
        for (coords, dir, shape) in data.wires.iter() {
            if grid.has_frag(coords, dir) {
                continue;
            }
            match shape {
                WireShape::Stub => {
                    grid.set_frag(coords, dir, WireShape::Stub);
                }
                WireShape::Straight => {
                    if !grid.has_frag(coords, -dir) {
                        grid.set_frag(coords, dir, WireShape::Straight);
                        grid.set_frag(coords, -dir, WireShape::Straight);
                    }
                }
                WireShape::TurnLeft => {
                    let dir2 = dir.rotate_cw();
                    if !grid.has_frag(coords, dir2) {
                        grid.set_frag(coords, dir, WireShape::TurnLeft);
                        grid.set_frag(coords, dir2, WireShape::TurnRight);
                    }
                }
                WireShape::TurnRight => {
                    let dir2 = dir.rotate_ccw();
                    if !grid.has_frag(coords, dir2) {
                        grid.set_frag(coords, dir, WireShape::TurnRight);
                        grid.set_frag(coords, dir2, WireShape::TurnLeft);
                    }
                }
                WireShape::SplitTee => {
                    let dir2 = dir.rotate_cw();
                    let dir3 = dir.rotate_ccw();
                    if !grid.has_frag(coords, dir2) &&
                        !grid.has_frag(coords, dir3)
                    {
                        grid.set_frag(coords, dir, WireShape::SplitTee);
                        grid.set_frag(coords, dir2, WireShape::SplitRight);
                        grid.set_frag(coords, dir3, WireShape::SplitLeft);
                    }
                }
                WireShape::SplitLeft => {
                    let dir2 = dir.rotate_cw();
                    let dir3 = -dir;
                    if !grid.has_frag(coords, dir2) &&
                        !grid.has_frag(coords, dir3)
                    {
                        grid.set_frag(coords, dir, WireShape::SplitLeft);
                        grid.set_frag(coords, dir2, WireShape::SplitTee);
                        grid.set_frag(coords, dir3, WireShape::SplitRight);
                    }
                }
                WireShape::SplitRight => {
                    let dir2 = dir.rotate_ccw();
                    let dir3 = -dir;
                    if !grid.has_frag(coords, dir2) &&
                        !grid.has_frag(coords, dir3)
                    {
                        grid.set_frag(coords, dir, WireShape::SplitRight);
                        grid.set_frag(coords, dir2, WireShape::SplitTee);
                        grid.set_frag(coords, dir3, WireShape::SplitLeft);
                    }
                }
                WireShape::Cross => {
                    if !grid.has_frag(coords, -dir) &&
                        !grid.has_frag(coords, dir.rotate_cw()) &&
                        !grid.has_frag(coords, dir.rotate_ccw())
                    {
                        for d in Direction::all() {
                            grid.set_frag(coords, d, WireShape::Cross);
                        }
                    }
                }
            }
        }

        // Repair broken fragments:
        let mut missing = Vec::new();
        for (&(coords, dir), _) in grid.fragments.iter() {
            let loc = (coords + dir, -dir);
            if !grid.fragments.contains_key(&loc) {
                missing.push(loc);
            }
        }
        for loc in missing {
            grid.fragments.insert(loc, (WireShape::Stub, usize::MAX));
        }

        grid.typecheck_wires();
        grid
    }

    pub fn to_circuit_data(&self) -> CircuitData {
        let mut data = CircuitData::new(self.bounds.x,
                                        self.bounds.y,
                                        self.bounds.width,
                                        self.bounds.height);
        for (coords, ctype, orient) in self.chips() {
            data.chips.insert(coords, ctype, orient);
        }
        for (&(coords, dir), &(shape, _)) in self.fragments.iter() {
            match (shape, dir) {
                (WireShape::Stub, _) => {
                    // Exclude stubs that can be inferred in from_circuit_data.
                    match self.fragments.get(&(coords + dir, -dir)) {
                        Some(&(WireShape::Stub, _)) => {
                            match dir {
                                Direction::East | Direction::South => {}
                                Direction::West | Direction::North => continue,
                            }
                        }
                        Some(&(_, _)) => continue,
                        None => unreachable!(),
                    }
                }
                (WireShape::Straight, Direction::East) |
                (WireShape::Straight, Direction::South) |
                (WireShape::TurnLeft, _) |
                (WireShape::SplitTee, _) |
                (WireShape::Cross, Direction::East) => {}
                (WireShape::Straight, Direction::West) |
                (WireShape::Straight, Direction::North) |
                (WireShape::TurnRight, _) |
                (WireShape::SplitLeft, _) |
                (WireShape::SplitRight, _) |
                (WireShape::Cross, _) => continue,
            }
            data.wires.insert(coords, dir, shape);
        }
        data
    }

    pub fn is_modified(&self) -> bool { self.modified_since.is_some() }

    pub fn has_been_modified_for_at_least(&self, duration: Duration) -> bool {
        match self.modified_since {
            None => false,
            Some(instant) => instant.elapsed() >= duration,
        }
    }

    pub fn mark_modified(&mut self) {
        if self.modified_since.is_none() {
            self.modified_since = Some(Instant::now());
        }
    }

    pub fn mark_unmodified(&mut self) { self.modified_since = None; }

    pub fn puzzle(&self) -> Puzzle { self.puzzle }

    pub fn bounds(&self) -> CoordsRect { self.bounds }

    pub fn can_have_bounds(&self, bounds: CoordsRect) -> bool {
        for &coords in self.chips.keys() {
            if !bounds.contains_point(coords) {
                return false;
            }
        }
        for (&(coords, dir), &(shape, _)) in self.fragments.iter() {
            if !bounds.contains_point(coords) &&
                (shape != WireShape::Stub ||
                     !bounds.contains_point(coords + dir))
            {
                return false;
            }
        }
        return true;
    }

    pub fn interfaces(&self) -> &[Interface] { &self.interfaces }

    /// Returns an interator over `(Coords, ChipType, Orientation)` tuples.
    pub fn chips(&self) -> ChipsIter { ChipsIter { inner: self.chips.iter() } }

    /// Returns an interator over `(Coords, Direction, WireShape, WireSize,
    /// WireColor)` tuples.
    pub fn wire_fragments(&self) -> WireFragmentsIter {
        WireFragmentsIter {
            inner: self.fragments.iter(),
            wires: &self.wires,
        }
    }

    pub fn chip_at(&self, coords: Coords)
                   -> Option<(Coords, ChipType, Orientation)> {
        match self.chips.get(&coords) {
            Some(&ChipCell::Chip(ctype, orient)) => {
                Some((coords, ctype, orient))
            }
            Some(&ChipCell::Ref(delta)) => {
                let new_coords = coords + delta;
                match self.chips.get(&new_coords) {
                    Some(&ChipCell::Chip(ctype, orient)) => {
                        Some((new_coords, ctype, orient))
                    }
                    other => {
                        panic!("ChipRef({:?}) at {:?} points to {:?} at {:?}",
                               delta,
                               coords,
                               other,
                               new_coords);
                    }
                }
            }
            None => None,
        }
    }

    pub fn interface_at(&self, coords: Coords)
                        -> Option<(usize, &'static Interface)> {
        for (index, interface) in self.interfaces.iter().enumerate() {
            let rect = Rect::with_size(interface.top_left(self.bounds),
                                       interface.size());
            if rect.contains_point(coords) {
                return Some((index, interface));
            }
        }
        return None;
    }

    pub fn wire_index_at(&self, coords: Coords, dir: Direction)
                         -> Option<usize> {
        self.fragments.get(&(coords, dir)).map(|&(_, index)| index)
    }

    pub fn wire_shape_at(&self, coords: Coords, dir: Direction)
                         -> Option<WireShape> {
        self.fragments.get(&(coords, dir)).map(|&(shape, _)| shape)
    }

    pub fn can_place_chip(&self, new_rect: CoordsRect) -> bool {
        if !self.bounds.contains_rect(new_rect) {
            return false;
        }
        for coords in new_rect {
            if self.chips.contains_key(&coords) {
                return false;
            }
        }
        return true;
    }

    pub fn undo(&mut self) {
        self.roll_back_provisional_changes();
        if let Some(changes) = self.undo_stack.pop() {
            for change in changes.iter() {
                if !self.mutate_one(change) {
                    debug_log!("WARNING: undo {:?} had no effect", change);
                }
            }
            self.redo_stack.push(GridChange::invert_group(changes));
            self.typecheck_wires();
            self.mark_modified();
        }
    }

    pub fn redo(&mut self) {
        if let Some(changes) = self.redo_stack.pop() {
            debug_assert!(self.provisional_changes.is_empty());
            for change in changes.iter() {
                if !self.mutate_one(change) {
                    debug_log!("WARNING: redo {:?} had no effect", change);
                }
            }
            self.undo_stack.push(GridChange::invert_group(changes));
            self.typecheck_wires();
            self.mark_modified();
        }
    }

    #[must_use = "must not ignore try_mutate failure"]
    pub fn try_mutate(&mut self, changes: Vec<GridChange>) -> bool {
        self.commit_provisional_changes();
        if let Some(changes) = self.try_mutate_internal(changes) {
            self.undo_stack.push(GridChange::invert_group(changes));
            true
        } else {
            false
        }
    }

    #[must_use = "must not ignore try_mutate_provisionally failure"]
    pub fn try_mutate_provisionally(&mut self, changes: Vec<GridChange>)
                                    -> bool {
        if let Some(mut changes) = self.try_mutate_internal(changes) {
            self.provisional_changes.append(&mut changes);
            true
        } else {
            false
        }
    }

    pub fn commit_provisional_changes(&mut self) -> bool {
        if self.provisional_changes.is_empty() {
            return false;
        }
        debug_assert!(self.redo_stack.is_empty());
        let changes = mem::replace(&mut self.provisional_changes, Vec::new());
        self.undo_stack.push(GridChange::invert_group(changes));
        return true;
    }

    pub fn roll_back_provisional_changes(&mut self) -> bool {
        if self.provisional_changes.is_empty() {
            return false;
        }
        debug_assert!(self.redo_stack.is_empty());
        let changes = mem::replace(&mut self.provisional_changes, Vec::new());
        for change in GridChange::invert_group(changes) {
            if !self.mutate_one(&change) {
                debug_log!("WARNING: failed to roll back {:?}", change);
            }
        }
        self.typecheck_wires();
        return true;
    }

    #[must_use = "must not ignore try_mutate_internal result"]
    fn try_mutate_internal(&mut self, mut changes: Vec<GridChange>)
                           -> Option<Vec<GridChange>> {
        let num_changes: usize = changes.len();
        let mut num_succeeded: usize = 0;
        for change in changes.iter() {
            if !self.mutate_one(change) {
                break;
            }
            num_succeeded += 1;
        }
        let changes = if num_succeeded < num_changes {
            changes.truncate(num_succeeded);
            for change in GridChange::invert_group(changes) {
                if !self.mutate_one(&change) {
                    debug_log!("WARNING: failed to roll back {:?}", change);
                }
            }
            None
        } else {
            self.redo_stack.clear();
            self.mark_modified();
            Some(changes)
        };
        self.typecheck_wires();
        changes
    }

    #[must_use = "must not ignore mutate_one failure"]
    fn mutate_one(&mut self, change: &GridChange) -> bool {
        match *change {
            GridChange::AddStubWire(coords, dir) => {
                let coords2 = coords + dir;
                // Don't allow creating a stub outside of the bounds.
                if !self.bounds.contains_point(coords) &&
                    !self.bounds.contains_point(coords2)
                {
                    return false;
                }
                // Fail if there's already a wire there.
                if self.wire_shape_at(coords, dir).is_some() {
                    return false;
                }
                // Don't allow creating a stub completely under a large chip.
                match (self.chip_at(coords), self.chip_at(coords2)) {
                    (Some((c1, _, _)), Some((c2, _, _))) if c1 == c2 => {
                        return false;
                    }
                    _ => {}
                }
                // Add the stub.
                self.set_frag(coords, dir, WireShape::Stub);
                self.set_frag(coords2, -dir, WireShape::Stub);
            }
            GridChange::RemoveStubWire(coords, dir) => {
                let coords2 = coords + dir;
                let dir2 = -dir;
                if self.wire_shape_at(coords, dir) != Some(WireShape::Stub) ||
                    self.wire_shape_at(coords2, dir2) !=
                        Some(WireShape::Stub)
                {
                    return false;
                }
                self.fragments.remove(&(coords, dir));
                self.fragments.remove(&(coords2, dir2));
            }
            GridChange::ToggleCenterWire(coords, dir1, dir2) => {
                if !self.bounds.contains_point(coords) ||
                    self.chips.contains_key(&coords)
                {
                    return false;
                }
                match (self.wire_shape_at(coords, dir1),
                         self.wire_shape_at(coords, dir2)) {
                    (Some(WireShape::Stub), Some(WireShape::Stub)) => {
                        if dir1 == -dir2 {
                            self.set_frag(coords, dir1, WireShape::Straight);
                            self.set_frag(coords, dir2, WireShape::Straight);
                        } else if dir1 == dir2.rotate_cw() {
                            self.set_frag(coords, dir1, WireShape::TurnRight);
                            self.set_frag(coords, dir2, WireShape::TurnLeft);
                        } else if dir1 == dir2.rotate_ccw() {
                            self.set_frag(coords, dir1, WireShape::TurnLeft);
                            self.set_frag(coords, dir2, WireShape::TurnRight);
                        }
                    }
                    (Some(WireShape::Straight), Some(WireShape::Straight)) => {
                        if dir1 == -dir2 {
                            self.set_frag(coords, dir1, WireShape::Stub);
                            self.set_frag(coords, dir2, WireShape::Stub);
                        }
                    }
                    (Some(WireShape::TurnRight),
                     Some(WireShape::TurnLeft)) => {
                        if dir1 == dir2.rotate_cw() {
                            self.set_frag(coords, dir1, WireShape::Stub);
                            self.set_frag(coords, dir2, WireShape::Stub);
                        }
                    }
                    (Some(WireShape::TurnLeft),
                     Some(WireShape::TurnRight)) => {
                        if dir1 == dir2.rotate_ccw() {
                            self.set_frag(coords, dir1, WireShape::Stub);
                            self.set_frag(coords, dir2, WireShape::Stub);
                        }
                    }
                    (_, _) => return false,
                }
            }
            GridChange::ToggleSplitWire(coords, dir) => {
                if !self.bounds.contains_point(coords) ||
                    self.chips.contains_key(&coords)
                {
                    return false;
                }
                match (self.wire_shape_at(coords, dir),
                         self.wire_shape_at(coords, -dir),
                         self.wire_shape_at(coords, dir.rotate_cw())) {
                    (Some(WireShape::Stub), Some(WireShape::SplitTee), _) => {
                        for dir in Direction::all() {
                            self.set_frag(coords, dir, WireShape::Cross);
                        }
                    }
                    (Some(WireShape::Cross), _, _) => {
                        self.set_frag(coords, dir, WireShape::Stub);
                        self.set_frag(coords, -dir, WireShape::SplitTee);
                        self.set_frag(coords,
                                      dir.rotate_cw(),
                                      WireShape::SplitLeft);
                        self.set_frag(coords,
                                      dir.rotate_ccw(),
                                      WireShape::SplitRight);
                    }
                    (Some(WireShape::Stub), Some(WireShape::TurnLeft), _) => {
                        self.set_frag(coords, dir, WireShape::SplitRight);
                        self.set_frag(coords, -dir, WireShape::SplitLeft);
                        self.set_frag(coords,
                                      dir.rotate_ccw(),
                                      WireShape::SplitTee);
                    }
                    (Some(WireShape::SplitRight), _, _) => {
                        self.set_frag(coords, dir, WireShape::Stub);
                        self.set_frag(coords, -dir, WireShape::TurnLeft);
                        self.set_frag(coords,
                                      dir.rotate_ccw(),
                                      WireShape::TurnRight);
                    }
                    (Some(WireShape::Stub), Some(WireShape::TurnRight), _) => {
                        self.set_frag(coords, dir, WireShape::SplitLeft);
                        self.set_frag(coords, -dir, WireShape::SplitRight);
                        self.set_frag(coords,
                                      dir.rotate_cw(),
                                      WireShape::SplitTee);
                    }
                    (Some(WireShape::SplitLeft), _, _) => {
                        self.set_frag(coords, dir, WireShape::Stub);
                        self.set_frag(coords, -dir, WireShape::TurnRight);
                        self.set_frag(coords,
                                      dir.rotate_cw(),
                                      WireShape::TurnLeft);
                    }
                    (Some(WireShape::Stub), _, Some(WireShape::Straight)) => {
                        self.set_frag(coords, dir, WireShape::SplitTee);
                        self.set_frag(coords,
                                      dir.rotate_cw(),
                                      WireShape::SplitRight);
                        self.set_frag(coords,
                                      dir.rotate_ccw(),
                                      WireShape::SplitLeft);
                    }
                    (Some(WireShape::SplitTee), _, _) => {
                        self.set_frag(coords, dir, WireShape::Stub);
                        self.set_frag(coords,
                                      dir.rotate_cw(),
                                      WireShape::Straight);
                        self.set_frag(coords,
                                      dir.rotate_ccw(),
                                      WireShape::Straight);
                    }
                    (_, _, _) => return false,
                }
            }
            GridChange::ToggleCrossWire(coords) => {
                if !self.bounds.contains_point(coords) ||
                    self.chips.contains_key(&coords)
                {
                    return false;
                }
                match self.wire_shape_at(coords, Direction::East) {
                    Some(WireShape::Cross) => {
                        for dir in Direction::all() {
                            self.set_frag(coords, dir, WireShape::Straight);
                        }
                    }
                    Some(WireShape::Straight) => {
                        if self.wire_shape_at(coords, Direction::South) ==
                            Some(WireShape::Straight)
                        {
                            for dir in Direction::all() {
                                self.set_frag(coords, dir, WireShape::Cross);
                            }
                        }
                    }
                    _ => return false,
                }
            }
            GridChange::ReplaceWires(ref old_wires, ref new_wires) => {
                for (&loc, &shape) in old_wires.iter() {
                    // If we're removing a wire fragment, that exact fragment
                    // must currently be there.
                    let (coords, dir) = loc;
                    if self.wire_shape_at(coords, dir) != Some(shape) {
                        return false;
                    }
                    // If we're removing a wire fragment, then we must either
                    // (1) also remove the fragment in the adjacent cell, or
                    // (2) add a new fragment in place of the removed one.
                    let loc2 = (coords + dir, -dir);
                    if !(old_wires.contains_key(&loc2) ||
                             new_wires.contains_key(&loc))
                    {
                        return false;
                    }
                    // If we're removing a wire fragment (and we're not just
                    // adding the same one back), then we must also be removing
                    // the other connected fragments in the grid cell.
                    match shape {
                        _ if new_wires.get(&loc) == Some(&shape) => {}
                        WireShape::Stub => {}
                        WireShape::Straight | WireShape::SplitRight => {
                            if !old_wires.contains_key(&(coords, -dir)) {
                                return false;
                            }
                        }
                        WireShape::TurnLeft | WireShape::SplitLeft |
                        WireShape::SplitTee | WireShape::Cross => {
                            if !old_wires
                                .contains_key(&(coords, dir.rotate_cw()))
                            {
                                return false;
                            }
                        }
                        WireShape::TurnRight => {
                            if !old_wires
                                .contains_key(&(coords, dir.rotate_ccw()))
                            {
                                return false;
                            }
                        }
                    }
                }
                for (&loc, &shape) in new_wires.iter() {
                    // If we're adding a wire fragment, it must be in bounds.
                    let (coords, dir) = loc;
                    if !(self.bounds.contains_point(coords) ||
                             (shape == WireShape::Stub &&
                                  self.bounds.contains_point(coords + dir)))
                    {
                        return false;
                    }
                    // If we're adding a wire fragment, it must not be
                    // completely under a chip.
                    if let Some((chip_coords, ctype, orient)) =
                        self.chip_at(coords)
                    {
                        if shape != WireShape::Stub ||
                            Rect::with_size(chip_coords,
                                            orient * ctype.size())
                                .contains_point(coords + dir)
                        {
                            return false;
                        }
                    }
                    // If we're adding a wire fragment, then we must be
                    // removing the fragment that's currently there, if any.
                    if self.fragments.contains_key(&loc) &&
                        !old_wires.contains_key(&loc)
                    {
                        return false;
                    }
                    // If we're adding a wire fragment, then either (1) we must
                    // also add a fragment in the adjacent cell, or (2) there
                    // must be an existing fragment in the adjacent cell that
                    // we're not removing.
                    let loc2 = (coords + dir, -dir);
                    if !(new_wires.contains_key(&loc2) ||
                             (self.fragments.contains_key(&loc2) &&
                                  !old_wires.contains_key(&loc2)))
                    {
                        return false;
                    }
                    // If we're adding a wire fragment (and we didn't just
                    // remove the exact same one), then we must also be adding
                    // the other connected fragments in the grid cell.
                    match shape {
                        _ if old_wires.get(&loc) == Some(&shape) => {}
                        WireShape::Stub => {}
                        WireShape::Straight => {
                            if new_wires.get(&(coords, -dir)) !=
                                Some(&WireShape::Straight)
                            {
                                return false;
                            }
                        }
                        WireShape::TurnLeft => {
                            if new_wires.get(&(coords, dir.rotate_cw())) !=
                                Some(&WireShape::TurnRight)
                            {
                                return false;
                            }
                        }
                        WireShape::TurnRight => {
                            if new_wires.get(&(coords, dir.rotate_ccw())) !=
                                Some(&WireShape::TurnLeft)
                            {
                                return false;
                            }
                        }
                        WireShape::SplitTee => {
                            if new_wires.get(&(coords, dir.rotate_cw())) !=
                                Some(&WireShape::SplitRight)
                            {
                                return false;
                            }
                        }
                        WireShape::SplitLeft => {
                            if new_wires.get(&(coords, dir.rotate_cw())) !=
                                Some(&WireShape::SplitTee)
                            {
                                return false;
                            }
                        }
                        WireShape::SplitRight => {
                            if new_wires.get(&(coords, -dir)) !=
                                Some(&WireShape::SplitLeft)
                            {
                                return false;
                            }
                        }
                        WireShape::Cross => {
                            if new_wires.get(&(coords, dir.rotate_cw())) !=
                                Some(&WireShape::Cross)
                            {
                                return false;
                            }
                        }
                    }
                }
                // Perform the replacement:
                for (loc, _) in old_wires.iter() {
                    self.fragments.remove(loc);
                }
                for (&(coords, dir), &shape) in new_wires.iter() {
                    self.set_frag(coords, dir, shape);
                }
            }
            GridChange::AddChip(coords, ctype, orient) => {
                let size = orient * ctype.size();
                let rect = CoordsRect::with_size(coords, size);
                // Fail if there's a chip in the way.
                if !self.can_place_chip(rect) {
                    return false;
                }
                // Fail if there's any wires in the way.
                for coords2 in rect {
                    for dir in Direction::all() {
                        match self.wire_shape_at(coords2, dir) {
                            None => {}
                            Some(WireShape::Stub) => {
                                if rect.contains_point(coords2 + dir) {
                                    return false;
                                }
                            }
                            _ => return false,
                        }
                    }
                }
                // Insert the chip.
                for coords2 in rect {
                    let cell = ChipCell::Ref(coords - coords2);
                    self.chips.insert(coords2, cell);
                }
                let cell = ChipCell::Chip(ctype, orient);
                self.chips.insert(coords, cell);
            }
            GridChange::RemoveChip(coords, ctype, orient) => {
                if let Some(&ChipCell::Chip(ctype2, orient2)) =
                    self.chips.get(&coords)
                {
                    if ctype2 == ctype && orient2 == orient {
                        let size = orient * ctype.size();
                        let rect = CoordsRect::with_size(coords, size);
                        for coords2 in rect {
                            self.chips.remove(&coords2);
                        }
                    }
                }
            }
            GridChange::SetBounds(old_bounds, new_bounds) => {
                if self.bounds == old_bounds &&
                    self.can_have_bounds(new_bounds)
                {
                    self.bounds = new_bounds;
                } else {
                    return false;
                }
            }
        }
        return true;
    }

    fn has_frag(&self, coords: Coords, dir: Direction) -> bool {
        self.fragments.contains_key(&(coords, dir))
    }

    fn set_frag(&mut self, coords: Coords, dir: Direction, shape: WireShape) {
        self.fragments.insert((coords, dir), (shape, usize::MAX));
    }

    fn typecheck_wires(&mut self) {
        self.wires_for_ports = HashMap::new();
        self.wire_groups = Vec::new();
        self.eval = None;
        self.validate_wire_fragments();

        let mut all_ports =
            HashMap::<(Coords, Direction), (PortFlow, PortColor)>::new();
        for interface in self.interfaces.iter() {
            for port in interface.ports(self.bounds) {
                all_ports.insert(port.loc(), (port.flow, port.color));
            }
        }
        for (coords, ctype, orient) in self.chips() {
            for port in ctype.ports(coords, orient) {
                all_ports.insert(port.loc(), (port.flow, port.color));
            }
        }

        self.wires = check::group_wires(&all_ports, &mut self.fragments);
        self.errors = check::recolor_wires(&mut self.wires);
        self.wires_for_ports = check::map_ports_to_wires(&self.wires);

        let constraints: Vec<PortConstraint> = self.interfaces
            .iter()
            .flat_map(|interface| interface.constraints(self.bounds))
            .chain(self.chips().flat_map(|(coords, ctype, orient)| {
                                             ctype.constraints(coords, orient)
                                         }))
            .collect();
        self.errors.extend(check::determine_wire_sizes(&mut self.wires,
                                                       &self.wires_for_ports,
                                                       constraints));

        let dependencies: Vec<PortDependency> = self.chips()
            .flat_map(|(coords, ctype, orient)| {
                          ctype.dependencies(coords, orient)
                      })
            .collect();
        match check::detect_loops(&mut self.wires,
                                    &self.wires_for_ports,
                                    dependencies) {
            Ok(groups) => {
                if self.errors.is_empty() {
                    self.wire_groups = groups;
                }
            }
            Err(errors) => self.errors.extend(errors),
        }
    }

    pub fn eval(&self) -> Option<&CircuitEval> { self.eval.as_ref() }

    pub fn eval_mut(&mut self) -> Option<&mut CircuitEval> {
        self.eval.as_mut()
    }

    pub fn start_eval(&mut self) -> bool {
        if !self.errors.is_empty() {
            return false;
        }

        let mut wires_for_ports = HashMap::<(Coords, Direction), usize>::new();
        let mut groups_for_ports =
            HashMap::<(Coords, Direction), usize>::new();
        for (group_index, group) in self.wire_groups.iter().enumerate() {
            for &wire_index in group.iter() {
                let wire = &self.wires[wire_index];
                for (&loc, &(flow, _)) in wire.ports.iter() {
                    debug_assert!(!wires_for_ports.contains_key(&loc));
                    wires_for_ports.insert(loc, wire_index);
                    debug_assert!(!groups_for_ports.contains_key(&loc));
                    if flow == PortFlow::Send {
                        groups_for_ports.insert(loc, group_index);
                    }
                }
            }
        }

        let interact = CircuitInteraction::new();
        let mut chip_evals: Vec<Vec<Box<ChipEval>>> =
            (0..self.wire_groups.len()).map(|_| vec![]).collect();
        for (coords, ctype, orient) in self.chips() {
            let ports = ctype.ports(coords, orient);
            let wires: Vec<(usize, WireSize)> = ports
                .iter()
                .map(|port| {
                         let wire_index = wires_for_ports[&port.loc()];
                         let wire = &self.wires[wire_index];
                         debug_assert!(!wire.has_error);
                         debug_assert!(!wire.size.is_empty());
                         (wire_index, wire.size.lower_bound().unwrap())
                     })
                .collect();
            for (port_index, chip_eval) in
                new_chip_evals(ctype, coords, &wires, &interact)
            {
                let port = &ports[port_index];
                let group_index = groups_for_ports[&port.loc()];
                chip_evals[group_index].push(chip_eval);
            }
        }

        let puzzle_eval = {
            let slots: Vec<Vec<((Coords, Direction), usize)>> =
                self.interfaces
                    .iter()
                    .map(|interface| {
                        interface
                            .ports(self.bounds)
                            .into_iter()
                            .map(|port| {
                                     let loc = port.loc();
                                     (loc, wires_for_ports[&loc])
                                 })
                            .collect()
                    })
                    .collect();
            new_puzzle_eval(self.puzzle, slots)
        };

        self.eval = Some(CircuitEval::new(self.wires.len(),
                                          chip_evals,
                                          puzzle_eval,
                                          interact));
        debug_log!("Starting evaluation");
        return true;
    }

    pub fn stop_eval(&mut self) {
        debug_log!("Stopping evaluation");
        self.eval = None;
    }

    pub fn port_value(&self, loc: (Coords, Direction)) -> Option<u32> {
        if let Some(ref eval) = self.eval {
            if let Some(&wire_index) = self.wires_for_ports.get(&loc) {
                return Some(eval.wire_value(wire_index));
            }
        }
        return None;
    }

    pub fn wire_tooltip_format(&self, index: usize) -> String {
        if index >= self.wires.len() {
            // This shouldn't happen.
            return format!("ERROR: index={} num_wires={}",
                           index,
                           self.wires.len());
        }
        let wire = &self.wires[index];
        let size = if let Some(size) = wire.size.lower_bound() {
            format!("{}-bit", size.num_bits())
        } else {
            "Unsized".to_string()
        };
        let mut fmt = match wire.color {
            WireColor::Unknown => "$*Disconnected wire$*".to_string(),
            WireColor::Ambiguous => format!("$*{} $Rambiguous$D wire$*", size),
            WireColor::Behavior => format!("$*{} $Obehavior$D wire$*", size),
            WireColor::Event => format!("$*{} $Cevent$D wire$*", size),
        };
        if let Some(ref eval) = self.eval {
            match wire.color {
                WireColor::Unknown | WireColor::Ambiguous => {}
                WireColor::Behavior => {
                    fmt.push_str(&format!("\nCurrent value: {}",
                                          eval.wire_value(index)));
                }
                WireColor::Event => {
                    if let Some(value) = eval.wire_event(index) {
                        if wire.size.lower_bound() == Some(WireSize::Zero) {
                            fmt.push_str("\nCurrently has an event.");
                        } else {
                            fmt.push_str(&format!("\nCurrent event value: {}",
                                                  value));
                        }
                    } else {
                        fmt.push_str("\nNo current event.");
                    }
                }
            }
        }
        for error in self.errors.iter() {
            match *error {
                WireError::MultipleSenders(idx) if idx == index => {
                    fmt.push_str("\n\n$RError:$D This wire is connected to \
                                  multiple send ports.  Disconnect it from \
                                  all but one of those ports.");
                }
                WireError::PortColorMismatch(idx) if idx == index => {
                    fmt.push_str("\n\n$RError:$D This wire is connected to \
                                  both a $Obehavior$D and an $Cevent$D \
                                  port.  Wires may only connect ports of the \
                                  same type.");
                }
                WireError::NoValidSize(idx) if idx == index => {
                    // TODO: Make this message more helpful.  For example, if
                    //   the wire must be exactly 2 bits on one side and 4 bits
                    //   on the other, we should give those values.
                    fmt.push_str("\n\n$RError:$D This wire is connecting \
                                  mismatching bit sizes.");
                }
                WireError::UnbrokenLoop(ref indices, contains_events)
                    if indices.contains(&index) => {
                    fmt.push_str("\n\n$RError:$D This wire forms a closed \
                                  loop");
                    if contains_events {
                        fmt.push_str(".  $CEvent$D wire loops can be broken \
                                      with a $*Clock$* or $*Delay$* chip.");
                    } else if self.puzzle.allows_events() {
                        fmt.push_str(".  $OBehavior$D wires may not form \
                                      loops, but loops with $Cevent$D wires \
                                      can be broken with a $*Clock$* or \
                                      $*Delay$* chip.");
                    } else {
                        fmt.push_str(", and $Obehavior$D circuits must be \
                                      acyclic.");
                    }
                }
                _ => {}
            }
        }
        fmt
    }

    #[cfg(not(debug_assertions))]
    fn validate_wire_fragments(&self) {}

    #[cfg(debug_assertions)]
    fn validate_wire_fragments(&self) {
        for (&(coords, dir), &(shape, _)) in self.fragments.iter() {
            assert!(self.fragments.get(&(coords + dir, -dir)).is_some(),
                    "{:?} at ({}, {}) {:?} has nothing in adjacent cell",
                    shape,
                    coords.x,
                    coords.y,
                    dir);
            match shape {
                WireShape::Stub => {}
                WireShape::Straight => {
                    assert_eq!(self.wire_shape_at(coords, -dir),
                               Some(WireShape::Straight),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                }
                WireShape::TurnLeft => {
                    assert_eq!(self.wire_shape_at(coords, dir.rotate_cw()),
                               Some(WireShape::TurnRight),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                }
                WireShape::TurnRight => {
                    assert_eq!(self.wire_shape_at(coords, dir.rotate_ccw()),
                               Some(WireShape::TurnLeft),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                }
                WireShape::SplitLeft => {
                    assert_eq!(self.wire_shape_at(coords, -dir),
                               Some(WireShape::SplitRight),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                    assert_eq!(self.wire_shape_at(coords, dir.rotate_cw()),
                               Some(WireShape::SplitTee),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                }
                WireShape::SplitRight => {
                    assert_eq!(self.wire_shape_at(coords, -dir),
                               Some(WireShape::SplitLeft),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                    assert_eq!(self.wire_shape_at(coords, dir.rotate_ccw()),
                               Some(WireShape::SplitTee),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                }
                WireShape::SplitTee => {
                    assert_eq!(self.wire_shape_at(coords, dir.rotate_ccw()),
                               Some(WireShape::SplitLeft),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                    assert_eq!(self.wire_shape_at(coords, dir.rotate_cw()),
                               Some(WireShape::SplitRight),
                               "({}, {}) {:?}",
                               coords.x,
                               coords.y,
                               dir);
                }
                WireShape::Cross => {
                    for dir2 in Direction::all() {
                        assert_eq!(self.wire_shape_at(coords, dir2),
                                   Some(WireShape::Cross),
                                   "({}, {}) {:?}",
                                   coords.x,
                                   coords.y,
                                   dir);
                    }
                }
            }
        }
    }
}

//===========================================================================//

pub struct ChipsIter<'a> {
    inner: hash_map::Iter<'a, Coords, ChipCell>,
}

impl<'a> Iterator for ChipsIter<'a> {
    type Item = (Coords, ChipType, Orientation);

    fn next(&mut self) -> Option<(Coords, ChipType, Orientation)> {
        while let Some((&coords, cell)) = self.inner.next() {
            match *cell {
                ChipCell::Chip(ctype, orient) => {
                    return Some((coords, ctype, orient));
                }
                ChipCell::Ref(_) => {}
            }
        }
        return None;
    }
}

//===========================================================================//

pub struct WireFragmentsIter<'a> {
    inner: hash_map::Iter<'a, (Coords, Direction), (WireShape, usize)>,
    wires: &'a Vec<WireInfo>,
}

impl<'a> Iterator for WireFragmentsIter<'a> {
    type Item = (Coords, Direction, WireShape, WireSize, WireColor, bool);

    fn next(
        &mut self)
        -> Option<(Coords, Direction, WireShape, WireSize, WireColor, bool)> {
        if let Some((&(coords, dir), &(shape, index))) = self.inner.next() {
            let wire = &self.wires[index];
            let size = wire.size.lower_bound().unwrap_or(WireSize::One);
            Some((coords, dir, shape, size, wire.color, wire.has_error))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl<'a> ExactSizeIterator for WireFragmentsIter<'a> {
    fn len(&self) -> usize { self.inner.len() }
}

//===========================================================================//

// TODO: Tests for to/from_circuit_data.  Make sure we enforce bounds.

//===========================================================================//
