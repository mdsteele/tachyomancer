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

use cgmath::Matrix4;
use tachy::font::{Align, Font};
use tachy::geom::{AsFloat, Color3, Color4, Coords, CoordsSize, Direction,
                  MatrixExt, Orientation, Rect};
use tachy::gui::Resources;
use tachy::save::ChipType;
use tachy::state::{ChipExt, EditGrid, Interface, PortColor, PortFlow,
                   PortSpec, WireSize};

//===========================================================================//

const INTERFACE_LABEL_COLOR: Color4 = Color4::new(0.75, 0.75, 0.75, 1.0);

//===========================================================================//

/// The margin around chip rects, in grid cell units.
pub const CHIP_MARGIN: f32 = 0.12;

pub fn chip_grid_rect(chip_coords: Coords, ctype: ChipType,
                      orient: Orientation)
                      -> Rect<f32> {
    Rect::with_size(chip_coords, orient * ctype.size())
        .as_f32()
        .expand(-CHIP_MARGIN)
}

pub fn interface_grid_rect(iface_coords: Coords, interface: &Interface)
                           -> Rect<f32> {
    Rect::with_size(iface_coords, interface.size())
        .as_f32()
        .expand(-CHIP_MARGIN)
}

//===========================================================================//

// Generated code:
// enum ChipIcon { ... }
include!(concat!(env!("OUT_DIR"), "/texture/chip_icons.rs"));

//===========================================================================//

pub struct ChipModel {}

impl ChipModel {
    pub fn draw_interface(resources: &Resources, matrix: &Matrix4<f32>,
                          interface: &Interface) {
        // Draw ports:
        let ports = interface.ports_with_top_left(Coords::new(0, 0));
        for &(_, ref port) in ports.iter() {
            draw_port(resources, matrix, port);
        }

        // Draw body:
        let size = interface.size();
        let width = size.width as f32 - 2.0 * CHIP_MARGIN;
        let height = size.height as f32 - 2.0 * CHIP_MARGIN;
        let color = Color3::new(0.3, 0.3, 0.3);
        let rect = Rect::new(CHIP_MARGIN, CHIP_MARGIN, width, height);
        resources.shaders().solid().fill_rect(matrix, color, rect);

        // Draw port labels:
        let font = resources.fonts().roman();
        for &(name, ref port) in ports.iter() {
            font.draw_style(matrix,
                            0.25,
                            Align::MidCenter,
                            (0.5 + (port.coords.x as f32),
                             0.5 + (port.coords.y as f32)),
                            &INTERFACE_LABEL_COLOR,
                            0.0,
                            name);
        }
    }

    pub fn draw_chip(resources: &Resources, matrix: &Matrix4<f32>,
                     ctype: ChipType, orient: Orientation,
                     coords_and_grid: Option<(Coords, &EditGrid)>) {
        let size = orient * ctype.size();

        for port in ctype.ports(Coords::new(0, 0), orient) {
            draw_port(resources, matrix, &port);
        }

        let icon = chip_icon(ctype, orient);
        draw_chip_icon(resources, matrix, orient, size, icon);

        match ctype {
            ChipType::Const(value) => {
                let label = value.to_string();
                let font_size = 0.5 /
                    ((label.len() as f32) * Font::Roman.ratio()).max(1.0);
                draw_chip_string(resources,
                                 matrix,
                                 size,
                                 font_size,
                                 &Color4::ORANGE4,
                                 &label);
            }
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
                    draw_chip_string(resources,
                                     matrix,
                                     size,
                                     0.3,
                                     &Color4::WHITE,
                                     &format!("{}", value));
                } else {
                    draw_chip_string(resources,
                                     matrix,
                                     size,
                                     0.3,
                                     &Color4::WHITE,
                                     "Display");
                };
            }
            _ => {
                if icon == ChipIcon::Blank {
                    draw_chip_string(resources,
                                     matrix,
                                     size,
                                     0.3,
                                     &Color4::WHITE,
                                     &format!("{:?}", ctype));
                }
            }
        }
    }
}

