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

use crate::mancer::font::{Align, Font};
use crate::mancer::gui::Resources;
use cgmath::{vec2, vec3, Matrix4, Point2};
use tachy::geom::{
    AsFloat, AsInt, Color3, Color4, Coords, CoordsSize, Direction, MatrixExt,
    Orientation, Rect, RectSize,
};
use tachy::save::{ChipType, WireSize};
use tachy::state::{
    ChipExt, EditGrid, Interface, PortColor, PortFlow, PortSpec,
};

//===========================================================================//

const INTERFACE_LABEL_COLOR: Color4 = Color4::new(0.75, 0.75, 0.75, 1.0);

//===========================================================================//

/// The margin around chip rects, in grid cell units.
pub const CHIP_MARGIN: f32 = 0.12;

pub fn chip_grid_rect(
    chip_coords: Coords,
    ctype: ChipType,
    orient: Orientation,
) -> Rect<f32> {
    Rect::with_size(chip_coords, orient * ctype.size())
        .as_f32()
        .expand(-CHIP_MARGIN)
}

pub fn interface_grid_rect(
    iface_coords: Coords,
    interface: &Interface,
) -> Rect<f32> {
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
    pub fn draw_interface(
        resources: &Resources,
        grid_matrix: &Matrix4<f32>,
        coords: Coords,
        interface: &Interface,
    ) {
        // Draw body:
        draw_basic_chip(
            resources,
            grid_matrix,
            coords,
            Orientation::default(),
            interface.size(),
            ChipIcon::Blank,
        );

        // Draw ports:
        let ports = interface.ports_with_top_left(coords);
        for &(_, ref port) in ports.iter() {
            draw_port(resources, grid_matrix, port);
        }

        // Draw port labels:
        for &(name, ref port) in ports.iter() {
            draw_chip_string(
                resources,
                grid_matrix,
                port.coords,
                CoordsSize::new(1, 1),
                Font::Roman,
                0.25,
                &INTERFACE_LABEL_COLOR,
                Orientation::default(),
                name,
            );
        }
    }

    pub fn draw_chip(
        resources: &Resources,
        grid_matrix: &Matrix4<f32>,
        coords: Coords,
        ctype: ChipType,
        orient: Orientation,
        opt_grid: Option<&EditGrid>,
    ) {
        let chip_size = ctype.size();
        let oriented_size = orient * chip_size;

        match ctype {
            ChipType::Break(mut enabled) => {
                if let Some(grid) = opt_grid {
                    if let Some(eval) = grid.eval() {
                        enabled = eval.display_data(coords)[0] != 0;
                    }
                }
                draw_break_chip(resources, grid_matrix, coords, enabled);
            }
            ChipType::Comment(_)
            | ChipType::DocBv(_, _)
            | ChipType::DocEv(_, _) => {
                draw_comment_chip(
                    resources,
                    grid_matrix,
                    coords,
                    orient,
                    chip_icon(ctype, orient),
                );
            }
            ChipType::Counter => {
                draw_counter_chip(resources, grid_matrix, coords, orient);
            }
            ChipType::Discard => {
                draw_discard_chip(resources, grid_matrix, coords, orient);
            }
            ChipType::Display | ChipType::EggTimer | ChipType::Stopwatch => {
                draw_basic_chip(
                    resources,
                    grid_matrix,
                    coords,
                    orient,
                    chip_size,
                    ChipIcon::Blank,
                );
                let text_orient = match orient * Direction::North {
                    Direction::North | Direction::South => {
                        Orientation::default()
                    }
                    Direction::East | Direction::West => {
                        Orientation::default().rotate_ccw()
                    }
                };
                let mut value: u32 = 0;
                if let Some(grid) = opt_grid {
                    if grid.eval().is_some() {
                        let ports = ctype.ports(coords, orient);
                        let index = match ctype {
                            ChipType::Display => 0,
                            ChipType::EggTimer => 1,
                            ChipType::Stopwatch => 3,
                            _ => unreachable!(),
                        };
                        value =
                            grid.port_value(ports[index].loc()).unwrap_or(0);
                    }
                }
                draw_chip_string(
                    resources,
                    &grid_matrix,
                    coords,
                    oriented_size,
                    Font::Led,
                    0.5,
                    &Color4::YELLOW5,
                    text_orient,
                    &format!("{:05}", value),
                );
            }
            ChipType::Latest => {
                draw_latest_chip(resources, grid_matrix, coords, orient);
            }
            ChipType::Screen => {
                draw_basic_chip(
                    resources,
                    grid_matrix,
                    coords,
                    orient,
                    chip_size,
                    ChipIcon::Blank,
                );
                if let Some(grid) = opt_grid {
                    if let Some(eval) = grid.eval() {
                        draw_chip_screen(
                            resources,
                            grid_matrix,
                            coords,
                            eval.display_data(coords),
                        );
                    }
                }
            }
            _ => {
                let icon = chip_icon(ctype, orient);
                draw_basic_chip(
                    resources,
                    grid_matrix,
                    coords,
                    orient,
                    chip_size,
                    icon,
                );
                if icon == ChipIcon::Blank {
                    draw_chip_string(
                        resources,
                        &grid_matrix,
                        coords,
                        oriented_size,
                        Font::Roman,
                        0.3,
                        &Color4::WHITE,
                        Orientation::default(),
                        &format!("{}", ctype),
                    );
                }
            }
        }

        match ctype {
            ChipType::DocBv(_, _) | ChipType::DocEv(_, _) => {}
            _ => {
                for port in ctype.ports(coords, orient) {
                    draw_port(resources, grid_matrix, &port);
                }
            }
        }

        match ctype {
            ChipType::Comment(bytes)
            | ChipType::DocBv(_, bytes)
            | ChipType::DocEv(_, bytes) => {
                let string: String =
                    bytes.iter().map(|&b| char::from(b)).collect();
                draw_chip_string(
                    resources,
                    &grid_matrix,
                    coords,
                    oriented_size,
                    Font::Roman,
                    0.25,
                    &Color4::WHITE,
                    Orientation::default(),
                    string.trim_end(),
                );
            }
            ChipType::Const(value) => {
                let label = value.to_string();
                let font = Font::Roman;
                let font_size =
                    0.4 / ((label.len() as f32) * font.ratio()).max(1.0);
                draw_chip_string(
                    resources,
                    &grid_matrix,
                    coords,
                    oriented_size,
                    font,
                    font_size,
                    &Color4::ORANGE4,
                    Orientation::default(),
                    &label,
                );
            }
            _ => {}
        }
    }

    pub fn chip_screen_cell(
        coords: Coords,
        grid_pt: Point2<f32>,
    ) -> Option<u32> {
        chip_screen_cell(coords, grid_pt)
    }
}

