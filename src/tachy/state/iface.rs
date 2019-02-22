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

use super::port::{PortColor, PortConstraint, PortFlow, PortSpec};
use super::size::WireSize;
use tachy::geom::{Coords, CoordsDelta, CoordsRect, CoordsSize, Direction};
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
    name: &'static str,
    description: &'static str,
    flow: PortFlow,
    color: PortColor,
    size: WireSize,
}

//===========================================================================//

pub struct Interface {
    name: &'static str,
    description: &'static str,
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

    pub fn tooltip_format(&self) -> String {
        if self.ports.len() == 1 && self.ports[0].name.is_empty() {
            let port = &self.ports[0];
            format!("$*{}$>({}-bit {} {:?})$<$*\n{}",
                    self.name,
                    port.size.num_bits(),
                    port.color.tooltip_format(),
                    port.flow,
                    self.description)
        } else {
            let mut fmt = format!("$*{}$*\n{}\n", self.name, self.description);
            for port in self.ports.iter() {
                fmt.push_str(&format!("\n$*{}$>({}-bit {} {:?})$<$*\n  $!{}",
                                      port.name,
                                      port.size.num_bits(),
                                      port.color.tooltip_format(),
                                      port.flow,
                                      port.description));
            }
            fmt
        }
    }
}

//===========================================================================//

pub fn puzzle_interfaces(puzzle: Puzzle) -> &'static [Interface] {
    match puzzle {
        Puzzle::TutorialOr => {
            &[
                Interface {
                    name: "Input1",
                    description: "First input (0 or 1).",
                    side: Direction::West,
                    pos: InterfacePosition::Center,
                    ports: &[
                        InterfacePort {
                            name: "",
                            description: "",
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::One,
                        },
                    ],
                },
                Interface {
                    name: "Input2",
                    description: "Second input (0 or 1).",
                    side: Direction::South,
                    pos: InterfacePosition::Center,
                    ports: &[
                        InterfacePort {
                            name: "",
                            description: "",
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::One,
                        },
                    ],
                },
                Interface {
                    name: "Output",
                    description: "\
                        Should be 1 if either input is 1.\n\
                        Should be 0 if both inputs are 0.",
                    side: Direction::East,
                    pos: InterfacePosition::Center,
                    ports: &[
                        InterfacePort {
                            name: "",
                            description: "",
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
                    name: "Sensor Interface",
                    description: "\
                        Connects to a photosensor array that determines the \
                        ideal position for the heliostat mirror.  Use the \
                        motor interface to move the mirror to this position.",
                    side: Direction::South,
                    pos: InterfacePosition::Left(0),
                    ports: &[
                        InterfacePort {
                            name: "XGoal",
                            description: "Outputs ideal X position.",
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                        InterfacePort {
                            name: "YGoal",
                            description: "Outputs ideal Y position.",
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                    ],
                },
                Interface {
                    name: "Motor Interface",
                    description: "\
                        Connects to a stepper motor that controls the \
                        position of the heliostat mirror.",
                    side: Direction::South,
                    pos: InterfacePosition::Right(0),
                    ports: &[
                        InterfacePort {
                            name: "XPos",
                            description: "Outputs current X position.",
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                        InterfacePort {
                            name: "YPos",
                            description: "Outputs current Y position.",
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Four,
                        },
                        InterfacePort {
                            name: "Motor",
                            description: "\
                                Receives motor commands.\n    \
                                Send 8 to move up.\n    \
                                Send 4 to move down.\n    \
                                Send 2 to move left.\n    \
                                Send 1 to move right.\n  \
                                Send any other value to not move.",
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
                    name: "Timer Interface",
                    description: "Connected to a digital timer.",
                    side: Direction::West,
                    pos: InterfacePosition::Right(0),
                    ports: &[
                        InterfacePort {
                            name: "Time",
                            description: "Outputs the current time step.",
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
                    name: "Timer Interface",
                    description: "Connected to a digital timer.",
                    side: Direction::West,
                    pos: InterfacePosition::Right(0),
                    ports: &[
                        InterfacePort {
                            name: "Time",
                            description: "Outputs the current time step.",
                            flow: PortFlow::Send,
                            color: PortColor::Behavior,
                            size: WireSize::Eight,
                        },
                        InterfacePort {
                            name: "Metronome",
                            description: "Sends an event at the beginning of \
                                          each time step.",
                            flow: PortFlow::Send,
                            color: PortColor::Event,
                            size: WireSize::Zero,
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
