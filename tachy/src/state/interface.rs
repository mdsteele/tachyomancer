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

use crate::geom::{Coords, CoordsDelta, CoordsRect, CoordsSize, Direction};
use crate::save::WireSize;
use crate::state::{PortColor, PortConstraint, PortFlow, PortSpec};

//===========================================================================//

#[derive(Clone, Copy)]
pub(super) enum InterfacePosition {
    Left(i32),
    Center,
    Right(i32),
}

//===========================================================================//

pub struct InterfacePort {
    pub name: &'static str,
    pub description: &'static str,
    pub flow: PortFlow,
    pub color: PortColor,
    pub size: WireSize,
}

//===========================================================================//

pub struct Interface {
    pub(super) name: &'static str,
    pub(super) description: &'static str,
    pub(super) side: Direction,
    pub(super) pos: InterfacePosition,
    pub(super) ports: &'static [InterfacePort],
}

impl Interface {
    pub fn min_bounds_size(interfaces: &[Interface]) -> CoordsSize {
        let mut min_width: i32 = 1;
        let mut min_height: i32 = 1;
        for dir in Direction::all() {
            let mut min_left: i32 = 0;
            let mut min_center: i32 = 0;
            let mut min_right: i32 = 0;
            for interface in interfaces.iter() {
                if interface.side == dir {
                    let num_ports = interface.ports.len() as i32;
                    match interface.pos {
                        InterfacePosition::Left(offset) => {
                            min_left = min_left.max(num_ports + offset);
                        }
                        InterfacePosition::Center => {
                            min_center = min_center.max(num_ports);
                        }
                        InterfacePosition::Right(offset) => {
                            min_right = min_right.max(num_ports + offset);
                        }
                    }
                    let min_size = if min_center > 0 {
                        2 * min_left.max(min_right) + min_center
                    } else {
                        min_left + min_right
                    };
                    if dir.is_vertical() {
                        min_width = min_width.max(min_size);
                    } else {
                        min_height = min_height.max(min_size);
                    }
                }
            }
        }
        CoordsSize::new(min_width, min_height)
    }

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

    pub fn side(&self) -> Direction {
        self.side
    }

    pub fn ports(&self, bounds: CoordsRect) -> Vec<(&'static str, PortSpec)> {
        self.ports_with_top_left(self.top_left(bounds))
    }

    pub fn ports_with_top_left(
        &self,
        top_left: Coords,
    ) -> Vec<(&'static str, PortSpec)> {
        let delta = match self.side {
            Direction::South | Direction::West => {
                self.side.rotate_ccw().delta()
            }
            Direction::East | Direction::North => {
                self.side.rotate_cw().delta()
            }
        };
        let port_dir = -self.side;
        self.ports
            .iter()
            .enumerate()
            .map(|(index, port)| {
                let spec = PortSpec {
                    flow: port.flow,
                    color: port.color,
                    coords: top_left + delta * (index as i32),
                    dir: port_dir,
                    max_size: port.size,
                };
                (port.name, spec)
            })
            .collect()
    }

    pub fn constraints(&self, bounds: CoordsRect) -> Vec<PortConstraint> {
        self.ports(bounds)
            .into_iter()
            .enumerate()
            .map(|(index, (_, port))| {
                PortConstraint::Exact(port.loc(), self.ports[index].size)
            })
            .collect()
    }

