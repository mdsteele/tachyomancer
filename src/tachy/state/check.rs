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
use super::port::{PortColor, PortConstraint, PortDependency, PortFlow};
use super::size::{WireSize, WireSizeInterval};
use super::topsort::topological_sort_into_groups;
use indexmap::IndexMap;
use pathfinding::prelude::strongly_connected_components;
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
    Cross,
}

//===========================================================================//

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

impl WireInfo {
    fn new(fragments: HashSet<(Coords, Direction)>,
           ports: HashMap<(Coords, Direction), (PortFlow, PortColor)>)
           -> WireInfo {
        WireInfo {
            fragments,
            ports,
            color: WireColor::Unknown,
            size: WireSizeInterval::full(),
        }
    }
}

//===========================================================================//

pub fn group_wires(all_ports: &HashMap<(Coords, Direction),
                                       (PortFlow, PortColor)>,
                   all_fragments: &mut HashMap<(Coords, Direction),
                                               (WireShape, usize)>)
                   -> Vec<WireInfo> {
    // TODO: Allow more limited starts for incremental typechecking.
    let mut starts: IndexMap<(Coords, Direction), WireShape> =
        all_fragments.iter().map(|(&k, &(v, _))| (k, v)).collect();

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
                WireShape::Cross => {
                    next.push((coords, -dir));
                    next.push((coords, dir.rotate_cw()));
                    next.push((coords, dir.rotate_ccw()));
                }
            }
            for &loc in next.iter() {
                if let Some(&(shape, _)) = all_fragments.get(&loc) {
                    if wire_fragments.insert(loc) {
                        starts.remove(&loc);
                        stack.push((loc, shape));
                    }
                }
            }
        }
        let wire_index = wires.len();
        for loc in wire_fragments.iter() {
            if let Some(&mut (_, ref mut index)) = all_fragments.get_mut(loc) {
                *index = wire_index;
            }
        }
        wires.push(WireInfo::new(wire_fragments, wire_ports));
    }

    // Add fragment-less wires for any ports that don't have a wire yet.
    for (&loc, &port) in all_ports.iter() {
        if !all_fragments.contains_key(&loc) {
            let mut wire_ports = HashMap::with_capacity(1);
            wire_ports.insert(loc, port);
            wires.push(WireInfo::new(HashSet::new(), wire_ports));
        }
    }

    wires
}

//===========================================================================//

pub fn recolor_wires(wires: &mut Vec<WireInfo>) -> Vec<WireError> {
    let mut errors = Vec::<WireError>::new();
    for (index, wire) in wires.iter_mut().enumerate() {
        let mut num_senders = 0;
        let mut has_behavior = false;
        let mut has_event = false;
        for &(flow, color) in wire.ports.values() {
            match flow {
                PortFlow::Send => num_senders += 1,
                PortFlow::Recv => {}
            }
            match color {
                PortColor::Behavior => has_behavior = true,
                PortColor::Event => has_event = true,
            }
        }
        if num_senders > 1 {
            errors.push(WireError::MultipleSenders(index));
            wire.color = WireColor::Error;
            wire.size = WireSizeInterval::empty();
        } else if has_behavior && has_event {
            errors.push(WireError::PortColorMismatch(index));
            wire.color = WireColor::Error;
            wire.size = WireSizeInterval::empty();
        } else if has_behavior {
            wire.color = WireColor::Behavior;
            wire.size = WireSizeInterval::at_least(WireSize::One);
        } else if has_event {
            wire.color = WireColor::Event;
            wire.size = WireSizeInterval::full();
        } else {
            wire.color = WireColor::Unknown;
            wire.size = WireSizeInterval::empty();
        }
    }
    errors
}

//===========================================================================//

pub fn map_ports_to_wires(wires: &Vec<WireInfo>)
                          -> HashMap<(Coords, Direction), usize> {
    let mut wires_for_ports = HashMap::<(Coords, Direction), usize>::new();
    for (index, wire) in wires.iter().enumerate() {
        for &loc in wire.ports.keys() {
            wires_for_ports.insert(loc, index);
        }
    }
    wires_for_ports
}

//===========================================================================//

