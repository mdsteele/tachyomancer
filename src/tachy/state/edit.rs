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

use super::geom::{Coords, CoordsDelta, Direction, Orientation};
use std::collections::{HashMap, hash_map};

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChipType {
    Not,
    And,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChipCell {
    /// A chip.
    Chip(ChipType, Orientation),
    /// For chips larger than 1x1, cells other than the top-left corner use
    /// ChipRef with the delta to the top-left corner.
    ChipRef(CoordsDelta),
}

//===========================================================================//

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WireColor {
    /// A wire not connected to any ports (or not yet typechecked).
    Unknown,
    /// A wire connected to ports of different types.
    Error,
    /// A behavior wire.
    Behavior,
    /// An event wire.
    Event,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WireShape {
    /// Wire enters from side of cell but stops immediately.
    Stub,
    /// Wire enters from side of cell and goes straight to the other side.  The
    /// opposite side will also be `Straight`.
    Straight,
    /// Wire enters from side of cell and turns 90 degrees left.  The adjacent
    /// side will be `TurnRight`.
    TurnLeft,
    /// Wire enters from side of cell and turns 90 degrees right.  The adjacent
    /// side will be `TurnLeft`.
    TurnRight,
    /// Wire enters from side of cell and splits, going straight and turning
    /// left.
    SplitLeft,
    /// Wire enters from side of cell and splits, going straight and turning
    /// right.
    SplitRight,
    /// Wire enters from side of cell and splits, turning left and right.
    SplitTee,
    /// Wire enters from side of cell and splits in all directions.
    SplitFour,
}

//===========================================================================//

pub struct EditGrid {
    fragments: HashMap<(Coords, Direction), WireShape>,
    chips: HashMap<Coords, ChipCell>,
}

impl EditGrid {
    pub fn example() -> EditGrid {
        let mut fragments = HashMap::new();
        fragments.insert(((1, 2).into(), Direction::East), WireShape::Stub);
        fragments
            .insert(((2, 2).into(), Direction::West), WireShape::Straight);
        fragments
            .insert(((2, 2).into(), Direction::East), WireShape::Straight);
        fragments
            .insert(((3, 2).into(), Direction::West), WireShape::TurnLeft);
        fragments
            .insert(((3, 2).into(), Direction::North), WireShape::TurnRight);
        fragments
            .insert(((3, 1).into(), Direction::South), WireShape::TurnRight);
        fragments
            .insert(((3, 1).into(), Direction::East), WireShape::TurnLeft);
        fragments
            .insert(((4, 1).into(), Direction::West), WireShape::Straight);
        fragments
            .insert(((4, 1).into(), Direction::East), WireShape::Straight);
        fragments.insert(((5, 1).into(), Direction::West), WireShape::Stub);
        let mut chips = HashMap::new();
        chips.insert((1, 2).into(),
                     ChipCell::Chip(ChipType::Not, Orientation::default()));
        chips.insert((5, 1).into(),
                     ChipCell::Chip(ChipType::And, Orientation::default()));
        EditGrid { fragments, chips }
    }

    pub fn chips(&self) -> ChipsIter { ChipsIter { inner: self.chips.iter() } }

    pub fn wire_fragments(&self) -> WireFragmentsIter {
        WireFragmentsIter { inner: self.fragments.iter() }
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
                ChipCell::ChipRef(_) => {}
            }
        }
        return None;
    }
}

//===========================================================================//

pub struct WireFragmentsIter<'a> {
    inner: hash_map::Iter<'a, (Coords, Direction), WireShape>,
}

impl<'a> Iterator for WireFragmentsIter<'a> {
    type Item = (Coords, Direction, WireShape);

    fn next(&mut self) -> Option<(Coords, Direction, WireShape)> {
        self.inner.next().map(|(&(coords, dir), &shape)| (coords, dir, shape))
    }
}

//===========================================================================//