fn chip_icon(ctype: ChipType, orient: Orientation) -> ChipIcon {
    match ctype {
        ChipType::Add => ChipIcon::Add,
        ChipType::Add2Bit => {
            if orient.is_mirrored() {
                ChipIcon::Add2Bit2
            } else {
                ChipIcon::Add2Bit1
            }
        }
        ChipType::And => ChipIcon::And,
        ChipType::Break(_) => ChipIcon::Break,
        ChipType::Clock => ChipIcon::Clock,
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
        ChipType::Comment(_) => ChipIcon::Comment,
        ChipType::Const(_) => ChipIcon::Const,
        ChipType::Counter => ChipIcon::Counter,
        ChipType::Delay => ChipIcon::Delay,
        ChipType::Demux => ChipIcon::Demux,
        ChipType::Discard => ChipIcon::Discard,
        ChipType::DocBv(_, _) => ChipIcon::DocBv,
        ChipType::DocEv(_, _) => ChipIcon::DocEv,
        ChipType::Eq => ChipIcon::Eq,
        ChipType::Halve => ChipIcon::Halve,
        ChipType::Inc => ChipIcon::Inc,
        ChipType::Join => ChipIcon::Join,
        ChipType::Latest => ChipIcon::Latest,
        ChipType::Mul => ChipIcon::Mul,
        ChipType::Mul4Bit => {
            if orient.is_mirrored() {
                ChipIcon::Mul4Bit2
            } else {
                ChipIcon::Mul4Bit1
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
        ChipType::Random => ChipIcon::Random,
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
        ChipIcon::Clock
        | ChipIcon::Counter
        | ChipIcon::Delay
        | ChipIcon::Demux
        | ChipIcon::Discard
        | ChipIcon::Inc
        | ChipIcon::Join
        | ChipIcon::Latest
        | ChipIcon::Random
        | ChipIcon::Sample => Color3::CYAN4,
        _ => Color3::ORANGE4,
    }
}

fn chip_icon_is_fixed(chip_icon: ChipIcon) -> bool {
    match chip_icon {
        ChipIcon::Halve | ChipIcon::Random | ChipIcon::Sub => true,
        _ => false,
    }
}

fn draw_break_chip(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    coords: Coords,
    enabled: bool,
) {
    let matrix = grid_matrix
        * Matrix4::trans2((coords.x as f32) + 0.5, (coords.y as f32) + 0.5);
    let color = if enabled {
        Color3::new(0.9, 0.3, 0.3)
    } else {
        Color3::new(0.5, 0.6, 0.6)
    };
    resources.shaders().chip().draw_basic(
        &matrix,
        RectSize::new(1.0, 1.0).expand(-CHIP_MARGIN),
        ChipIcon::Break as u32,
        color,
        resources.textures().chip_icons(),
    );
}

fn draw_comment_chip(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    coords: Coords,
    orient: Orientation,
    icon: ChipIcon,
) {
    let matrix = grid_matrix
        * Matrix4::trans2((coords.x as f32) + 0.5, (coords.y as f32) + 0.5)
        * orient.matrix();
    resources.shaders().chip().draw_comment(
        &matrix,
        RectSize::new(0.9, 0.9),
        icon as u32,
        Color3::PURPLE5,
        resources.textures().chip_icons(),
    );
}

fn draw_counter_chip(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    coords: Coords,
    orient: Orientation,
) {
    let orient = orient * Orientation::default().rotate_ccw();
    let oriented_size = orient * CoordsSize::new(1, 2);
    let matrix = grid_matrix
        * Matrix4::trans2(
            (coords.x as f32) + 0.5 * (oriented_size.width as f32),
            (coords.y as f32) + 0.5 * (oriented_size.height as f32),
        )
        * orient.matrix();
    let icon = ChipIcon::Counter;
    resources.shaders().chip().draw_memory(
        &matrix,
        RectSize::new(1.0, 2.0).expand(-CHIP_MARGIN),
        icon as u32,
        chip_icon_color(icon),
        resources.textures().chip_icons(),
    );
}

fn draw_discard_chip(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    coords: Coords,
    orient: Orientation,
) {
    let matrix = grid_matrix
        * Matrix4::trans2((coords.x as f32) + 0.5, (coords.y as f32) + 0.5)
        * orient.matrix();
    let icon = ChipIcon::Discard;
    resources.shaders().chip().draw_discard(
        &matrix,
        RectSize::new(1.0, 1.0).expand(-CHIP_MARGIN),
        icon as u32,
        chip_icon_color(icon),
        resources.textures().chip_icons(),
    );
}

fn draw_latest_chip(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    coords: Coords,
    orient: Orientation,
) {
    let matrix = grid_matrix
        * Matrix4::trans2((coords.x as f32) + 0.5, (coords.y as f32) + 0.5)
        * orient.matrix();
    let icon = ChipIcon::Latest;
    resources.shaders().chip().draw_memory(
        &matrix,
        RectSize::new(1.0, 1.0).expand(-CHIP_MARGIN),
        icon as u32,
        chip_icon_color(icon),
        resources.textures().chip_icons(),
    );
}

fn draw_basic_chip(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    coords: Coords,
    orient: Orientation,
    chip_size: CoordsSize,
    icon: ChipIcon,
) {
    let oriented_size = orient * chip_size;
    let orient = if chip_icon_is_fixed(icon) {
        debug_assert_eq!(chip_size, CoordsSize::new(1, 1));
        Orientation::default()
    } else {
        orient
    };
    let matrix = grid_matrix
        * Matrix4::trans2(
            (coords.x as f32) + 0.5 * (oriented_size.width as f32),
            (coords.y as f32) + 0.5 * (oriented_size.height as f32),
        )
        * orient.matrix();
    let icon_index = icon as u32;
    let icon_color = chip_icon_color(icon);
    resources.shaders().chip().draw_basic(
        &matrix,
        chip_size.as_f32().expand(-CHIP_MARGIN),
        icon_index,
        icon_color,
        resources.textures().chip_icons(),
    );
}

fn draw_chip_string(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    coords: Coords,
    chip_size: CoordsSize,
    font: Font,
    font_size: f32,
    color: &Color4,
    text_orient: Orientation,
    string: &str,
) {
    let matrix = grid_matrix
        * Matrix4::from_translation(vec3(
            (coords.x as f32) + 0.5 * (chip_size.width as f32),
            (coords.y as f32) + 0.5 * (chip_size.height as f32),
            0.101,
        ))
        * text_orient.matrix();
    let font = resources.fonts().get(font);
    font.draw_style(
        &matrix,
        font_size,
        Align::MidCenter,
        (0.0, 0.0),
        color,
        0.0,
        string,
    );
}

fn draw_port(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    port: &PortSpec,
) {
    let x = port.coords.x as f32 + 0.5;
    let y = port.coords.y as f32 + 0.5;
    let matrix = grid_matrix
        * Matrix4::trans2(x, y)
        * Matrix4::from_angle_z(port.dir.angle_from_east())
        * Matrix4::scale2(0.5, 0.3);

    let shader = resources.shaders().port();
    shader.bind();
    shader.set_mvp(&matrix);
    shader.set_port_flow_and_color(
        port.flow == PortFlow::Send,
        port.color == PortColor::Event,
    );
    shader.set_texture(resources.textures().brushed_metal());
    let width_scale = match port.max_size {
        WireSize::Zero => 0.25,
        WireSize::One => 0.4,
        WireSize::Two => 0.5,
        WireSize::Four => 0.6,
        WireSize::Eight => 0.75,
        WireSize::Sixteen => 0.9,
    };
    shader.draw(width_scale);
}

//===========================================================================//

// Margin around screen from edges of grid cells, in grid units:
const SCREEN_MARGIN: f32 = 0.3;
// Width/height of a screen chip, in grid units:
const SCREEN_CHIP_SIZE: f32 = 5.0;
// Width/height of one screen cell, in grid units:
const SCREEN_CELL_SIZE: f32 = (SCREEN_CHIP_SIZE - 2.0 * SCREEN_MARGIN) / 16.0;

fn chip_screen_cell(coords: Coords, grid_pt: Point2<f32>) -> Option<u32> {
    let cell = (((grid_pt - coords.as_f32())
        - vec2(SCREEN_MARGIN, SCREEN_MARGIN))
        / SCREEN_CELL_SIZE)
        .as_i32_floor();
    if cell.x >= 0 && cell.x < 16 && cell.y >= 0 && cell.y < 16 {
        Some((cell.x + 16 * cell.y) as u32)
    } else {
        None
    }
}

fn draw_chip_screen(
    resources: &Resources,
    grid_matrix: &Matrix4<f32>,
    coords: Coords,
    bytes: &[u8],
) {
    let font = resources.fonts().roman();
    let matrix = grid_matrix
        * Matrix4::from_translation(vec3(
            0.5 * SCREEN_CHIP_SIZE + (coords.x as f32),
            coords.y as f32,
            0.101,
        ))
        * Matrix4::from_nonuniform_scale(font.ratio().recip(), 1.0, 1.0);
    for row in 0..16 {
        font.draw_chars(
            &matrix,
            SCREEN_CELL_SIZE,
            Align::TopCenter,
            (0.0, SCREEN_MARGIN + SCREEN_CELL_SIZE * (row as f32)),
            &Color4::YELLOW5,
            0.0,
            &bytes[(16 * row)..(16 * (row + 1))],
        );
    }
}

//===========================================================================//
