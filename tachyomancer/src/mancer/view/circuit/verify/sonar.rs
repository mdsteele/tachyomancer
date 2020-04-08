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

use super::shared::PuzzleVerifyView;
use crate::mancer::gl::Stencil;
use crate::mancer::gui::Resources;
use cgmath::{Deg, Matrix4, Point2};
use tachy::geom::{AsFloat, Color3, Color4, MatrixExt, Rect, RectSize};
use tachy::state::{CircuitEval, SonarEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 200;
const VIEW_HEIGHT: i32 = 300;

const PIXELS_PER_HORZ_STEP: f32 = 8.0;
const WALL_STRIDE: f32 = 30.0;

//===========================================================================//

pub struct SonarVerifyView {
    rect: Rect<f32>,
}

impl SonarVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(SonarVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        // Define clipping area:
        let stencil = Stencil::new();
        resources.shaders().solid().tint_rect(
            &matrix,
            Color4::TRANSPARENT,
            self.rect,
        );
        stencil.enable_clipping();

        // Canyon:
        let center_matrix = matrix
            * Matrix4::trans2(
                self.rect.x + 0.5 * self.rect.width,
                self.rect.bottom() - 32.0,
            );
        let canyon_matrix = center_matrix
            * Matrix4::trans2(0.0, WALL_STRIDE * data.dist_travalled);
        let start_index = (data.dist_travalled - 1.0).max(0.0) as usize;
        let max_walls =
            (((VIEW_HEIGHT as f32) / WALL_STRIDE).ceil() as usize) + 2;
        // Port-side walls:
        let walls = SonarEval::port_walls();
        let walls = &walls[start_index.min(walls.len())..];
        for (index, &horz) in walls.iter().take(max_walls).enumerate() {
            let left = -0.5 * self.rect.width;
            let right = -16.0 + PIXELS_PER_HORZ_STEP * (horz as f32);
            let top = -16.0 - WALL_STRIDE * ((start_index + index) as f32);
            resources.shaders().solid().fill_rect(
                &canyon_matrix,
                Color3::ORANGE1,
                Rect::new(left, top, right - left, 32.0),
            );
        }
        for (index, &horz) in walls.iter().take(max_walls).enumerate() {
            let left = -32.0 + PIXELS_PER_HORZ_STEP * (horz as f32);
            let top = -16.0 - WALL_STRIDE * ((start_index + index) as f32);
            resources.shaders().diagram().draw(
                &canyon_matrix,
                Rect::new(left, top, 32.0, 32.0),
                Rect::new(0.75, 0.25, 0.25, 0.25),
                resources.textures().diagram_lander(),
            );
        }
        // Starboard-side walls:
        let walls = SonarEval::starboard_walls();
        let walls = &walls[start_index.min(walls.len())..];
        for (index, &horz) in walls.iter().take(max_walls).enumerate() {
            let left = 16.0 + PIXELS_PER_HORZ_STEP * (horz as f32);
            let right = 0.5 * self.rect.width;
            let top = -16.0 - WALL_STRIDE * ((start_index + index) as f32);
            resources.shaders().solid().fill_rect(
                &canyon_matrix,
                Color3::ORANGE1,
                Rect::new(left, top, right - left, 32.0),
            );
        }
        for (index, &horz) in walls.iter().take(max_walls).enumerate() {
            let left = PIXELS_PER_HORZ_STEP * (horz as f32);
            let top = -16.0 - WALL_STRIDE * ((start_index + index) as f32);
            resources.shaders().diagram().draw(
                &canyon_matrix,
                Rect::new(left, top, 32.0, 32.0),
                Rect::new(1.0, 0.25, -0.25, 0.25),
                resources.textures().diagram_lander(),
            );
        }

        // AUV:
        let auv_angle: Deg<f32> = if data.heading < 0 {
            Deg(-20.0)
        } else if data.heading > 0 {
            Deg(20.0)
        } else {
            Deg(0.0)
        };
        let auv_matrix = center_matrix
            * Matrix4::trans2(
                PIXELS_PER_HORZ_STEP * (data.horz_position as f32),
                0.0,
            )
            * Matrix4::from_angle_z(auv_angle);
        resources.shaders().diagram().draw(
            &auv_matrix,
            Rect::new(-16.0, -16.0, 32.0, 32.0),
            Rect::new(0.75, 0.0, 0.25, 0.25),
            resources.textures().diagram_lander(),
        );
    }
}

impl PuzzleVerifyView for SonarVerifyView {
    fn size(&self) -> RectSize<i32> {
        RectSize::new(VIEW_WIDTH, VIEW_HEIGHT)
    }

    fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        circuit_eval: Option<&CircuitEval>,
    ) {
        if let Some(eval) = circuit_eval {
            let eval = eval.puzzle_eval::<SonarEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    heading: eval.heading(),
                    horz_position: eval.horz_position(),
                    dist_travalled: eval.distance_travelled(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    heading: 0,
                    horz_position: 0,
                    dist_travalled: 0.0,
                },
            );
        };
    }
}

//===========================================================================//

struct EvalData {
    heading: i32,
    horz_position: i32,
    dist_travalled: f32,
    // TODO: sonar pings
}

//===========================================================================//
