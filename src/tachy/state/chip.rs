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

use super::eval::{self, ChipEval, CircuitInteraction};
use super::port::{PortColor, PortConstraint, PortDependency, PortFlow,
                  PortSpec};
use super::size::WireSize;
use std::cell::RefCell;
use std::rc::Rc;
use std::str;
use tachy::geom::{Coords, CoordsSize, Orientation};
use tachy::geom::Direction::{self, East, North, South, West};
use tachy::save::{Puzzle, PuzzleKind};

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChipType {
    Const(u32),
    // Bitwise:
    Not,
    And,
    Pack,
    Unpack,
    // Arithmetic:
    Add,
    Compare,
    Mux,
    // Events:
    Clock,
    Delay,
    Discard,
    Join,
    Latest,
    Sample,
    // Special:
    Break,
    Ram,
    Display,
    Button,
}

impl str::FromStr for ChipType {
    type Err = String;

    fn from_str(string: &str) -> Result<ChipType, String> {
        match string {
            "Add" => Ok(ChipType::Add),
            "And" => Ok(ChipType::And),
            "Break" => Ok(ChipType::Break),
            "Button" => Ok(ChipType::Button),
            "Clock" => Ok(ChipType::Clock),
            "Compare" => Ok(ChipType::Compare),
            "Delay" => Ok(ChipType::Delay),
            "Discard" => Ok(ChipType::Discard),
            "Display" => Ok(ChipType::Display),
            "Join" => Ok(ChipType::Join),
            "Latest" => Ok(ChipType::Latest),
            "Mux" => Ok(ChipType::Mux),
            "Not" => Ok(ChipType::Not),
            "Pack" => Ok(ChipType::Pack),
            "Ram" => Ok(ChipType::Ram),
            "Sample" => Ok(ChipType::Sample),
            "Unpack" => Ok(ChipType::Unpack),
            _ => {
                if string.starts_with("Const(") && string.ends_with(')') {
                    let inner = &string[6..(string.len() - 1)];
                    if let Ok(value) = inner.parse() {
                        return Ok(ChipType::Const(value));
                    }
                }
                Err(string.to_string())
            }
        }
    }
}

impl ChipType {
    pub fn is_allowed_in(self, puzzle: Puzzle) -> bool {
        match self {
            ChipType::Clock | ChipType::Delay | ChipType::Discard |
            ChipType::Join | ChipType::Latest | ChipType::Sample |
            ChipType::Break | ChipType::Ram => puzzle.allows_events(),
            ChipType::Button => {
                match puzzle.kind() {
                    PuzzleKind::Command | PuzzleKind::Sandbox => {
                        puzzle.allows_events()
                    }
                    _ => false,
                }
            }
            _ => true,
        }
    }

    /// Returns the width and height of the chip in its default orientation.
    pub fn size(self) -> CoordsSize {
        match self {
            ChipType::Ram => CoordsSize::new(2, 2),
            ChipType::Display => CoordsSize::new(2, 1),
            _ => CoordsSize::new(1, 1),
        }
    }

