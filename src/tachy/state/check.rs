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

use super::geom::{Coords, Direction};
use super::port::{PortColor, PortConstraint, PortFlow};
use super::size::{WireSize, WireSizeInterval};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

//===========================================================================//

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

//===========================================================================//

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

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
pub enum WireError {
    MultipleSenders(usize),
    PortColorMismatch(usize),
    NoValidSize(usize),
    UnbrokenLoop(Vec<usize>),
}

//===========================================================================//

#[derive(Debug)]
pub struct WireInfo {
    pub fragments: HashSet<(Coords, Direction)>,
    pub ports: HashMap<(Coords, Direction), (PortFlow, PortColor)>,
    pub color: WireColor,
    pub size: WireSizeInterval,
}

//===========================================================================//

pub fn group_wires(all_ports: &HashMap<(Coords, Direction),
                                       (PortFlow, PortColor)>,
                   all_fragments: &HashMap<(Coords, Direction), WireShape>)
                   -> Vec<WireInfo> {
    // TODO: Allow more limited starts for incremental typechecking.
    let mut starts: IndexMap<(Coords, Direction), WireShape> =
        all_fragments.iter().map(|(&k, &v)| (k, v)).collect();

    // Collect fragments into wires:
    let mut wires = Vec::<WireInfo>::new();
    while let Some(start) = starts.pop() {
        let mut stack = vec![start];
        let mut wire_fragments = HashSet::<(Coords, Direction)>::new();
        let mut wire_ports =
            HashMap::<(Coords, Direction), (PortFlow, PortColor)>::new();
        while let Some(((coords, dir), shape)) = stack.pop() {
            let mut next = vec![(coords + dir, -dir)];
            match shape {
                WireShape::Stub => {
                    if let Some(&port) = all_ports.get(&(coords, dir)) {
                        wire_ports.insert((coords, dir), port);
                    }
                }
                WireShape::Straight => {
                    next.push((coords, -dir));
                }
                WireShape::TurnLeft => {
                    next.push((coords, dir.rotate_cw()));
                }
                WireShape::TurnRight => {
                    next.push((coords, dir.rotate_ccw()));
                }
                WireShape::SplitLeft => {
                    next.push((coords, -dir));
                    next.push((coords, dir.rotate_cw()));
                }
                WireShape::SplitRight => {
                    next.push((coords, -dir));
                    next.push((coords, dir.rotate_ccw()));
                }
                WireShape::SplitTee => {
                    next.push((coords, dir.rotate_cw()));
                    next.push((coords, dir.rotate_ccw()));
                }
                WireShape::SplitFour => {
                    next.push((coords, -dir));
                    next.push((coords, dir.rotate_cw()));
                    next.push((coords, dir.rotate_ccw()));
                }
            }
            for &loc in next.iter() {
                if let Some(&shape) = all_fragments.get(&loc) {
                    if wire_fragments.insert(loc) {
                        starts.remove(&loc);
                        stack.push((loc, shape));
                    }
                }
            }
        }
        wires.push(WireInfo {
                       fragments: wire_fragments,
                       ports: wire_ports,
                       color: WireColor::Unknown,
                       size: WireSizeInterval::full(),
                   });
    }
    wires
}

//===========================================================================//

pub fn recolor_wires(wires: &mut Vec<WireInfo>) -> Vec<WireError> {
    let mut errors = Vec::<WireError>::new();
    for (index, wire) in wires.iter_mut().enumerate() {
        let mut has_sender = false;
        let mut has_behavior = false;
        let mut has_event = false;
        for &(flow, color) in wire.ports.values() {
            if flow == PortFlow::Send {
                if has_sender {
                    errors.push(WireError::MultipleSenders(index));
                    wire.color = WireColor::Error;
                    break;
                }
                has_sender = true;
            }
            match color {
                PortColor::Behavior => has_behavior = true,
                PortColor::Event => has_event = true,
            }
        }
        if has_behavior {
            if has_event {
                errors.push(WireError::PortColorMismatch(index));
                wire.color = WireColor::Error;
                break;
            } else {
                wire.color = WireColor::Behavior;
                wire.size.make_at_least(WireSize::One);
            }
        } else if has_event {
            wire.color = WireColor::Event;
        } else {
            wire.color = WireColor::Unknown;
        }
    }
    errors
}