fn chip_icon(ctype: ChipType, orient: Orientation) -> ChipIcon {
    match ctype {
        ChipType::Add => ChipIcon::Add,
        ChipType::Add2Bit => {
            if orient.is_mirrored() {
                ChipIcon::Add2bit2
            } else {
                ChipIcon::Add2bit1
            }
        }
        ChipType::And => ChipIcon::And,
        ChipType::Clock => ChipIcon::Clock,
        ChipType::Cmp => ChipIcon::Cmp,
        ChipType::CmpEq => {
            let flip = match orient * Direction::North {
                Direction::North | Direction::East => false,
                Direction::South | Direction::West => true,
            };
            if flip {
                ChipIcon::Cmpeq2
            } else {
                ChipIcon::Cmpeq1
            }
        }
        ChipType::Const(_) => ChipIcon::Const,
        ChipType::Delay => ChipIcon::Delay,
        ChipType::Demux => ChipIcon::Demux,
        ChipType::Discard => ChipIcon::Discard,
        ChipType::Eq => ChipIcon::Eq,
        ChipType::Filter => ChipIcon::Filter,
        ChipType::Halve => ChipIcon::Halve,
        ChipType::Inc => ChipIcon::Inc,
        ChipType::Join => ChipIcon::Join,
        ChipType::Latest => ChipIcon::Latest,
        ChipType::Mul => ChipIcon::Mul,
        ChipType::Mul4Bit => {
            if orient.is_mirrored() {
                ChipIcon::Mul4bit2
            } else {
                ChipIcon::Mul4bit1
            }
        }
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
        ChipType::Sample => ChipIcon::Sample,
        ChipType::Sub => ChipIcon::Sub,
        ChipType::Unpack => {
            if orient.is_mirrored() {
                ChipIcon::Unpack2
            } else {
                ChipIcon::Unpack1
            }
        }
        ChipType::Xor => ChipIcon::Xor,
        _ => ChipIcon::Blank,
    }
}

fn chip_icon_color(chip_icon: ChipIcon) -> Color3 {
    match chip_icon {
        ChipIcon::Clock | ChipIcon::Delay | ChipIcon::Demux |
        ChipIcon::Discard | ChipIcon::Filter | ChipIcon::Inc |
        ChipIcon::Join | ChipIcon::Latest | ChipIcon::Sample => Color3::CYAN4,
        _ => Color3::ORANGE4,
    }
}

fn chip_icon_is_fixed(chip_icon: ChipIcon) -> bool {
    match chip_icon {
        ChipIcon::Halve | ChipIcon::Sub => true,
        _ => false,
    }
}

fn draw_chip_icon(resources: &Resources, matrix: &Matrix4<f32>,
                  orient: Orientation, size: CoordsSize, icon: ChipIcon) {
    let width = size.width as f32 - 2.0 * CHIP_MARGIN;
    let height = size.height as f32 - 2.0 * CHIP_MARGIN;
    let orient = if chip_icon_is_fixed(icon) {
        Orientation::default()
    } else {
        orient
    };
    let matrix = matrix * Matrix4::trans2(CHIP_MARGIN, CHIP_MARGIN) *
        Matrix4::scale2(width, height) *
        Matrix4::trans2(0.5, 0.5) * orient.matrix() *
        Matrix4::trans2(-0.5, -0.5);
    let icon_index = icon as u32;
    let icon_color = chip_icon_color(icon);
    resources.shaders().chip().draw_basic(&matrix,
                                          icon_index,
                                          icon_color,
                                          resources.textures().chip_icons());
}

fn draw_chip_string(resources: &Resources, matrix: &Matrix4<f32>,
                    chip_size: CoordsSize, font_size: f32, color: &Color4,
                    string: &str) {
    let (width, height) = (chip_size.width as f32, chip_size.height as f32);
    let font = resources.fonts().roman();
    font.draw_style(matrix,
                    font_size,
                    Align::MidCenter,
                    (0.5 * width, 0.5 * height),
                    color,
                    0.0,
                    string);
}

fn draw_port(resources: &Resources, matrix: &Matrix4<f32>, port: &PortSpec) {
    let x = port.coords.x as f32 + 0.5;
    let y = port.coords.y as f32 + 0.5;
    let mat = matrix * Matrix4::trans2(x, y) *
        Matrix4::from_angle_z(port.dir.angle_from_east()) *
        Matrix4::scale2(0.5, 0.3);

    let shader = resources.shaders().port();
    shader.bind();
    shader.set_mvp(&mat);
    shader.set_port_flow_and_color(port.flow == PortFlow::Send,
                                   port.color == PortColor::Event);
    shader.set_texture(resources.textures().brushed_metal());
    let width_scale = match port.max_size {
        WireSize::Zero => 0.25,
        WireSize::One => 0.4,
        WireSize::Two => 0.5,
        WireSize::Four => 0.65,
        WireSize::Eight => 0.8,
        WireSize::Sixteen => 1.0,
    };
    shader.draw(width_scale);
}

//===========================================================================//
