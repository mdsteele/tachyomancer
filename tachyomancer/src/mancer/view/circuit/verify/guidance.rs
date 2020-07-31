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
use cgmath::{Matrix4, Point2};
use tachy::geom::{AsFloat, Color3, Color4, MatrixExt, Rect, RectSize};
use tachy::state::{CircuitEval, GuidanceEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 144;
const VIEW_HEIGHT: i32 = 208;

const CELL_SIZE: f32 = 16.0;

//===========================================================================//

pub struct GuidanceVerifyView {
    rect: Rect<f32>,
}

impl GuidanceVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(GuidanceVerifyView { rect: rect.as_f32() })
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

        // Enemies:
        let enemies_matrix = matrix
            * Matrix4::trans2(
                self.rect.x - 0.5 * CELL_SIZE,
                self.rect.bottom()
                    + ((data.dist_travalled as f32) - 0.5) * CELL_SIZE,
            );
        for &(dist, x_pos) in GuidanceEval::enemies() {
            let enemy_matrix = enemies_matrix
                * Matrix4::trans2(
                    (x_pos as f32) * CELL_SIZE,
                    -(dist as f32) * CELL_SIZE,
                );
            resources.shaders().solid().fill_rect(
                &enemy_matrix,
                Color3::new(0.25, 0.0, 0.0),
                Rect::new(
                    -1.5 * CELL_SIZE,
                    -1.5 * CELL_SIZE,
                    3.0 * CELL_SIZE,
                    3.0 * CELL_SIZE,
                ),
            );
            resources.shaders().solid().fill_rect(
                &enemy_matrix,
                Color3::new(0.75, 0.0, 0.0),
                Rect::new(
                    -0.5 * CELL_SIZE,
                    -0.5 * CELL_SIZE,
                    CELL_SIZE,
                    CELL_SIZE,
                ),
            );
        }

        // Torpedo:
        let torp_matrix = matrix
            * Matrix4::trans2(
                self.rect.x + ((data.torp_x_pos as f32) - 0.5) * CELL_SIZE,
                self.rect.bottom() - 0.5 * CELL_SIZE,
            );
        resources.shaders().diagram().draw(
            &torp_matrix,
            Rect::new(
                -0.5 * CELL_SIZE,
                -0.5 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
            ),
            Rect::new(0.0, 0.0, 0.25, 0.25),
            resources.textures().diagram_lander(),
        );
    }
}

impl PuzzleVerifyView for GuidanceVerifyView {
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
            let eval = eval.puzzle_eval::<GuidanceEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    dist_travalled: eval.distance_travelled(),
                    torp_x_pos: eval.torp_x_pos(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    dist_travalled: 0,
                    torp_x_pos: GuidanceEval::init_torp_x_pos(),
                },
            );
        };
    }
}

//===========================================================================//

struct EvalData {
    dist_travalled: i32,
    torp_x_pos: u32,
}

//===========================================================================//
