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

use super::geom::{Coords, CoordsDelta, CoordsRect, CoordsSize, Direction};
use super::port::{PortColor, PortConstraint, PortFlow, PortSpec};
use super::size::WireSize;
use tachy::save::Puzzle;

//===========================================================================//

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum InterfacePosition {
    Left(i32),
    Center,
    Right(i32),
}

//===========================================================================//

struct InterfacePort {
    // TODO name and description for help box
    flow: PortFlow,
    color: PortColor,
    size: WireSize,
}

//===========================================================================//

pub struct Interface {
    // TODO: name and description for help box
    side: Direction,
    pos: InterfacePosition,
    ports: &'static [InterfacePort],
}

impl Interface {
    pub fn top_left(&self, bounds: CoordsRect) -> Coords {
        let span = match self.side {
            Direction::East | Direction::West => bounds.height,
            Direction::South | Direction::North => bounds.width,
        };
        let len = self.ports.len() as i32;
        let dist = match self.pos {
            InterfacePosition::Left(d) => d,
            InterfacePosition::Center => (span - len) / 2,
            InterfacePosition::Right(d) => span - len - d,
        };
        let delta = match self.side {
            Direction::East => {
                CoordsDelta::new(bounds.width, span - len - dist)
            }
            Direction::South => CoordsDelta::new(dist, bounds.height),
            Direction::West => CoordsDelta::new(-1, dist),
            Direction::North => CoordsDelta::new(span - len - dist, -1),
        };
        bounds.top_left() + delta
    }

    pub fn size(&self) -> CoordsSize {
        let len = self.ports.len() as i32;
        let size = match self.side {
            Direction::East | Direction::West => (1, len),
            Direction::South | Direction::North => (len, 1),
        };
        size.into()
    }

    pub fn ports(&self, bounds: CoordsRect) -> Vec<PortSpec> {
        self.ports_with_top_left(self.top_left(bounds))
    }

    pub fn ports_with_top_left(&self, top_left: Coords) -> Vec<PortSpec> {
        let delta = self.side.rotate_ccw().delta();
        let start = match self.side {
            Direction::East | Direction::North => {
                top_left - delta * ((self.ports.len() as i32) - 1)
            }
            Direction::South | Direction::West => top_left,
        };
        let port_dir = -self.side;
        self.ports
            .iter()
            .enumerate()
            .map(|(index, port)| {
                     PortSpec {
                         flow: port.flow,
                         color: port.color,
                         pos: start + delta * (index as i32),
                         dir: port_dir,
                     }
                 })
            .collect()
    }

    pub fn constraints(&self, bounds: CoordsRect) -> Vec<PortConstraint> {
        self.ports(bounds)
            .into_iter()
            .enumerate()
            .map(|(index, port)| {
                     PortConstraint::Exact(port.loc(), self.ports[index].size)
                 })
            .collect()
    }
}

//===========================================================================//

pub fn puzzle_interfaces(puzzle: Puzzle) -> &'static [Interface] {
    match puzzle {
        Puzzle::TutorialOr => {
            &[
                Interface {
                    side: Direction::West,
                    pos: InterfacePosition::Center,
                    ports: &[
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::One,
                        },
                    ],
                },
                Interface {
                    side: Direction::South,
                    pos: InterfacePosition::Center,
                    ports: &[
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::One,
                        },
                    ],
                },
                Interface {
                    side: Direction::East,
                    pos: InterfacePosition::Center,
                    ports: &[
                        InterfacePort {
                            flow: PortFlow::Recv,
                            color: PortColor::Behavior,
                            size: WireSize::One,
                        },
                    ],
                },
            ]
        }
        Puzzle::AutomateHeliostat => {
            &[
                Interface {
                    side: Direction::South,
                    pos: InterfacePosition::Left(0),
                    ports: &[
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                    ],
                },
                Interface {
                    side: Direction::South,
                    pos: InterfacePosition::Right(0),
                    ports: &[
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                        InterfacePort {
                            flow: PortFlow::Recv,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                    ],
                },
            ]
        }
        Puzzle::SandboxBehavior => {
            &[
                Interface {
                    side: Direction::West,
                    pos: InterfacePosition::Right(0),
                    ports: &[
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Eight,
                        },
                    ],
                },
            ]
        }
        Puzzle::SandboxEvent => {
            &[
                Interface {
                    side: Direction::West,
                    pos: InterfacePosition::Right(0),
                    ports: &[
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Event,
                            size: WireSize::Zero,
                        },
                        InterfacePort {
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Eight,
                        },
                    ],
                },
            ]
        }
    }
}

//===========================================================================//

// TODO: add tests for Interface positioning methods

//===========================================================================//
