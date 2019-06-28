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

use super::super::port::{PortColor, PortConstraint, PortFlow, PortSpec};
use super::super::size::WireSize;
use tachy::geom::{Coords, CoordsDelta, CoordsRect, CoordsSize, Direction};

//===========================================================================//

#[derive(Clone, Copy)]
pub(super) enum InterfacePosition {
    Left(i32),
    Center,
    Right(i32),
}

//===========================================================================//

pub(super) struct InterfacePort {
    pub(super) name: &'static str,
    pub(super) description: &'static str,
    pub(super) flow: PortFlow,
    pub(super) color: PortColor,
    pub(super) size: WireSize,
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
                        min_left.max(min_right) + min_center
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

    pub fn side(&self) -> Direction { self.side }

    pub fn ports(&self, bounds: CoordsRect) -> Vec<(&'static str, PortSpec)> {
        self.ports_with_top_left(self.top_left(bounds))
    }

    pub fn ports_with_top_left(&self, top_left: Coords)
                               -> Vec<(&'static str, PortSpec)> {
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
            format!("$*{}$>({}-bit {} {:?})$<$*\n{}",
                    self.name,
                    port.size.num_bits(),
                    port.color.tooltip_format(),
                    port.flow,
                    self.description)
        } else {
            let mut fmt = format!("$*{}$*\n{}\n", self.name, self.description);
            for port in self.ports.iter() {
                fmt.push_str(&format!("\n$*{}$>({}-bit {} {:?})$<$*",
                                      port.name,
                                      port.size.num_bits(),
                                      port.color.tooltip_format(),
                                      port.flow));
                if !port.description.is_empty() {
                    fmt.push_str(&format!("\n  $!{}", port.description));
                }
            }
            fmt
        }
    }
}

//===========================================================================//

// TODO: add tests for Interface positioning methods

//===========================================================================//