//===========================================================================//

pub fn determine_wire_sizes(wires: &mut Vec<WireInfo>,
                            mut constraints: Vec<PortConstraint>)
                            -> Vec<WireError> {
    let mut wires_for_ports = HashMap::<(Coords, Direction), usize>::new();
    for (index, wire) in wires.iter().enumerate() {
        for &loc in wire.ports.keys() {
            wires_for_ports.insert(loc, index);
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        constraints.retain(|&constraint| {
            match constraint {
                PortConstraint::Exact(loc, size) => {
                    if let Some(&index) = wires_for_ports.get(&loc) {

                        let wire = &mut wires[index];
                        let new_size =
                            wire.size
                                .intersection(WireSizeInterval::exactly(size));
                        if new_size != wire.size {
                            wire.size = new_size;
                            changed = true;
                        }
                    }
                }
                PortConstraint::AtLeast(loc, size) => {
                    if let Some(&index) = wires_for_ports.get(&loc) {
                        changed |= wires[index].size.make_at_least(size);
                    }
                }
                PortConstraint::AtMost(loc, size) => {
                    if let Some(&index) = wires_for_ports.get(&loc) {
                        changed |= wires[index].size.make_at_most(size);
                    }
                }
                PortConstraint::Equal(loc1, loc2) => {
                    if let Some(&index1) = wires_for_ports.get(&loc1) {
                        if let Some(&index2) = wires_for_ports.get(&loc2) {
                            if index1 != index2 {
                                let size1 = wires[index1].size;
                                let size2 = wires[index2].size;
                                let new_size = size1.intersection(size2);
                                changed = changed || new_size != size1 ||
                                    new_size != size2;
                                wires[index1].size = new_size;
                                wires[index2].size = new_size;
                                return new_size.is_ambiguous();
                            }
                        }
                    }
                }
                PortConstraint::Double(loc1, loc2) => {
                    if let Some(&index1) = wires_for_ports.get(&loc1) {
                        changed |=
                            wires[index1].size.make_at_least(WireSize::Two);
                        if let Some(&index2) = wires_for_ports.get(&loc2) {
                            if index1 == index2 {
                                let wire = &mut wires[index1];
                                changed |= !wire.size.is_empty();
                                wire.size = WireSizeInterval::empty();
                            } else {
                                let size1 = wires[index1].size;
                                let size2 = wires[index2].size;
                                let new_size1 =
                                    size1.intersection(size2.double());
                                let new_size2 =
                                    size2.intersection(size1.half());
                                changed |= new_size1 != size1 ||
                                    new_size2 != size2;
                                wires[index1].size = new_size1;
                                wires[index2].size = new_size2;
                                return new_size1.is_ambiguous() ||
                                    new_size2.is_ambiguous();
                            }
                        }
                    } else if let Some(&index2) = wires_for_ports.get(&loc2) {
                        let old_size2 = wires[index2].size;
                        let new_size2 =
                            old_size2
                                .intersection(WireSizeInterval::full().half());
                        if new_size2 != old_size2 {
                            wires[index2].size = new_size2;
                            changed = true;
                        }
                    }
                }
            }
            return false;
        });
    }

    let mut errors = Vec::<WireError>::new();
    for (index, wire) in wires.iter_mut().enumerate() {
        if wire.size.is_empty() {
            wire.color = WireColor::Error;
            errors.push(WireError::NoValidSize(index));
        }
    }
    errors
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_no_wires() {
        let mut wires = group_wires(&HashMap::new(), &HashMap::new());
        assert_eq!(0, wires.len());
        let errors = recolor_wires(&mut wires);
        assert_eq!(0, errors.len());
    }

    #[test]
    fn group_multiple_wires() {
        let mut ports =
            HashMap::<(Coords, Direction), (PortFlow, PortColor)>::new();
        ports.insert(((0, 0).into(), Direction::East),
                     (PortFlow::Send, PortColor::Event));
        ports.insert(((2, 1).into(), Direction::West),
                     (PortFlow::Recv, PortColor::Behavior));
        let mut frags = HashMap::<(Coords, Direction), WireShape>::new();
        frags.insert(((0, 0).into(), Direction::East), WireShape::Stub);
        frags.insert(((1, 0).into(), Direction::West), WireShape::TurnRight);
        frags.insert(((1, 0).into(), Direction::South), WireShape::TurnLeft);
        frags.insert(((1, 1).into(), Direction::North), WireShape::Stub);
        frags.insert(((1, 1).into(), Direction::East), WireShape::Stub);
        frags.insert(((2, 1).into(), Direction::West), WireShape::Stub);
        let mut wires = group_wires(&ports, &frags);
        assert_eq!(2, wires.len(), "wires: {:?}", wires);
        wires.sort_unstable_by_key(|info| info.fragments.len());
        assert_eq!(2, wires[0].fragments.len());
        assert_eq!(4, wires[1].fragments.len());
        let errors = recolor_wires(&mut wires);
        assert_eq!(0, errors.len());
        assert_eq!(WireColor::Behavior, wires[0].color);
        assert_eq!(WireColor::Event, wires[1].color);
    }

    #[test]
    fn typecheck_no_wires() {
        let errors = determine_wire_sizes(&mut vec![], vec![]);
        assert!(errors.is_empty(), "errors: {:?}", errors);
    }

    #[test]
    fn typecheck_one_wire_success() {
        let loc1: (Coords, Direction) = ((0, 0).into(), Direction::East);
        let loc2: (Coords, Direction) = ((1, 0).into(), Direction::West);
        let mut ports = HashMap::new();
        ports.insert(loc1, (PortFlow::Send, PortColor::Event));
        ports.insert(loc2, (PortFlow::Recv, PortColor::Event));
        let mut wires = vec![
            WireInfo {
                fragments: HashSet::new(),
                ports,
                color: WireColor::Event,
                size: WireSizeInterval::full(),
            },
        ];
        let constraints = vec![
            PortConstraint::Exact(loc1, WireSize::Four),
            PortConstraint::Exact(loc2, WireSize::Four),
        ];
        let errors = determine_wire_sizes(&mut wires, constraints);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(WireSizeInterval::exactly(WireSize::Four), wires[0].size);
    }

    #[test]
    fn typecheck_one_wire_error() {
        let loc1: (Coords, Direction) = ((0, 0).into(), Direction::East);
        let loc2: (Coords, Direction) = ((1, 0).into(), Direction::West);
        let mut ports = HashMap::new();
        ports.insert(loc1, (PortFlow::Send, PortColor::Event));
        ports.insert(loc2, (PortFlow::Recv, PortColor::Event));
        let mut wires = vec![
            WireInfo {
                fragments: HashSet::new(),
                ports,
                color: WireColor::Event,
                size: WireSizeInterval::full(),
            },
        ];
        let constraints = vec![
            PortConstraint::Exact(loc1, WireSize::Four),
            PortConstraint::Exact(loc2, WireSize::Eight),
        ];
        let errors = determine_wire_sizes(&mut wires, constraints);
        assert_eq!(vec![WireError::NoValidSize(0)], errors);
        assert!(wires[0].size.is_empty());
    }

    #[test]
    fn typecheck_two_wires_success() {
        let loc1: (Coords, Direction) = ((0, 0).into(), Direction::East);
        let loc2: (Coords, Direction) = ((1, 0).into(), Direction::West);
        let loc3: (Coords, Direction) = ((1, 0).into(), Direction::East);
        let mut ports0 = HashMap::new();
        ports0.insert(loc1, (PortFlow::Send, PortColor::Event));
        ports0.insert(loc2, (PortFlow::Recv, PortColor::Event));
        let mut ports1 = HashMap::new();
        ports1.insert(loc3, (PortFlow::Send, PortColor::Event));
        let mut wires = vec![
            WireInfo {
                fragments: HashSet::new(),
                ports: ports0,
                color: WireColor::Event,
                size: WireSizeInterval::full(),
            },
            WireInfo {
                fragments: HashSet::new(),
                ports: ports1,
                color: WireColor::Event,
                size: WireSizeInterval::full(),
            },
        ];
        let constraints = vec![
            PortConstraint::Double(loc2, loc3),
            PortConstraint::Exact(loc1, WireSize::Four),
        ];
        let errors = determine_wire_sizes(&mut wires, constraints);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(WireSizeInterval::exactly(WireSize::Four), wires[0].size);
        assert_eq!(WireSizeInterval::exactly(WireSize::Two), wires[1].size);
    }

}

//===========================================================================//
