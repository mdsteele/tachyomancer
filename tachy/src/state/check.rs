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

use super::port::{PortColor, PortConstraint, PortDependency, PortFlow};
use crate::geom::{Coords, Direction};
use crate::save::{WireShape, WireSize, WireSizeInterval};
use indexmap::IndexMap;
use pathfinding::prelude::{
    strongly_connected_components, topological_sort_into_groups,
};
use std::collections::{HashMap, HashSet};

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WireColor {
    /// A wire not connected to any ports (or not yet typechecked).
    Unknown,
    /// A wire connected to ports of different types.
    Ambiguous,
    /// A behavior wire.
    Behavior,
    /// An event wire.
    Event,
    /// An analog wire.
    Analog,
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WireId(pub usize);

impl WireId {
    pub const NULL: WireId = WireId(usize::MAX);
}

//===========================================================================//

#[derive(Debug, Eq, PartialEq)]
pub enum WireError {
    MultipleSenders(WireId),
    PortColorMismatch(WireId),
    NoValidSize(WireId),
    UnbrokenLoop(Vec<WireId>, bool),
}

impl WireError {
    pub fn wire_ids(&self) -> Vec<WireId> {
        match self {
            WireError::MultipleSenders(wire_id)
            | WireError::PortColorMismatch(wire_id)
            | WireError::NoValidSize(wire_id) => vec![*wire_id],
            WireError::UnbrokenLoop(wire_ids, _) => wire_ids.clone(),
        }
    }
}

//===========================================================================//

#[derive(Debug)]
pub struct WireInfo {
    pub fragments: HashSet<(Coords, Direction)>,
    pub ports: HashMap<(Coords, Direction), (PortFlow, PortColor)>,
    pub color: WireColor,
    pub size: WireSizeInterval,
    pub has_error: bool,
}

impl WireInfo {
    fn new(
        fragments: HashSet<(Coords, Direction)>,
        ports: HashMap<(Coords, Direction), (PortFlow, PortColor)>,
    ) -> WireInfo {
        WireInfo {
            fragments,
            ports,
            color: WireColor::Unknown,
            size: WireSizeInterval::full(),
            has_error: false,
        }
    }
}

//===========================================================================//

pub fn group_wires(
    all_ports: &HashMap<(Coords, Direction), (PortFlow, PortColor)>,
    all_fragments: &mut HashMap<(Coords, Direction), (WireShape, WireId)>,
) -> Vec<WireInfo> {
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
                        starts.swap_remove(&loc);
                        stack.push((loc, shape));
                    }
                }
            }
        }
        let wire_id = WireId(wires.len());
        for loc in wire_fragments.iter() {
            if let Some(&mut (_, ref mut id)) = all_fragments.get_mut(loc) {
                *id = wire_id;
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
        let mut has_analog = false;
        for &(flow, color) in wire.ports.values() {
            match flow {
                PortFlow::Send => num_senders += 1,
                PortFlow::Recv => {}
            }
            match color {
                PortColor::Behavior => has_behavior = true,
                PortColor::Event => has_event = true,
                PortColor::Analog => has_analog = true,
            }
        }
        if has_behavior && (has_event || has_analog) || has_event && has_analog
        {
            wire.color = WireColor::Ambiguous;
            wire.size = WireSizeInterval::at_least(WireSize::One);
            wire.has_error = true;
            errors.push(WireError::PortColorMismatch(WireId(index)));
        } else if has_behavior {
            wire.color = WireColor::Behavior;
            wire.size = WireSizeInterval::at_least(WireSize::One);
        } else if has_event {
            wire.color = WireColor::Event;
            wire.size = WireSizeInterval::full();
        } else if has_analog {
            wire.color = WireColor::Analog;
            wire.size = WireSizeInterval::exactly(WireSize::ANALOG);
        } else {
            wire.color = WireColor::Unknown;
            wire.size = WireSizeInterval::empty();
        }
        if num_senders > 1 {
            wire.has_error = true;
            errors.push(WireError::MultipleSenders(WireId(index)));
        }
    }
    errors
}

//===========================================================================//

pub fn map_ports_to_wires(
    wires: &Vec<WireInfo>,
) -> HashMap<(Coords, Direction), WireId> {
    let mut wires_for_ports = HashMap::<(Coords, Direction), WireId>::new();
    for (index, wire) in wires.iter().enumerate() {
        for &loc in wire.ports.keys() {
            wires_for_ports.insert(loc, WireId(index));
        }
    }
    wires_for_ports
}

//===========================================================================//

