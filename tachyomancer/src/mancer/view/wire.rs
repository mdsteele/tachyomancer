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

use crate::mancer::gui::Resources;
use cgmath::Matrix4;
use tachy::geom::{Color3, Color4, Coords, Direction, MatrixExt};
use tachy::save::{WireShape, WireSize};
use tachy::state::WireColor;

//===========================================================================//

const WIRE_COLOR_UNKNOWN: Color3 = Color3::new(0.65, 0.65, 0.65);

//===========================================================================//

pub struct WireModel {}

impl WireModel {
    /// The matrix should go from grid space to GL clip space.
    pub fn draw_fragment(
        resources: &Resources,
        grid_matrix: &Matrix4<f32>,
        coords: Coords,
        dir: Direction,
        shape: WireShape,
        color: WireColor,
        size: WireSize,
        hilight: &Color4,
    ) {
        let shader = resources.shaders().wire();
        let matrix = grid_matrix * obj_to_grid(coords, dir);
        let texture = resources.textures().wire();
        match (shape, dir) {
            (WireShape::Stub, _) => {
                shader.draw_stub(
                    &matrix,
                    wire_size_index(size),
                    wire_color(color),
                    hilight,
                    texture,
                );
            }
            (WireShape::Straight, Direction::East)
            | (WireShape::Straight, Direction::North) => {
                shader.draw_straight(
                    &matrix,
                    wire_size_index(size),
                    wire_color(color),
                    hilight,
                    texture,
                );
            }
            (WireShape::TurnLeft, _) => {
                shader.draw_turn(
                    &matrix,
                    wire_size_index(size),
                    wire_color(color),
                    hilight,
                    texture,
                );
            }
            (WireShape::SplitTee, _) => {
                shader.draw_tee(
                    &matrix,
                    wire_size_index(size),
                    wire_color(color),
                    hilight,
                    texture,
                );
            }
            (WireShape::Cross, Direction::East) => {
                shader.draw_cross(
                    &matrix,
                    wire_size_index(size),
                    wire_color(color),
                    hilight,
                    texture,
                );
            }
            _ => {}
        }
    }

    /// The matrix should go from grid space to GL clip space.
    pub fn draw_half_straight(
        resources: &Resources,
        grid_matrix: &Matrix4<f32>,
        coords: Coords,
        dir: Direction,
        color: WireColor,
        size: WireSize,
        hilight: &Color4,
    ) {
        let shader = resources.shaders().wire();
        let matrix = grid_matrix * obj_to_grid(coords, dir);
        let texture = resources.textures().wire();
        shader.draw_half_straight(
            &matrix,
            wire_size_index(size),
            wire_color(color),
            hilight,
            texture,
        );
    }
}

fn obj_to_grid(coords: Coords, dir: Direction) -> Matrix4<f32> {
    Matrix4::trans2((coords.x as f32) + 0.5, (coords.y as f32) + 0.5)
        * Matrix4::from_angle_z(dir.angle_from_east())
        * Matrix4::from_scale(0.5)
}

fn wire_color(color: WireColor) -> &'static Color3 {
    match color {
        WireColor::Unknown => &WIRE_COLOR_UNKNOWN,
        WireColor::Ambiguous => &Color3::RED3,
        WireColor::Behavior => &Color3::ORANGE3,
        WireColor::Event => &Color3::CYAN3,
    }
}

fn wire_size_index(size: WireSize) -> usize {
    match size {
        WireSize::Zero => 0,
        WireSize::One => 1,
        WireSize::Two => 2,
        WireSize::Four => 3,
        WireSize::Eight => 4,
    }
}

//===========================================================================//
