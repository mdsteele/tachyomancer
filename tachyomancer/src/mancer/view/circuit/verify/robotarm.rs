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
use crate::mancer::font::Align;
use crate::mancer::gui::Resources;
use cgmath::{Angle, Deg, Matrix4, Point2};
use tachy::geom::{AsFloat, MatrixExt, Rect, RectSize};
use tachy::state::{CircuitEval, RobotArmEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 220;
const VIEW_HEIGHT: i32 = 220;

//===========================================================================//

pub struct RobotArmVerifyView {
    rect: Rect<f32>,
}

impl RobotArmVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(RobotArmVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        let cx = self.rect.x + 0.5 * self.rect.width;
        let cy = self.rect.y + 0.5 * self.rect.height;
        // Stations:
        for station in 0..8 {
            let angle = Deg((station * 45) as f32);
            let station_matrix = matrix
                * Matrix4::trans2(
                    cx + 94.0 * angle.sin(),
                    cy - 94.0 * angle.cos(),
                );
            let mut rotation = Deg(0.0);
            if let Some((pos, manip)) = data.station_manipulation {
                if pos == station {
                    rotation = Deg(manip * 90.0);
                }
            }
            let wheel_tex_x =
                if data.current_command == Some(station) { 0.75 } else { 0.5 };
            resources.shaders().diagram().draw(
                &(station_matrix * Matrix4::from_angle_z(rotation)),
                Rect::new(-12.0, -12.0, 24.0, 24.0),
                Rect::new(wheel_tex_x, 0.75, 0.25, 0.25),
                resources.textures().diagram_storage(),
            );
            resources.fonts().roman().draw(
                &station_matrix,
                18.0,
                Align::MidCenter,
                (0.0, 0.0),
                &format!("{}", station),
            );
        }
        // Arm extension:
        let arm_matrix = matrix
            * Matrix4::trans2(cx, cy)
            * Matrix4::from_angle_z(data.angle);
        if data.arm_extension > 0.0 {
            resources.shaders().diagram().draw(
                &arm_matrix,
                Rect::new(
                    -12.0,
                    -36.0 - 36.0 * data.arm_extension,
                    24.0,
                    36.0 * data.arm_extension,
                ),
                Rect::new(0.0, 0.0, 0.25, 0.5),
                resources.textures().diagram_storage(),
            );
        }
        // Arm claw:
        resources.shaders().diagram().draw(
            &arm_matrix,
            Rect::new(-18.0, -72.0 - 36.0 * data.arm_extension, 36.0, 36.0),
            Rect::new(0.25, 0.0, 0.375, 0.375),
            resources.textures().diagram_storage(),
        );
        // Arm base:
        resources.shaders().diagram().draw(
            &arm_matrix,
            Rect::new(-12.0, -36.0, 24.0, 48.0),
            Rect::new(0.0, 0.5, 0.25, 0.5),
            resources.textures().diagram_storage(),
        );
    }
}

impl PuzzleVerifyView for RobotArmVerifyView {
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
            let eval = eval.puzzle_eval::<RobotArmEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    angle: Deg(eval.arm_angle() as f32),
                    arm_extension: eval.arm_extension(),
                    station_manipulation: eval.station_manipulation(),
                    current_command: eval.current_command(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    angle: Deg(0.0),
                    arm_extension: 0.0,
                    station_manipulation: None,
                    current_command: None,
                },
            );
        };
    }
}

//===========================================================================//

struct EvalData {
    angle: Deg<f32>,
    arm_extension: f32,
    station_manipulation: Option<(u32, f32)>,
    current_command: Option<u32>,
}

//===========================================================================//
