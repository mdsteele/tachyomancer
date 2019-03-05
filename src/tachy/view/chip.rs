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

use cgmath::{Matrix4, vec2};
use tachy::font::Align;
use tachy::geom::{Coords, CoordsSize, Direction, MatrixExt, Orientation, Rect};
use tachy::gui::Resources;
use tachy::save::ChipType;
use tachy::state::{ChipExt, EditGrid, Interface, PortColor, PortFlow,
                   PortSpec};

//===========================================================================//

const MARGIN: f32 = 0.1;

//===========================================================================//

// These must match the list of SVG files in src/tacy/texture/chip/, and must
// appear in alphabetical order (except for Blank, which should come last).
// TODO: Have build.rs generate this declaration.
#[derive(Clone, Copy, Eq, PartialEq)]
enum ChipIcon {
    Add = 0,
    And,
    Cmp,
    CmpEq1,
    CmpEq2,
    Eq,
    Mux,
    Not,
    Or,
    Pack1,
    Pack2,
    Sub1,
    Sub2,
    Unpack1,
    Unpack2,
    Blank,
}

//===========================================================================//

pub struct ChipModel {}

impl ChipModel {
    pub fn new() -> ChipModel { ChipModel {} }

    pub fn draw_interface(&self, resources: &Resources,
                          matrix: &Matrix4<f32>, interface: &Interface) {
        let size = interface.size();
        for port in interface.ports_with_top_left(Coords::new(0, 0)) {
            self.draw_port(resources, matrix, &port);
        }
        let width = size.width as f32 - 2.0 * MARGIN;
        let height = size.height as f32 - 2.0 * MARGIN;
        let color = (0.3, 0.3, 0.3);
        let rect = Rect::new(MARGIN, MARGIN, width, height);
        resources.shaders().solid().fill_rect(matrix, color, rect);
    }

    pub fn draw_chip(&self, resources: &Resources, matrix: &Matrix4<f32>,
                     ctype: ChipType, orient: Orientation,
                     coords_and_grid: Option<(Coords, &EditGrid)>) {
        let size = orient * ctype.size();

        for port in ctype.ports(Coords::new(0, 0), orient) {
            self.draw_port(resources, matrix, &port);
        }

        let icon = self.chip_icon(ctype, orient);
        self.draw_chip_icon(resources, matrix, orient, size, icon);

        match ctype {
            ChipType::Display => {
                let mut opt_value: Option<u32> = None;
                if let Some((coords, grid)) = coords_and_grid {
                    if grid.eval().is_some() {
                        let ports = ctype.ports(coords, orient);
                        debug_assert_eq!(ports.len(), 1);
                        opt_value = grid.port_value(ports[0].loc())
                    }
                }
                if let Some(value) = opt_value {
                    self.draw_chip_string(resources,
                                          matrix,
                                          size,
                                          &format!("{}", value));
                } else {
                    self.draw_chip_string(resources, matrix, size, "Display");
                };
            }
            _ => {
                if icon == ChipIcon::Blank {
                    self.draw_chip_string(resources,
                                          matrix,
                                          size,
                                          &format!("{:?}", ctype));
                }
            }
        }
    }

    fn chip_icon(&self, ctype: ChipType, orient: Orientation) -> ChipIcon {
        match ctype {
            ChipType::Add => ChipIcon::Add,
            ChipType::And => ChipIcon::And,
            ChipType::Cmp => ChipIcon::Cmp,
            ChipType::CmpEq => {
                let flip = match orient * Direction::North {
                    Direction::North | Direction::East => false,
                    Direction::South | Direction::West => true,
                };
                if flip {
                    ChipIcon::CmpEq2
                } else {
                    ChipIcon::CmpEq1
                }
            }
            ChipType::Eq => ChipIcon::Eq,
            ChipType::Mux => ChipIcon::Mux,
            ChipType::Not => ChipIcon::Not,
            ChipType::Or => ChipIcon::Or,
            ChipType::Pack => {
                if orient.is_mirrored() {
                    ChipIcon::Pack2
                } else {
                    ChipIcon::Pack1
                }
            }
            ChipType::Sub => {
                if orient.is_rotated_90() {
                    ChipIcon::Sub2
                } else {
                    ChipIcon::Sub1
                }
            }
            ChipType::Unpack => {
                if orient.is_mirrored() {
                    ChipIcon::Unpack2
                } else {
                    ChipIcon::Unpack1
                }
            }
            _ => ChipIcon::Blank,
        }
    }

    fn draw_chip_icon(&self, resources: &Resources, matrix: &Matrix4<f32>,
                      orient: Orientation, size: CoordsSize, icon: ChipIcon) {
        let width = size.width as f32 - 2.0 * MARGIN;
        let height = size.height as f32 - 2.0 * MARGIN;
        let matrix = matrix * Matrix4::trans2(MARGIN, MARGIN) *
            Matrix4::scale2(width, height) *
            Matrix4::trans2(0.5, 0.5) *
            orient.matrix() * Matrix4::trans2(-0.5, -0.5);
        let icon_index = icon as u32;
        let icon_coords = vec2(icon_index % 8, icon_index / 8);
        resources.textures().chip_icons().bind();
        resources.shaders().chip().draw(&matrix, icon_coords);
    }

    fn draw_chip_string(&self, resources: &Resources, matrix: &Matrix4<f32>,
                        size: CoordsSize, string: &str) {
        let (width, height) = (size.width as f32, size.height as f32);
        resources.fonts().roman().draw(matrix,
                                       0.3,
                                       Align::MidCenter,
                                       (0.5 * width, 0.5 * height),
                                       string);
    }

    fn draw_port(&self, resources: &Resources, matrix: &Matrix4<f32>,
                 port: &PortSpec) {
        let x = port.pos.x as f32 + 0.5;
        let y = port.pos.y as f32 + 0.5;
        let mat = matrix * Matrix4::trans2(x, y) *
            Matrix4::from_angle_z(port.dir.angle_from_east());
        let color = match (port.color, port.flow) {
            (PortColor::Behavior, PortFlow::Send) => (1.0, 0.5, 0.0),
            (PortColor::Behavior, PortFlow::Recv) => (0.75, 0.0, 0.0),
            (PortColor::Event, PortFlow::Send) => (0.0, 1.0, 1.0),
            (PortColor::Event, PortFlow::Recv) => (0.0, 0.0, 1.0),
        };
        let rect = Rect::new(0.5 - MARGIN, -0.3, 0.5 * MARGIN, 0.6);
        resources.shaders().solid().fill_rect(&mat, color, rect);
    }
}

//===========================================================================//