pub fn determine_wire_sizes(wires: &mut Vec<WireInfo>,
                            wires_for_ports: &HashMap<(Coords, Direction),
                                                      usize>,
                            mut constraints: Vec<PortConstraint>)
                            -> Vec<WireError> {
    let mut changed = true;
    while changed {
        changed = false;
        constraints.retain(|&constraint| {
            match constraint {
                PortConstraint::Exact(loc, size) => {
                    let wire = &mut wires[wires_for_ports[&loc]];
                    let new_size =
                        wire.size
                            .intersection(WireSizeInterval::exactly(size));
                    if new_size != wire.size {
                        wire.size = new_size;
                        changed = true;
                    }
                }
                PortConstraint::AtLeast(loc, size) => {
                    let wire = &mut wires[wires_for_ports[&loc]];
                    changed |= wire.size.make_at_least(size);
                }
                PortConstraint::AtMost(loc, size) => {
                    let wire = &mut wires[wires_for_ports[&loc]];
                    changed |= wire.size.make_at_most(size);
                }
                PortConstraint::Equal(loc1, loc2) => {
                    let index1 = wires_for_ports[&loc1];
                    let index2 = wires_for_ports[&loc2];
                    if index1 != index2 {
                        let size1 = wires[index1].size;
                        let size2 = wires[index2].size;
                        if !size1.is_empty() && !size2.is_empty() {
                            let new_size = size1.intersection(size2);
                            changed = changed || new_size != size1 ||
                                new_size != size2;
                            wires[index1].size = new_size;
                            wires[index2].size = new_size;
                            return new_size.is_ambiguous();
                        }
                    }
                }
                PortConstraint::Double(loc1, loc2) => {
                    let index1 = wires_for_ports[&loc1];
                    let index2 = wires_for_ports[&loc2];
                    if index1 == index2 {
                        let wire = &mut wires[index1];
                        changed |= !wire.size.is_empty();
                        wire.size = WireSizeInterval::empty();
                    } else {
                        let size1 = wires[index1].size;
                        let size2 = wires[index2].size;
                        if !size1.is_empty() && !size2.is_empty() {
                            let new_size1 = size1.intersection(size2.double());
                            let new_size2 = size2.intersection(size1.half());
                            changed |= new_size1 != size1 ||
                                new_size2 != size2;
                            wires[index1].size = new_size1;
                            wires[index2].size = new_size2;
                            return new_size1.is_ambiguous() ||
                                new_size2.is_ambiguous();
                        }
                    }
                }
            }
            return false;
        });
    }

    let mut errors = Vec::<WireError>::new();
    for (index, wire) in wires.iter_mut().enumerate() {
        match wire.color {
            WireColor::Behavior | WireColor::Event => {
                if wire.size.is_empty() {
                    wire.color = WireColor::Error;
                    errors.push(WireError::NoValidSize(index));
                }
            }
            WireColor::Unknown | WireColor::Error => {}
        }
    }
    errors
}

//===========================================================================//

pub fn detect_loops(wires: &mut Vec<WireInfo>,
                    wires_for_ports: &HashMap<(Coords, Direction), usize>,
                    dependencies: Vec<PortDependency>)
                    -> Result<Vec<Vec<usize>>, Vec<WireError>> {
    let wire_indices: Vec<usize> = (0..wires.len()).collect();
    let mut wire_successors: HashMap<usize, Vec<usize>> =
        wire_indices.iter().map(|&index| (index, Vec::new())).collect();
    for dependency in dependencies.into_iter() {
        let recv = wires_for_ports[&dependency.recv];
        let send = wires_for_ports[&dependency.send];
        wire_successors.get_mut(&recv).unwrap().push(send);
    }

    match topological_sort_into_groups(&wire_indices, |index| {
        wire_successors[&index].iter().cloned()
    }) {
        Ok(groups) => return Ok(groups),
        Err((_, remaining)) => {
            let mut errors = Vec::<WireError>::new();
            let comps = strongly_connected_components(&remaining, |index| {
                wire_successors.get(index).unwrap().iter().cloned()
            });
            for comp in comps.into_iter() {
                // By definition, a wire that isn't part of any cycle forms a
                // strongly-connected component of size 1.  A wire that forms a
                // self-loop can also be a component of size 1.  We want to
                // issue errors only for the latter case.
                if comp.len() == 1 {
                    let wire = &comp[0];
                    if !wire_successors.get(wire).unwrap().contains(wire) {
                        continue; // This wire doesn't have a self-loop.
                    }
                }
                for &index in comp.iter() {
                    wires[index].color = WireColor::Error;
                }
                errors.push(WireError::UnbrokenLoop(comp))
            }
            Err(errors)
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::*;
    use std::usize;

    #[test]
    fn group_no_wires() {
        let mut wires = group_wires(&HashMap::new(), &mut HashMap::new());
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
        let mut frags =
            HashMap::<(Coords, Direction), (WireShape, usize)>::new();
        frags.insert(((0, 0).into(), Direction::East),
                     (WireShape::Stub, usize::MAX));
        frags.insert(((1, 0).into(), Direction::West),
                     (WireShape::TurnRight, usize::MAX));
        frags.insert(((1, 0).into(), Direction::South),
                     (WireShape::TurnLeft, usize::MAX));
        frags.insert(((1, 1).into(), Direction::North),
                     (WireShape::Stub, usize::MAX));
        frags.insert(((1, 1).into(), Direction::East),
                     (WireShape::Stub, usize::MAX));
        frags.insert(((2, 1).into(), Direction::West),
                     (WireShape::Stub, usize::MAX));
        let mut wires = group_wires(&ports, &mut frags);
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
        let mut wires = vec![];
        let constraints = vec![];
        let wires_for_ports = map_ports_to_wires(&wires);
        let errors =
            determine_wire_sizes(&mut wires, &wires_for_ports, constraints);
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
        let wires_for_ports = map_ports_to_wires(&wires);
        let errors =
            determine_wire_sizes(&mut wires, &wires_for_ports, constraints);
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
        let wires_for_ports = map_ports_to_wires(&wires);
        let errors =
            determine_wire_sizes(&mut wires, &wires_for_ports, constraints);
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
        let wires_for_ports = map_ports_to_wires(&wires);
        let errors =
            determine_wire_sizes(&mut wires, &wires_for_ports, constraints);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(WireSizeInterval::exactly(WireSize::Four), wires[0].size);
        assert_eq!(WireSizeInterval::exactly(WireSize::Two), wires[1].size);
    }

}

//===========================================================================//
