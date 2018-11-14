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

use super::check::{self, WireColor, WireInfo, WireShape};
use super::chip::ChipType;
use super::geom::{Coords, CoordsDelta, Direction, Orientation, RectSize};
use super::port::{PortColor, PortConstraint, PortFlow};
use super::size::WireSize;
use std::collections::{HashMap, hash_map};
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
}

//===========================================================================//

pub struct EditGrid {
    fragments: HashMap<(Coords, Direction), (WireShape, usize)>,
    chips: HashMap<Coords, ChipCell>,
    wires: Vec<WireInfo>,
}

impl EditGrid {
    pub fn example() -> EditGrid {
        let fragments = vec![
            ((1, 2), Direction::East, WireShape::Stub),
            ((2, 2), Direction::West, WireShape::Straight),
            ((2, 2), Direction::East, WireShape::Straight),
            ((3, 2), Direction::West, WireShape::TurnLeft),
            ((3, 2), Direction::North, WireShape::TurnRight),
            ((3, 1), Direction::South, WireShape::SplitTee),
            ((3, 1), Direction::West, WireShape::SplitRight),
            ((3, 1), Direction::East, WireShape::SplitLeft),
            ((2, 1), Direction::East, WireShape::Stub),
            ((4, 1), Direction::West, WireShape::Cross),
            ((4, 1), Direction::North, WireShape::Cross),
            ((4, 1), Direction::South, WireShape::Cross),
            ((4, 1), Direction::East, WireShape::Cross),
            ((5, 1), Direction::West, WireShape::Stub),
            ((4, 0), Direction::South, WireShape::Stub),
            ((4, 2), Direction::North, WireShape::Stub),
            ((5, 1), Direction::East, WireShape::Stub),
            ((6, 1), Direction::West, WireShape::SplitRight),
            ((6, 1), Direction::East, WireShape::SplitLeft),
            ((6, 1), Direction::South, WireShape::SplitTee),
            ((7, 1), Direction::West, WireShape::Stub),
            ((6, 2), Direction::North, WireShape::TurnLeft),
            ((6, 2), Direction::East, WireShape::TurnRight),
            ((7, 2), Direction::West, WireShape::Stub),
            ((7, 4), Direction::East, WireShape::Stub),
            ((8, 4), Direction::West, WireShape::TurnLeft),
            ((8, 4), Direction::North, WireShape::TurnRight),
            ((8, 3), Direction::South, WireShape::Stub),
        ];
        let fragments = fragments
            .into_iter()
            .map(|(coords, dir, shape)| {
                     ((coords.into(), dir), (shape, usize::MAX))
                 })
            .collect();
        let mut chips = HashMap::new();
        chips.insert((1, 2).into(),
                     ChipCell::Chip(ChipType::Const(3),
                                    Orientation::default()));
        chips.insert((5, 1).into(),
                     ChipCell::Chip(ChipType::Pack, Orientation::default()));
        chips.insert((7, 4).into(),
                     ChipCell::Chip(ChipType::Delay, Orientation::default()));
        chips.insert((2, 3).into(),
                     ChipCell::Chip(ChipType::Not,
                                    Orientation::default().rotate_cw()));
        chips.insert((7, 2).into(),
                     ChipCell::Chip(ChipType::Ram, Orientation::default()));
        chips.insert((8, 2).into(), ChipCell::Ref((-1, 0).into()));
        chips.insert((7, 3).into(), ChipCell::Ref((0, -1).into()));
        chips.insert((8, 3).into(), ChipCell::Ref((-1, -1).into()));
        chips.insert((7, 0).into(),
                     ChipCell::Chip(ChipType::Discard,
                                    Orientation::default()));
        chips.insert((7, 1).into(),
                     ChipCell::Chip(ChipType::And, Orientation::default()));
        let mut grid = EditGrid {
            fragments,
            chips,
            wires: Vec::new(),
        };
        grid.typecheck_wires();
        grid
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
        for row in 0..size.height {
            for col in 0..size.width {
                let delta = CoordsDelta { x: col, y: row };
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
                            for y in 0..size.height {
                                for x in 0..size.width {
                                    let delta = CoordsDelta { x, y };
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
                        for y in 0..size.height {
                            for x in 0..size.width {
                                let delta = CoordsDelta { x, y };
                                self.chips.remove(&(coords + delta));
                            }
                        }
                    }
                    _ => debug_log!("{:?} had no effect", change),
                }
            }
        }
    }

    fn set_frag(&mut self, coords: Coords, dir: Direction, shape: WireShape) {
        self.fragments.insert((coords, dir), (shape, usize::MAX));
    }

    fn typecheck_wires(&mut self) {
        if cfg!(debug_assertions) {
            self.validate_wire_fragments();
        }
        let mut all_ports =
            HashMap::<(Coords, Direction), (PortFlow, PortColor)>::new();
        for (coords, ctype, orient) in self.chips() {
            for port in ctype.ports(coords, orient) {
                all_ports
                    .insert((port.pos, port.dir), (port.flow, port.color));
            }
        }

        let mut wires = check::group_wires(&all_ports, &mut self.fragments);
        let _errors = check::recolor_wires(&mut wires);
        let constraints: Vec<PortConstraint> = self.chips()
            .flat_map(|(coords, ctype, orient)| {
                          ctype.constraints(coords, orient)
                      })
            .collect();
        let _more_errors = check::determine_wire_sizes(&mut wires,
                                                       constraints);
        // TODO: check for loops
        self.wires = wires;
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
