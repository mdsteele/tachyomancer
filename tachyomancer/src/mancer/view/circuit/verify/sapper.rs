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
use crate::mancer::gui::Resources;
use cgmath::{Deg, Matrix4, Point2, Vector2};
use tachy::geom::{AsFloat, AsInt, Color3, MatrixExt, Rect, RectSize};
use tachy::state::{CircuitEval, SapperEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 216;
const VIEW_HEIGHT: i32 = 192;

const CELL_SIZE: f32 = 16.0;
const RADAR_RANGE: i32 = 4;

//===========================================================================//

pub struct SapperVerifyView {
    rect: Rect<f32>,
}

impl SapperVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(SapperVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        let center = Point2::new(
            self.rect.x + 0.5 * self.rect.width,
            self.rect.y + 0.5 * self.rect.height,
        );
        let origin = data.sapper_position.as_i32_round();
        let shift = (origin.as_f32() - data.sapper_position) * CELL_SIZE;
        for dy in -RADAR_RANGE..=RADAR_RANGE {
            for dx in -RADAR_RANGE..=RADAR_RANGE {
                let delta = Vector2::new(dx, dy);
                let cell = SapperEval::maze_cell(origin + delta);
                if cell > 0 {
                    if !data.sections_armed[(cell - 1) as usize] {
                        // Mine:
                        let pt = center + delta.as_f32() * CELL_SIZE + shift;
                        resources.shaders().solid().fill_rect(
                            matrix,
                            Color3::ORANGE2,
                            Rect::new(
                                pt.x - 0.5 * CELL_SIZE,
                                pt.y - 0.5 * CELL_SIZE,
                                CELL_SIZE,
                                CELL_SIZE,
                            ),
                        );
                    }
                } else if cell < 0 {
                    if !data.sections_armed[(-cell - 1) as usize] {
                        // Controller:
                        let pt = center + delta.as_f32() * CELL_SIZE + shift;
                        resources.shaders().solid().fill_rect(
                            matrix,
                            Color3::YELLOW5,
                            Rect::new(
                                pt.x - 0.5 * CELL_SIZE,
                                pt.y - 0.5 * CELL_SIZE,
                                CELL_SIZE,
                                CELL_SIZE,
                            ),
                        );
                    }
                }
            }
        }
        // Sapper:
        let sapper_matrix = matrix
            * Matrix4::trans2(center.x, center.y)
            * Matrix4::from_angle_z(data.sapper_angle + Deg(90.0));
        resources.shaders().diagram().draw(
            &sapper_matrix,
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

impl PuzzleVerifyView for SapperVerifyView {
    fn size(&self) -> RectSize<i32> {
        RectSize::new(VIEW_WIDTH, VIEW_HEIGHT)
    }

    fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        opt_circuit_eval: Option<&CircuitEval>,
    ) {
        if let Some(eval) = opt_circuit_eval {
            let eval = eval.puzzle_eval::<SapperEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    sapper_position: eval.sapper_position(),
                    sapper_angle: eval.sapper_angle(),
                    sections_armed: eval.sections_armed(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    sapper_position: SapperEval::initial_sapper_position(),
                    sapper_angle: SapperEval::initial_sapper_angle(),
                    sections_armed: [false; SapperEval::NUM_SECTIONS],
                },
            );
        }
    }
}

//===========================================================================//

struct EvalData {
    sapper_position: Point2<f32>,
    sapper_angle: Deg<f32>,
    sections_armed: [bool; SapperEval::NUM_SECTIONS],
}

//===========================================================================//