    fn ports_internal(self) -> &'static [AbstractPort] {
        match self {
            ChipType::Const(_) => {
                &[(PortFlow::Send, PortColor::Behavior, (0, 0), East)]
            }
            ChipType::Not => {
                &[
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), West),
                    (PortFlow::Send, PortColor::Behavior, (0, 0), East),
                ]
            }
            ChipType::And | ChipType::Pack => {
                &[
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), West),
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), South),
                    (PortFlow::Send, PortColor::Behavior, (0, 0), East),
                ]
            }
            ChipType::Unpack => {
                &[
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), West),
                    (PortFlow::Send, PortColor::Behavior, (0, 0), East),
                    (PortFlow::Send, PortColor::Behavior, (0, 0), North),
                ]
            }
            ChipType::Add | ChipType::Compare => {
                &[
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), West),
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), South),
                    (PortFlow::Send, PortColor::Behavior, (0, 0), East),
                    (PortFlow::Send, PortColor::Behavior, (0, 0), North),
                ]
            }
            ChipType::Mux => {
                &[
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), West),
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), South),
                    (PortFlow::Send, PortColor::Behavior, (0, 0), East),
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), North),
                ]
            }
            ChipType::Clock | ChipType::Delay | ChipType::Discard |
            ChipType::Break => {
                &[
                    (PortFlow::Recv, PortColor::Event, (0, 0), West),
                    (PortFlow::Send, PortColor::Event, (0, 0), East),
                ]
            }
            ChipType::Join => {
                &[
                    (PortFlow::Recv, PortColor::Event, (0, 0), West),
                    (PortFlow::Recv, PortColor::Event, (0, 0), South),
                    (PortFlow::Send, PortColor::Event, (0, 0), East),
                ]
            }
            ChipType::Latest => {
                &[
                    (PortFlow::Recv, PortColor::Event, (0, 0), West),
                    (PortFlow::Send, PortColor::Behavior, (0, 0), East),
                ]
            }
            ChipType::Sample => {
                &[
                    (PortFlow::Recv, PortColor::Event, (0, 0), West),
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), South),
                    (PortFlow::Send, PortColor::Event, (0, 0), East),
                ]
            }
            ChipType::Ram => {
                &[
                    (PortFlow::Recv, PortColor::Behavior, (0, 0), West),
                    (PortFlow::Recv, PortColor::Event, (0, 0), North),
                    (PortFlow::Send, PortColor::Behavior, (0, 1), West),
                    (PortFlow::Recv, PortColor::Behavior, (1, 1), East),
                    (PortFlow::Recv, PortColor::Event, (1, 1), South),
                    (PortFlow::Send, PortColor::Behavior, (1, 0), East),
                ]
            }
            ChipType::Display => {
                &[(PortFlow::Recv, PortColor::Behavior, (0, 0), West)]
            }
            ChipType::Button => {
                &[(PortFlow::Send, PortColor::Event, (0, 0), East)]
            }
        }
    }

    fn constraints_internal(self) -> &'static [AbstractConstraint] {
        match self {
            ChipType::Const(value) => {
                match WireSize::min_for_value(value) {
                    WireSize::Two => {
                        &[AbstractConstraint::AtLeast(0, WireSize::Two)]
                    }
                    WireSize::Four => {
                        &[AbstractConstraint::AtLeast(0, WireSize::Four)]
                    }
                    WireSize::Eight => {
                        &[AbstractConstraint::AtLeast(0, WireSize::Eight)]
                    }
                    WireSize::Sixteen => {
                        &[AbstractConstraint::AtLeast(0, WireSize::Sixteen)]
                    }
                    WireSize::ThirtyTwo => {
                        &[AbstractConstraint::AtLeast(0, WireSize::ThirtyTwo)]
                    }
                    _ => &[],
                }
            }
            ChipType::Not | ChipType::Delay | ChipType::Latest |
            ChipType::Break => &[AbstractConstraint::Equal(0, 1)],
            ChipType::And | ChipType::Join => {
                &[
                    AbstractConstraint::Equal(0, 1),
                    AbstractConstraint::Equal(0, 2),
                    AbstractConstraint::Equal(1, 2),
                ]
            }
            ChipType::Add => {
                &[
                    AbstractConstraint::Equal(0, 1),
                    AbstractConstraint::Equal(0, 2),
                    AbstractConstraint::Equal(0, 3),
                    AbstractConstraint::Equal(1, 2),
                    AbstractConstraint::Equal(1, 3),
                    AbstractConstraint::Equal(2, 3),
                ]
            }
            ChipType::Compare => {
                &[
                    AbstractConstraint::Equal(0, 1),
                    AbstractConstraint::Exact(2, WireSize::One),
                    AbstractConstraint::Exact(3, WireSize::One),
                ]
            }
            ChipType::Mux => {
                &[
                    AbstractConstraint::Equal(0, 1),
                    AbstractConstraint::Equal(0, 2),
                    AbstractConstraint::Equal(1, 2),
                    AbstractConstraint::Exact(3, WireSize::One),
                ]
            }
            ChipType::Pack => {
                &[
                    AbstractConstraint::Equal(0, 1),
                    AbstractConstraint::Double(2, 0),
                    AbstractConstraint::Double(2, 1),
                ]
            }
            ChipType::Unpack => {
                &[
                    AbstractConstraint::Equal(1, 2),
                    AbstractConstraint::Double(0, 1),
                    AbstractConstraint::Double(0, 2),
                ]
            }
            ChipType::Clock => {
                &[
                    AbstractConstraint::Exact(0, WireSize::Zero),
                    AbstractConstraint::Exact(1, WireSize::Zero),
                ]
            }
            ChipType::Discard => {
                &[
                    AbstractConstraint::AtLeast(0, WireSize::One),
                    AbstractConstraint::Exact(1, WireSize::Zero),
                ]
            }
            ChipType::Sample => {
                &[
                    AbstractConstraint::Exact(0, WireSize::Zero),
                    AbstractConstraint::Equal(1, 2),
                ]
            }
            ChipType::Ram => {
                &[
                    AbstractConstraint::AtMost(0, WireSize::Eight),
                    AbstractConstraint::AtMost(3, WireSize::Eight),
                    AbstractConstraint::AtLeast(1, WireSize::One),
                    AbstractConstraint::AtLeast(4, WireSize::One),
                    AbstractConstraint::Equal(0, 3),
                    AbstractConstraint::Equal(1, 4),
                    AbstractConstraint::Equal(2, 5),
                    AbstractConstraint::Equal(1, 2),
                    AbstractConstraint::Equal(4, 5),
                ]
            }
            ChipType::Display => &[],
            ChipType::Button => {
                &[AbstractConstraint::Exact(0, WireSize::Zero)]
            }
        }
    }

    pub fn dependencies_internal(self) -> &'static [(usize, usize)] {
        match self {
            ChipType::Const(_) |
            ChipType::Clock |
            ChipType::Delay |
            ChipType::Display |
            ChipType::Button => &[],
            ChipType::Not | ChipType::Discard | ChipType::Latest |
            ChipType::Break => &[(0, 1)],
            ChipType::And | ChipType::Pack | ChipType::Join |
            ChipType::Sample => &[(0, 2), (1, 2)],
            ChipType::Unpack => &[(0, 1), (0, 2)],
            ChipType::Add | ChipType::Compare => {
                &[(0, 2), (1, 2), (0, 3), (1, 3)]
            }
            ChipType::Mux => &[(0, 2), (1, 2), (3, 2)],
            ChipType::Ram => &[(0, 2), (1, 2), (3, 5), (4, 5), (1, 5), (4, 2)],
        }
    }

    pub(super) fn chip_evals(self, coords: Coords,
                             slots: &[(usize, WireSize)],
                             interact: &Rc<RefCell<CircuitInteraction>>)
                             -> Vec<(usize, Box<ChipEval>)> {
        debug_assert_eq!(slots.len(), self.ports_internal().len());
        match self {
            ChipType::Add => {
                let chip_eval = eval::AddChipEval::new(slots[2].1,
                                                       slots[0].0,
                                                       slots[1].0,
                                                       slots[2].0,
                                                       slots[3].0);
                vec![(2, chip_eval)]
            }
            ChipType::And => {
                let chip_eval =
                    eval::AndChipEval::new(slots[0].0, slots[1].0, slots[2].0);
                vec![(2, chip_eval)]
            }
            ChipType::Break => {
                let chip_eval =
                    eval::BreakChipEval::new(slots[0].0, slots[1].0, coords);
                vec![(1, chip_eval)]
            }
            ChipType::Button => {
                let chip_eval = eval::ButtonChipEval::new(slots[0].0,
                                                          coords,
                                                          interact.clone());
                vec![(0, chip_eval)]
            }
            ChipType::Clock => {
                vec![(1, eval::ClockChipEval::new(slots[0].0, slots[1].0))]
            }
            ChipType::Compare => {
                let chip_eval = eval::CompareChipEval::new(slots[0].0,
                                                           slots[1].0,
                                                           slots[2].0,
                                                           slots[3].0);
                vec![(2, chip_eval)]
            }
            ChipType::Const(value) => {
                vec![(0, eval::ConstChipEval::new(value, slots[0].0))]
            }
            ChipType::Delay => {
                vec![(1, eval::DelayChipEval::new(slots[0].0, slots[1].0))]
            }
            ChipType::Discard => {
                vec![(1, eval::DiscardChipEval::new(slots[0].0, slots[1].0))]
            }
            ChipType::Display => vec![],
            ChipType::Join => {
                let chip_eval = eval::JoinChipEval::new(slots[0].0,
                                                        slots[1].0,
                                                        slots[2].0);
                vec![(2, chip_eval)]
            }
            ChipType::Latest => {
                vec![(1, eval::LatestChipEval::new(slots[0].0, slots[1].0))]
            }
            ChipType::Mux => {
                let chip_eval = eval::MuxChipEval::new(slots[0].0,
                                                       slots[1].0,
                                                       slots[2].0,
                                                       slots[3].0);
                vec![(2, chip_eval)]
            }
            ChipType::Not => {
                let chip_eval =
                    eval::NotChipEval::new(slots[1].1, slots[0].0, slots[1].0);
                vec![(1, chip_eval)]
            }
            ChipType::Pack => {
                let chip_eval = eval::PackChipEval::new(slots[0].1,
                                                        slots[0].0,
                                                        slots[1].0,
                                                        slots[2].0);
                vec![(2, chip_eval)]
            }
            ChipType::Ram => {
                let addr_size = slots[0].1;
                let num_addrs = 1usize << addr_size.num_bits();
                let storage = Rc::new(RefCell::new(vec![0u32; num_addrs]));
                vec![
                    (2,
                     eval::RamChipEval::new(slots[0].0,
                                            slots[1].0,
                                            slots[2].0,
                                            storage.clone())),
                    (5,
                     eval::RamChipEval::new(slots[3].0,
                                            slots[4].0,
                                            slots[5].0,
                                            storage.clone())),
                ]
            }
            ChipType::Sample => {
                let chip_eval = eval::SampleChipEval::new(slots[0].0,
                                                          slots[1].0,
                                                          slots[2].0);
                vec![(2, chip_eval)]
            }
            ChipType::Unpack => {
                let chip_eval = eval::UnpackChipEval::new(slots[2].1,
                                                          slots[0].0,
                                                          slots[1].0,
                                                          slots[2].0);
                vec![(2, chip_eval)]
            }
        }
    }

    pub fn ports(self, coords: Coords, orient: Orientation) -> Vec<PortSpec> {
        let size = self.size();
        self.ports_internal()
            .iter()
            .map(|&(flow, color, delta, dir)| {
                PortSpec {
                    flow,
                    color,
                    pos: coords + orient.transform_in_rect(delta.into(), size),
                    dir: orient * dir,
                }
            })
            .collect()
    }

    pub fn constraints(self, coords: Coords, orient: Orientation)
                       -> Vec<PortConstraint> {
        let size = self.size();
        let ports = self.ports_internal();
        self.constraints_internal()
            .iter()
            .map(|constraint| constraint.reify(coords, orient, size, ports))
            .collect()
    }

    pub fn dependencies(self, coords: Coords, orient: Orientation)
                        -> Vec<PortDependency> {
        let size = self.size();
        let ports = self.ports_internal();
        self.dependencies_internal()
            .iter()
            .map(|&(recv_index, send_index)| {
                let recv_port = &ports[recv_index];
                let send_port = &ports[send_index];
                debug_assert_eq!(recv_port.0, PortFlow::Recv);
                debug_assert_eq!(send_port.0, PortFlow::Send);
                PortDependency {
                    recv: localize(coords, orient, size, recv_port),
                    send: localize(coords, orient, size, send_port),
                }
            })
            .collect()
    }
}

