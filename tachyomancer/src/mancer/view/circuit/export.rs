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

use super::super::chip::ChipModel;
use super::super::wire::WireModel;
use crate::mancer::font::Align;
use crate::mancer::gl::{Depth, FrameBufferMultisample};
use crate::mancer::gui::Resources;
use cgmath::{Deg, Matrix4, Point3, Vector3};
use tachy::geom::{AsFloat, Color3, Color4, MatrixExt, Rect, RectSize};
use tachy::save::Puzzle;
use tachy::state::{EditGrid, WireColor};

//===========================================================================//

const EXPORT_WIDTH: usize = 640;
const EXPORT_HEIGHT: usize = 480;

//===========================================================================//

pub fn export_circuit_image(
    resources: &Resources,
    grid: &EditGrid,
    score: u32,
) -> (RectSize<usize>, Vec<u8>) {
    let fbo = FrameBufferMultisample::new(EXPORT_WIDTH, EXPORT_HEIGHT, true);
    fbo.bind();
    draw_background(resources);
    draw_circuit(resources, grid);
    draw_title(resources, grid.puzzle(), grid.bounds().area(), score);
    fbo.unbind(resources.window_size());
    let data = resources.shaders().frame().read_rgb_data(&fbo);
    (RectSize::new(EXPORT_WIDTH, EXPORT_HEIGHT), data)
}

fn draw_background(resources: &Resources) {
    let texel_rect =
        Rect::new(0.0, 0.0, EXPORT_WIDTH as f32, EXPORT_HEIGHT as f32);
    resources.shaders().diagram().draw(
        &cgmath::ortho(0.0, 1.0, 1.0, 0.0, -1.0, 1.0),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        texel_rect / 512.0,
        resources.textures().diagram_background(),
    );
}

fn draw_circuit(resources: &Resources, grid: &EditGrid) {
    let aspect = (EXPORT_WIDTH as f32) / (EXPORT_HEIGHT as f32);
    let p_matrix = cgmath::perspective(Deg(45.0), aspect, 0.1, 1000.0);

    // TODO: do a better job of centering board within the exported image rect
    let bounds = grid.bounds().as_f32();
    let v_matrix = {
        let center = Point3::new(0.0, 0.0, 0.0);
        let maximum = bounds.width.max(bounds.height);
        let scale = 0.9 * maximum;
        let eye = Point3::new(scale, scale, scale);
        let up = Vector3::<f32>::unit_z();
        Matrix4::<f32>::look_at(eye, center, up)
    };

    let m_matrix = Matrix4::scale2(-1.0, 1.0)
        * Matrix4::trans2(
            -(bounds.x + 0.5 * bounds.width),
            -(bounds.y + 0.5 * bounds.height),
        );

    let grid_matrix = p_matrix * v_matrix * m_matrix;

    let depth = Depth::enable_with_face_culling(false);
    let board_rect = bounds.expand(0.25);
    resources.shaders().solid().fill_rect(
        &grid_matrix,
        Color3::PURPLE1,
        board_rect,
    );
    // TODO: chip bodies are not showing up
    for (coords, ctype, orient) in grid.chips() {
        ChipModel::draw_chip(
            resources,
            &grid_matrix,
            coords,
            ctype,
            orient,
            None,
        );
    }
    for interface in grid.interfaces() {
        let coords = interface.top_left(grid.bounds());
        ChipModel::draw_interface(resources, &grid_matrix, coords, interface);
    }
    for (coords, dir, shape, size, color, error) in grid.wire_fragments() {
        let color = if error { WireColor::Ambiguous } else { color };
        WireModel::draw_fragment(
            resources,
            &grid_matrix,
            coords,
            dir,
            shape,
            color,
            size,
            &Color4::TRANSPARENT,
        );
    }
    depth.disable();
}

fn draw_title(resources: &Resources, puzzle: Puzzle, area: i32, score: u32) {
    // TODO: make exported image title look nicer
    let width = EXPORT_WIDTH as f32;
    let height = EXPORT_HEIGHT as f32;
    let margin = 8.0;
    let matrix = cgmath::ortho(0.0, width, height, 0.0, -1.0, 1.0);
    resources.fonts().roman().draw(
        &matrix,
        40.0,
        Align::TopLeft,
        (margin, margin),
        "TACHYOMANCER",
    );
    resources.fonts().roman().draw(
        &matrix,
        30.0,
        Align::BottomLeft,
        (margin, height - margin),
        puzzle.title(),
    );
    resources.fonts().roman().draw(
        &matrix,
        30.0,
        Align::BottomRight,
        (width - margin, height - margin),
        &format!(
            "{}: {}  Area: {}",
            puzzle.score_units().label(),
            score,
            area
        ),
    );
}

//===========================================================================//