    pub fn tooltip_format(&self) -> String {
        if self.ports.len() == 1 && self.ports[0].description.is_empty() {
            let port = &self.ports[0];
            if port.color == PortColor::Analog {
                format!(
                    "$*{}$>({} {})$<$*\n{}",
                    self.name,
                    port.color.tooltip_format(),
                    port.flow.tooltip_format(),
                    self.description
                )
            } else {
                format!(
                    "$*{}$>({}-bit {} {})$<$*\n{}",
                    self.name,
                    port.size.num_bits(),
                    port.color.tooltip_format(),
                    port.flow.tooltip_format(),
                    self.description
                )
            }
        } else {
            let mut fmt = format!("$*{}$*\n{}\n", self.name, self.description);
            for port in self.ports.iter() {
                if port.color == PortColor::Analog {
                    fmt.push_str(&format!(
                        "\n$*{}$>({} {})$<$*",
                        port.name,
                        port.color.tooltip_format(),
                        port.flow.tooltip_format(),
                    ));
                } else {
                    fmt.push_str(&format!(
                        "\n$*{}$>({}-bit {} {})$<$*",
                        port.name,
                        port.size.num_bits(),
                        port.color.tooltip_format(),
                        port.flow.tooltip_format(),
                    ));
                }
                if !port.description.is_empty() {
                    fmt.push_str(&format!("\n  $!{}", port.description));
                }
            }
            fmt
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Interface, InterfacePort, InterfacePosition};
    use crate::geom::{Coords, CoordsRect, CoordsSize, Direction};
    use crate::save::WireSize;
    use crate::state::{PortColor, PortFlow};

    #[test]
    fn interface_positioning() {
        let mut interface = Interface {
            name: "Foobar",
            description: "",
            side: Direction::North,
            pos: InterfacePosition::Center,
            ports: &[
                InterfacePort {
                    name: "Foo",
                    description: "",
                    flow: PortFlow::Source,
                    color: PortColor::Event,
                    size: WireSize::One,
                },
                InterfacePort {
                    name: "Bar",
                    description: "",
                    flow: PortFlow::Source,
                    color: PortColor::Event,
                    size: WireSize::Two,
                },
            ],
        };
        let rect = CoordsRect::new(-1, 3, 8, 7);

        interface.side = Direction::East;
        interface.pos = InterfacePosition::Left(0);
        assert_eq!(interface.top_left(rect), Coords::new(7, 8));
        interface.pos = InterfacePosition::Right(1);
        assert_eq!(interface.top_left(rect), Coords::new(7, 4));
        interface.pos = InterfacePosition::Center;
        assert_eq!(interface.top_left(rect), Coords::new(7, 6));
        assert_eq!(interface.size(), CoordsSize::new(1, 2));

        interface.side = Direction::West;
        interface.pos = InterfacePosition::Left(0);
        assert_eq!(interface.top_left(rect), Coords::new(-2, 3));
        interface.pos = InterfacePosition::Right(1);
        assert_eq!(interface.top_left(rect), Coords::new(-2, 7));
        interface.pos = InterfacePosition::Center;
        assert_eq!(interface.top_left(rect), Coords::new(-2, 5));
        assert_eq!(interface.size(), CoordsSize::new(1, 2));

        interface.side = Direction::North;
        interface.pos = InterfacePosition::Left(1);
        assert_eq!(interface.top_left(rect), Coords::new(4, 2));
        interface.pos = InterfacePosition::Right(0);
        assert_eq!(interface.top_left(rect), Coords::new(-1, 2));
        interface.pos = InterfacePosition::Center;
        assert_eq!(interface.top_left(rect), Coords::new(2, 2));
        assert_eq!(interface.size(), CoordsSize::new(2, 1));

        interface.side = Direction::South;
        interface.pos = InterfacePosition::Left(1);
        assert_eq!(interface.top_left(rect), Coords::new(0, 10));
        interface.pos = InterfacePosition::Right(0);
        assert_eq!(interface.top_left(rect), Coords::new(5, 10));
        interface.pos = InterfacePosition::Center;
        assert_eq!(interface.top_left(rect), Coords::new(2, 10));
        assert_eq!(interface.size(), CoordsSize::new(2, 1));
    }

    #[test]
    fn interface_min_bounds_size() {
        let ports = &[
            InterfacePort {
                name: "Foo",
                description: "",
                flow: PortFlow::Source,
                color: PortColor::Event,
                size: WireSize::One,
            },
            InterfacePort {
                name: "Bar",
                description: "",
                flow: PortFlow::Source,
                color: PortColor::Event,
                size: WireSize::Two,
            },
        ];
        let interfaces = vec![
            Interface {
                name: "Spam",
                description: "",
                side: Direction::North,
                pos: InterfacePosition::Right(0),
                ports,
            },
            Interface {
                name: "Eggs",
                description: "",
                side: Direction::North,
                pos: InterfacePosition::Center,
                ports,
            },
            Interface {
                name: "Bacon",
                description: "",
                side: Direction::North,
                pos: InterfacePosition::Left(0),
                ports,
            },
            Interface {
                name: "Beans",
                description: "",
                side: Direction::West,
                pos: InterfacePosition::Right(1),
                ports,
            },
            Interface {
                name: "Sausage",
                description: "",
                side: Direction::West,
                pos: InterfacePosition::Left(0),
                ports,
            },
        ];
        assert_eq!(
            Interface::min_bounds_size(&interfaces),
            CoordsSize::new(6, 5)
        );
    }
}

//===========================================================================//
