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

use cgmath::{Matrix4, vec2, vec3};
use tachy::font::Align;
use tachy::gui::Resources;
use tachy::state::{ChipType, Coords, EditGrid, Orientation, PortColor,
                   PortFlow, RectSize};

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
    Not,
    Pack1,
    Pack2,
    Unpack1,
    Unpack2,
    Blank,
}

//===========================================================================//

pub struct ChipModel {}

impl ChipModel {
    pub fn new() -> ChipModel { ChipModel {} }

    pub fn draw_chip(&self, resources: &Resources, matrix: &Matrix4<f32>,
                     ctype: ChipType, orient: Orientation,
                     coords_and_grid: Option<(Coords, &EditGrid)>) {
        let size = orient * ctype.size();

        for port in ctype.ports((0, 0).into(), orient) {
            let x = port.pos.x as f32 + 0.5;
            let y = port.pos.y as f32 + 0.5;
            let angle = port.dir.angle_from_east();
            let mat = matrix * Matrix4::from_translation(vec3(x, y, 0.0)) *
                Matrix4::from_axis_angle(vec3(0.0, 0.0, 1.0), angle);
            let color = match (port.color, port.flow) {
                (PortColor::Behavior, PortFlow::Send) => (1.0, 0.5, 0.0),
                (PortColor::Behavior, PortFlow::Recv) => (0.75, 0.0, 0.0),
                (PortColor::Event, PortFlow::Send) => (0.0, 1.0, 1.0),
                (PortColor::Event, PortFlow::Recv) => (0.0, 0.0, 1.0),
            };
            let rect = (0.5 - MARGIN, -0.3, 0.5 * MARGIN, 0.6);
            resources.shaders().solid().fill_rect(&mat, color, rect);
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
            ChipType::Not => ChipIcon::Not,
            ChipType::Pack => {
                if orient.is_mirrored() {
                    ChipIcon::Pack2
                } else {
                    ChipIcon::Pack1
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
                      orient: Orientation, size: RectSize<i32>,
                      icon: ChipIcon) {
        let width = size.width as f32 - 2.0 * MARGIN;
        let height = size.height as f32 - 2.0 * MARGIN;
        let matrix = matrix *
            Matrix4::from_translation(vec3(MARGIN, MARGIN, 0.0)) *
            Matrix4::from_nonuniform_scale(width, height, 1.0) *
            Matrix4::from_translation(vec3(0.5, 0.5, 0.0)) *
            orient.matrix() *
            Matrix4::from_translation(vec3(-0.5, -0.5, 0.0));
        let icon_index = icon as u32;
        let icon_coords = vec2(icon_index % 8, icon_index / 8);
        resources.textures().chip_icons().bind();
        resources.shaders().chip().draw(&matrix, icon_coords);
    }

    fn draw_chip_string(&self, resources: &Resources, matrix: &Matrix4<f32>,
                        size: RectSize<i32>, string: &str) {
        let (width, height) = (size.width as f32, size.height as f32);
        resources.fonts().roman().draw(matrix,
                                       (0.15, 0.3),
                                       Align::Center,
                                       (0.5 * width, 0.5 * height - 0.15),
                                       string);
    }
}

//===========================================================================//