//===========================================================================//

type AbstractPort = (PortFlow, PortColor, (i32, i32), Direction);

enum AbstractConstraint {
    /// The port must be the given size.
    Exact(usize, WireSize),
    /// The port must be no bigger than the given size.
    AtMost(usize, WireSize),
    /// The port must be no smaller than the given size.
    AtLeast(usize, WireSize),
    /// The two ports must be the same size.
    Equal(usize, usize),
    /// The first port must be double the size of the second port.
    Double(usize, usize),
}

impl AbstractConstraint {
    fn reify(&self, coords: Coords, orient: Orientation, size: CoordsSize,
             ports: &[AbstractPort])
             -> PortConstraint {
        match *self {
            AbstractConstraint::Exact(index, wsize) => {
                let loc = localize(coords, orient, size, &ports[index]);
                PortConstraint::Exact(loc, wsize)
            }
            AbstractConstraint::AtMost(index, wsize) => {
                let loc = localize(coords, orient, size, &ports[index]);
                PortConstraint::AtMost(loc, wsize)
            }
            AbstractConstraint::AtLeast(index, wsize) => {
                let loc = localize(coords, orient, size, &ports[index]);
                PortConstraint::AtLeast(loc, wsize)
            }
            AbstractConstraint::Equal(index1, index2) => {
                let loc1 = localize(coords, orient, size, &ports[index1]);
                let loc2 = localize(coords, orient, size, &ports[index2]);
                PortConstraint::Equal(loc1, loc2)
            }
            AbstractConstraint::Double(index1, index2) => {
                let loc1 = localize(coords, orient, size, &ports[index1]);
                let loc2 = localize(coords, orient, size, &ports[index2]);
                PortConstraint::Double(loc1, loc2)
            }
        }
    }
}

fn localize(coords: Coords, orient: Orientation, size: CoordsSize,
            port: &AbstractPort)
            -> (Coords, Direction) {
    let &(_, _, delta, dir) = port;
    (coords + orient.transform_in_rect(delta.into(), size), orient * dir)
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::ChipType;

    #[test]
    fn chip_type_to_and_from_string() {
        let chip_types = &[
            ChipType::Add,
            ChipType::And,
            ChipType::Break,
            ChipType::Button,
            ChipType::Clock,
            ChipType::Compare,
            ChipType::Const(0),
            ChipType::Const(13),
            ChipType::Const(0xffffffff),
            ChipType::Delay,
            ChipType::Discard,
            ChipType::Display,
            ChipType::Join,
            ChipType::Latest,
            ChipType::Mux,
            ChipType::Not,
            ChipType::Pack,
            ChipType::Ram,
            ChipType::Sample,
            ChipType::Unpack,
        ];
        for &ctype in chip_types.iter() {
            assert_eq!(format!("{:?}", ctype).parse(), Ok(ctype));
        }
    }
}

//===========================================================================//