pub fn determine_wire_sizes(
    wires: &mut Vec<WireInfo>,
    wires_for_ports: &HashMap<(Coords, Direction), WireId>,
    mut constraints: Vec<PortConstraint>,
) -> Vec<WireError> {
    let mut changed = true;
    while changed {
        changed = false;
        constraints.retain(|&constraint| {
            match constraint {
                PortConstraint::Exact(loc, size) => {
                    let wire = &mut wires[wires_for_ports[&loc].0];
                    let new_size = wire
                        .size
                        .intersection(WireSizeInterval::exactly(size));
                    if new_size != wire.size {
                        wire.size = new_size;
                        changed = true;
                    }
                }
                PortConstraint::AtLeast(loc, size) => {
                    let wire = &mut wires[wires_for_ports[&loc].0];
                    changed |= wire.size.make_at_least(size);
                }
                PortConstraint::AtMost(loc, size) => {
                    let wire = &mut wires[wires_for_ports[&loc].0];
                    changed |= wire.size.make_at_most(size);
                }
                PortConstraint::Equal(loc1, loc2) => {
                    let id1 = wires_for_ports[&loc1];
                    let id2 = wires_for_ports[&loc2];
                    if id1 != id2 {
                        let size1 = wires[id1.0].size;
                        let size2 = wires[id2.0].size;
                        if !size1.is_empty() && !size2.is_empty() {
                            let new_size = size1.intersection(size2);
                            changed = changed
                                || new_size != size1
                                || new_size != size2;
                            wires[id1.0].size = new_size;
                            wires[id2.0].size = new_size;
                            return new_size.is_ambiguous();
                        }
                    }
                }
                PortConstraint::Double(loc1, loc2) => {
                    let id1 = wires_for_ports[&loc1];
                    let id2 = wires_for_ports[&loc2];
                    if id1 == id2 {
                        let wire = &mut wires[id1.0];
                        changed |= !wire.size.is_empty();
                        wire.size = WireSizeInterval::empty();
                    } else {
                        let size1 = wires[id1.0].size;
                        let size2 = wires[id2.0].size;
                        if !size1.is_empty() && !size2.is_empty() {
                            let new_size1 = size1.intersection(size2.double());
                            let new_size2 = size2.intersection(size1.half());
                            changed |=
                                new_size1 != size1 || new_size2 != size2;
                            wires[id1.0].size = new_size1;
                            wires[id2.0].size = new_size2;
                            return new_size1.is_ambiguous()
                                || new_size2.is_ambiguous();
                        }
                    }
                }
            }
            return false;
        });
    }

    let mut errors = Vec::<WireError>::new();
    for (index, wire) in wires.iter_mut().enumerate() {
        if wire.color != WireColor::Unknown && wire.size.is_empty() {
            wire.has_error = true;
            errors.push(WireError::NoValidSize(WireId(index)));
        }
    }
    errors
}

//===========================================================================//

pub fn detect_loops(
    wires: &mut Vec<WireInfo>,
    wires_for_ports: &HashMap<(Coords, Direction), WireId>,
    dependencies: Vec<PortDependency>,
) -> Result<Vec<Vec<WireId>>, Vec<WireError>> {
    let wire_ids: Vec<WireId> = (0..wires.len()).map(WireId).collect();
    let mut wire_successors: HashMap<WireId, Vec<WireId>> =
        wire_ids.iter().map(|&id| (id, Vec::new())).collect();
    for dependency in dependencies.into_iter() {
        let recv = wires_for_ports[&dependency.recv];
        let send = wires_for_ports[&dependency.send];
        wire_successors.get_mut(&recv).unwrap().push(send);
    }

    match topological_sort_into_groups(&wire_ids, |id| {
        wire_successors[&id].iter().copied()
    }) {
        Ok(groups) => return Ok(groups),
        Err((_, remaining)) => {
            let mut errors = Vec::<WireError>::new();
            let comps = strongly_connected_components(&remaining, |id| {
                wire_successors.get(id).unwrap().iter().copied()
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
                let mut contains_events = false;
                for &WireId(index) in comp.iter() {
                    contains_events |= wires[index].color == WireColor::Event;
                    wires[index].has_error = true;
                }
                errors.push(WireError::UnbrokenLoop(comp, contains_events))
            }
            Err(errors)
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::*;

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
        ports.insert(
            ((0, 0).into(), Direction::East),
            (PortFlow::Send, PortColor::Event),
        );
        ports.insert(
            ((2, 1).into(), Direction::West),
            (PortFlow::Recv, PortColor::Behavior),
        );
        let mut frags =
            HashMap::<(Coords, Direction), (WireShape, WireId)>::new();
        frags.insert(
            ((0, 0).into(), Direction::East),
            (WireShape::Stub, WireId::NULL),
        );
        frags.insert(
            ((1, 0).into(), Direction::West),
            (WireShape::TurnRight, WireId::NULL),
        );
        frags.insert(
            ((1, 0).into(), Direction::South),
            (WireShape::TurnLeft, WireId::NULL),
        );
        frags.insert(
            ((1, 1).into(), Direction::North),
            (WireShape::Stub, WireId::NULL),
        );
        frags.insert(
            ((1, 1).into(), Direction::East),
            (WireShape::Stub, WireId::NULL),
        );
        frags.insert(
            ((2, 1).into(), Direction::West),
            (WireShape::Stub, WireId::NULL),
        );
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
        let mut wires = vec![WireInfo {
            fragments: HashSet::new(),
            ports,
            color: WireColor::Event,
            size: WireSizeInterval::full(),
            has_error: false,
        }];
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
        let mut wires = vec![WireInfo {
            fragments: HashSet::new(),
            ports,
            color: WireColor::Event,
            size: WireSizeInterval::full(),
            has_error: false,
        }];
        let constraints = vec![
            PortConstraint::Exact(loc1, WireSize::Four),
            PortConstraint::Exact(loc2, WireSize::Eight),
        ];
        let wires_for_ports = map_ports_to_wires(&wires);
        let errors =
            determine_wire_sizes(&mut wires, &wires_for_ports, constraints);
        assert_eq!(vec![WireError::NoValidSize(WireId(0))], errors);
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
                has_error: false,
            },
            WireInfo {
                fragments: HashSet::new(),
                ports: ports1,
                color: WireColor::Event,
                size: WireSizeInterval::full(),
                has_error: false,
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
