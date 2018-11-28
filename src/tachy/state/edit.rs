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

use super::check::{self, WireColor, WireError, WireInfo, WireShape};
use super::chip::ChipType;
use super::eval::{ChipEval, CircuitEval, CircuitInteraction};
use super::geom::{Coords, CoordsDelta, CoordsRect, Direction, Orientation,
                  Rect, RectSize};
use super::port::{PortColor, PortConstraint, PortDependency, PortFlow};
use super::size::WireSize;
use std::collections::{HashMap, hash_map};
use std::mem;
use std::usize;

//===========================================================================//

#[derive(Clone, Copy, Debug)]
pub enum ChipCell {
    /// A chip.
    Chip(ChipType, Orientation),
    /// For chips larger than 1x1, cells other than the top-left corner use
    /// Ref with the delta to the top-left corner.
    Ref(CoordsDelta),
}

//===========================================================================//

#[derive(Debug)]
pub enum GridChange {
    /// Toggles whether there is a wire between two adjacent cells.
    ToggleStubWire(Coords, Direction),
    /// Toggles whether two edges of a cell are connected.
    ToggleCenterWire(Coords, Direction, Direction),
    /// Toggles whether a wire is connected to the split in the middle of a
    /// cell.
    ToggleSplitWire(Coords, Direction),
    /// Toggles a cell between a four-way split and an overpass/underpass.
    ToggleCrossWire(Coords),
    /// Places or removes a chip on the board.
    ToggleChip(Coords, Orientation, ChipType),
    /// Change the bounds rect from one rect to the other.
    SwapBounds(CoordsRect, CoordsRect),
}

//===========================================================================//

pub struct EditGrid {
    bounds: CoordsRect,
    fragments: HashMap<(Coords, Direction), (WireShape, usize)>,
    chips: HashMap<Coords, ChipCell>,
    wires: Vec<WireInfo>,
    wires_for_ports: HashMap<(Coords, Direction), usize>,
    wire_groups: Vec<Vec<usize>>,
    errors: Vec<WireError>,
    eval: Option<CircuitEval>,
}

impl EditGrid {
    pub fn new() -> EditGrid {
        let mut grid = EditGrid {
            bounds: Rect::new(-4, -3, 8, 6),
            fragments: HashMap::new(),
            chips: HashMap::new(),
            wires: Vec::new(),
            wires_for_ports: HashMap::new(),
            wire_groups: Vec::new(),
            errors: Vec::new(),
            eval: None,
        };
        grid.typecheck_wires();
        grid
    }

    pub fn bounds(&self) -> CoordsRect { self.bounds }

    pub fn can_have_bounds(&self, bounds: CoordsRect) -> bool {
        for &coords in self.chips.keys() {
            if !bounds.contains_point(coords) {
                return false;
            }
        }
        // TODO: Also check wires.
        return true;
    }

    pub fn chips(&self) -> ChipsIter { ChipsIter { inner: self.chips.iter() } }

    pub fn wire_fragments(&self) -> WireFragmentsIter {
        WireFragmentsIter {
            inner: self.fragments.iter(),
            wires: &self.wires,
        }
    }

