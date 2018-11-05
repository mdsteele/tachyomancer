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
use super::geom::{Coords, CoordsDelta, Direction, Orientation};
use super::port::{PortColor, PortConstraint, PortFlow};
use super::size::WireSize;
use std::collections::{HashMap, hash_map};
use std::usize;

//===========================================================================//

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
            ((3, 1), Direction::South, WireShape::TurnRight),
            ((3, 1), Direction::East, WireShape::TurnLeft),
            ((4, 1), Direction::West, WireShape::Straight),
            ((4, 1), Direction::East, WireShape::Straight),
            ((5, 1), Direction::West, WireShape::Stub),
        ];
        let fragments = fragments
            .into_iter()
            .map(|(coords, dir, shape)| {
                     ((coords.into(), dir), (shape, usize::MAX))
                 })
            .collect();
        let mut chips = HashMap::new();
        chips.insert((1, 2).into(),
                     ChipCell::Chip(ChipType::Not, Orientation::default()));
        chips.insert((5, 1).into(),
                     ChipCell::Chip(ChipType::And, Orientation::default()));
        chips.insert((5, 2).into(),
                     ChipCell::Chip(ChipType::Delay, Orientation::default()));
        chips.insert((2, 3).into(),
                     ChipCell::Chip(ChipType::Const(7),
                                    Orientation::default()));
        chips.insert((7, 0).into(),
                     ChipCell::Chip(ChipType::Discard,
                                    Orientation::default()));
        chips.insert((7, 2).into(),
                     ChipCell::Chip(ChipType::Pack, Orientation::default()));
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

    fn typecheck_wires(&mut self) {
        let mut all_ports =
            HashMap::<(Coords, Direction), (PortFlow, PortColor)>::new();
        for (coords, ctype, orient) in self.chips() {
            for port in ctype.ports(orient) {
                all_ports.insert((coords + port.pos, port.dir),
                                 (port.flow, port.color));
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
        self.wires = wires;
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
