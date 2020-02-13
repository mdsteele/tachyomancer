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
use cgmath::{Deg, Matrix4, Point2};
use tachy::geom::{AsFloat, MatrixExt, Rect, RectSize};
use tachy::state::{CircuitEval, LanderEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 128;
const VIEW_HEIGHT: i32 = 300;

//===========================================================================//

pub struct LanderVerifyView {
    rect: Rect<f32>,
}

impl LanderVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(LanderVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        // Ground:
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(self.rect.x, self.rect.bottom() - 32.0, 128.0, 32.0),
            Rect::new(0.0, 0.75, 1.0, 0.25),
            resources.textures().diagram_lander(),
        );
        // Thrust:
        let lander_matrix = matrix
            * Matrix4::trans2(
                self.rect.x + 0.5 * self.rect.width,
                self.rect.bottom() - 32.0 - data.altitude,
            )
            * Matrix4::from_angle_z(data.angle);
        if data.port_thrust > 0.0 {
            resources.shaders().diagram().draw(
                &lander_matrix,
                Rect::new(-16.0, 16.0, 16.0, 16.0 * data.port_thrust),
                Rect::new(0.0, 0.25, 0.125, 0.125),
                resources.textures().diagram_lander(),
            );
        }
        if data.stbd_thrust > 0.0 {
            resources.shaders().diagram().draw(
                &lander_matrix,
                Rect::new(16.0, 16.0, -16.0, 16.0 * data.stbd_thrust),
                Rect::new(0.0, 0.25, 0.125, 0.125),
                resources.textures().diagram_lander(),
            );
        }
        // Lander:
        resources.shaders().diagram().draw(
            &lander_matrix,
            Rect::new(-16.0, -16.0, 32.0, 32.0),
            Rect::new(0.0, 0.0, 0.25, 0.25),
            resources.textures().diagram_lander(),
        );
        // Fuel gauge:
        let margin = 5.0;
        let bar_height = 90.0;
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(self.rect.right() - 20.0, self.rect.y, 32.0, 16.0),
            Rect::new(0.0, 0.5, 0.25, 0.125),
            resources.textures().diagram_lander(),
        );
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(
                self.rect.right() - 20.0,
                self.rect.y + bar_height + margin * 2.0 - 16.0,
                32.0,
                16.0,
            ),
            Rect::new(0.0, 0.625, 0.25, 0.125),
            resources.textures().diagram_lander(),
        );
        let missing_bar_height = bar_height * (1.0 - data.fuel);
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(
                self.rect.right() - 12.0,
                self.rect.y + margin + missing_bar_height,
                16.0,
                bar_height - missing_bar_height,
            ),
            Rect::new(0.125, 0.25, 0.125, 0.25),
            resources.textures().diagram_lander(),
        );
    }
}

impl PuzzleVerifyView for LanderVerifyView {
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
            let eval = eval.puzzle_eval::<LanderEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    altitude: eval.lander_altitude() as f32,
                    angle: Deg(eval.lander_angle_from_vertical() as f32),
                    fuel: eval.fuel(),
                    port_thrust: eval.port_thrust(),
                    stbd_thrust: eval.stbd_thrust(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    altitude: LanderEval::INITIAL_ALTITUDE as f32,
                    angle: Deg(0.0),
                    fuel: 1.0,
                    port_thrust: 0.0,
                    stbd_thrust: 0.0,
                },
            );
        }
    }
}

//===========================================================================//

struct EvalData {
    altitude: f32,
    angle: Deg<f32>,
    fuel: f32,
    port_thrust: f32,
    stbd_thrust: f32,
}

//===========================================================================//