    pub fn chip_at(&self, coords: Coords)
                   -> Option<(ChipType, Orientation, Coords)> {
        match self.chips.get(&coords) {
            Some(&ChipCell::Chip(ctype, orient)) => {
                Some((ctype, orient, coords))
            }
            Some(&ChipCell::Ref(delta)) => {
                let new_coords = coords + delta;
                match self.chips.get(&new_coords) {
                    Some(&ChipCell::Chip(ctype, orient)) => {
                        Some((ctype, orient, new_coords))
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

    pub fn wire_shape_at(&self, coords: Coords, dir: Direction)
                         -> Option<WireShape> {
        self.fragments.get(&(coords, dir)).map(|&(shape, _)| shape)
    }

    pub fn can_place_chip(&self, coords: Coords, size: RectSize<i32>) -> bool {
        if !self.bounds.contains_rect(Rect::with_size(coords, size)) {
            return false;
        }
        for row in 0..size.height {
            for col in 0..size.width {
                let delta = CoordsDelta::new(col, row);
                if self.chips.contains_key(&(coords + delta)) {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn mutate(&mut self, changes: &[GridChange]) {
        for change in changes {
            self.mutate_one(change);
        }
        self.typecheck_wires();
    }

    fn mutate_one(&mut self, change: &GridChange) {
        match *change {
            // TODO: enforce wires must be in bounds
            GridChange::ToggleStubWire(coords, dir) => {
                let loc1 = (coords, dir);
                let loc2 = (coords + dir, -dir);
                match self.fragments.get(&loc1) {
                    Some(&(WireShape::Stub, _)) => {
                        if let Some(&(WireShape::Stub, _)) =
                            self.fragments.get(&loc2)
                        {
                            self.fragments.remove(&loc1);
                            self.fragments.remove(&loc2);
                        }
                    }
                    None => {
                        if self.fragments.get(&loc2).is_none() {
                            self.fragments
                                .insert(loc1, (WireShape::Stub, usize::MAX));
                            self.fragments
                                .insert(loc2, (WireShape::Stub, usize::MAX));
                        }
                    }
                    _ => debug_log!("{:?} had no effect", change),
                }
            }
            GridChange::ToggleCenterWire(coords, dir1, dir2) => {
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
                    (_, _) => debug_log!("{:?} had no effect", change),
                }
            }
            GridChange::ToggleSplitWire(coords, dir) => {
                match (self.wire_shape_at(coords, dir),
                         self.wire_shape_at(coords, -dir),
                         self.wire_shape_at(coords, dir.rotate_cw())) {
                    (Some(WireShape::Stub), Some(WireShape::SplitTee), _) => {
                        for &dir in Direction::all() {
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
                    (_, _, _) => debug_log!("{:?} had no effect", change),
                }
            }
            GridChange::ToggleCrossWire(coords) => {
                match self.wire_shape_at(coords, Direction::East) {
                    Some(WireShape::Cross) => {
                        for &dir in Direction::all() {
                            self.set_frag(coords, dir, WireShape::Straight);
                        }
                    }
                    Some(WireShape::Straight) => {
                        if self.wire_shape_at(coords, Direction::South) ==
                            Some(WireShape::Straight)
                        {
                            for &dir in Direction::all() {
                                self.set_frag(coords, dir, WireShape::Cross);
                            }
                        }
                    }
                    _ => debug_log!("{:?} had no effect", change),
                }
            }
            GridChange::ToggleChip(coords, orient, ctype) => {
                match self.chips.get(&coords) {
                    None => {
                        let size = orient * ctype.size();
                        if self.can_place_chip(coords, size) {
                            for row in 0..size.height {
                                for col in 0..size.width {
                                    let delta = CoordsDelta::new(col, row);
                                    let cell = ChipCell::Ref(-delta);
                                    self.chips.insert(coords + delta, cell);
                                }
                            }
                            let cell = ChipCell::Chip(ctype, orient);
                            self.chips.insert(coords, cell);
                        } else {
                            debug_log!("{:?} had no effect", change);
                        }
                    }
                    Some(&ChipCell::Chip(ctype2, orient2))
                        if ctype2 == ctype && orient2 == orient => {
                        let size = orient * ctype.size();
                        for row in 0..size.height {
                            for col in 0..size.width {
                                let delta = CoordsDelta::new(col, row);
                                self.chips.remove(&(coords + delta));
                            }
                        }
                    }
                    _ => debug_log!("{:?} had no effect", change),
                }
            }
            GridChange::SwapBounds(mut old_bounds, mut new_bounds) => {
                if self.bounds != old_bounds {
                    mem::swap(&mut old_bounds, &mut new_bounds);
                }
                if self.bounds == old_bounds &&
                    self.can_have_bounds(new_bounds)
                {
                    self.bounds = new_bounds;
                } else {
                    debug_log!("{:?} had no effect", change);
                }
            }
        }
    }

    fn set_frag(&mut self, coords: Coords, dir: Direction, shape: WireShape) {
        self.fragments.insert((coords, dir), (shape, usize::MAX));
    }

    fn typecheck_wires(&mut self) {
        self.wires_for_ports = HashMap::new();
        self.wire_groups = Vec::new();
        self.eval = None;
        if cfg!(debug_assertions) {
            self.validate_wire_fragments();
        }

        let mut all_ports =
            HashMap::<(Coords, Direction), (PortFlow, PortColor)>::new();
        for (coords, ctype, orient) in self.chips() {
            for port in ctype.ports(coords, orient) {
                all_ports.insert(port.loc(), (port.flow, port.color));
            }
        }

        self.wires = check::group_wires(&all_ports, &mut self.fragments);
        self.errors = check::recolor_wires(&mut self.wires);
        self.wires_for_ports = check::map_ports_to_wires(&self.wires);

        let constraints: Vec<PortConstraint> = self.chips()
            .flat_map(|(coords, ctype, orient)| {
                          ctype.constraints(coords, orient)
                      })
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
                         debug_assert!(wire.color != WireColor::Error);
                         debug_assert!(!wire.size.is_empty());
                         (wire_index, wire.size.lower_bound().unwrap())
                     })
                .collect();
            for (port_index, chip_eval) in
                ctype.chip_evals(coords, &wires, &interact)
            {
                let port = &ports[port_index];
                let group_index = groups_for_ports[&port.loc()];
                chip_evals[group_index].push(chip_eval);
            }
        }

        self.eval =
            Some(CircuitEval::new(self.wires.len(), chip_evals, interact));
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
                    for &dir2 in Direction::all() {
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
    type Item = (Coords, Direction, WireShape, WireSize, WireColor);

    fn next(&mut self)
            -> Option<(Coords, Direction, WireShape, WireSize, WireColor)> {
        if let Some((&(coords, dir), &(shape, index))) = self.inner.next() {
            let wire = &self.wires[index];
            let size = wire.size.lower_bound().unwrap_or(WireSize::One);
            Some((coords, dir, shape, size, wire.color))
        } else {
            None
        }
    }
}

//===========================================================================//
