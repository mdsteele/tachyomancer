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
use tachy::geom::{AsFloat, Color4, MatrixExt, Rect, RectSize};
use tachy::state::{CircuitEval, StorageDepotEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 240;
const VIEW_HEIGHT: i32 = 240;

//===========================================================================//

pub struct StorageDepotVerifyView {
    rect: Rect<f32>,
}

impl StorageDepotVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(StorageDepotVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        let cx = self.rect.x + 0.5 * self.rect.width;
        let cy = self.rect.y + 0.5 * self.rect.height + 4.0;
        // Stations:
        for station in 0..8 {
            let angle = Deg((station * 45) as f32);
            let label_matrix = matrix
                * Matrix4::trans2(
                    cx + 96.0 * angle.sin(),
                    cy - 96.0 * angle.cos(),
                );
            resources.fonts().roman().draw_style(
                &label_matrix,
                24.0,
                Align::MidCenter,
                (0.0, 0.0),
                &Color4::CYAN2,
                0.0,
                &format!("{}", station),
            );
            let station_matrix = label_matrix * Matrix4::from_angle_z(angle);
            let station_tex_y = if station == 0 { 0.375 } else { 0.0 };
            resources.shaders().diagram().draw(
                &station_matrix,
                Rect::new(-18.0, -20.0, 36.0, 36.0),
                Rect::new(0.625, station_tex_y, 0.375, 0.375),
                resources.textures().diagram_storage(),
            );
            let crate_id = data.stations[station];
            if crate_id != 0 {
                resources.shaders().diagram().draw(
                    &station_matrix,
                    Rect::new(-12.0, -12.0, 24.0, 24.0),
                    Rect::new(0.25, 0.75, 0.25, 0.25),
                    resources.textures().diagram_storage(),
                );
                resources.fonts().roman().draw_style(
                    &station_matrix,
                    16.0,
                    Align::MidCenter,
                    (0.0, 0.0),
                    &Color4::ORANGE5,
                    0.0,
                    &format!("{:02}", crate_id),
                );
            }
        }
        // Desired crate:
        if let Some(desired_crate_id) = data.desired {
            resources.fonts().led().draw_style(
                matrix,
                24.0,
                Align::TopLeft,
                (self.rect.x + 64.0, self.rect.y),
                &Color4::YELLOW4,
                0.0,
                &format!("{:02}", desired_crate_id),
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
        let claw_top = -72.0 - 36.0 * data.arm_extension;
        resources.shaders().diagram().draw(
            &arm_matrix,
            Rect::new(-18.0, claw_top, 36.0, 36.0),
            Rect::new(0.25, 0.0, 0.375, 0.375),
            resources.textures().diagram_storage(),
        );
        if data.holding != 0 {
            resources.shaders().diagram().draw(
                &arm_matrix,
                Rect::new(-12.0, claw_top, 24.0, 24.0),
                Rect::new(0.25, 0.75, 0.25, 0.25),
                resources.textures().diagram_storage(),
            );
            resources.fonts().roman().draw_style(
                &arm_matrix,
                16.0,
                Align::MidCenter,
                (0.0, claw_top + 12.0),
                &Color4::ORANGE5,
                0.0,
                &format!("{:02}", data.holding),
            );
        }
        // Arm base:
        resources.shaders().diagram().draw(
            &arm_matrix,
            Rect::new(-12.0, -36.0, 24.0, 48.0),
            Rect::new(0.0, 0.5, 0.25, 0.5),
            resources.textures().diagram_storage(),
        );
    }
}

impl PuzzleVerifyView for StorageDepotVerifyView {
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
            let eval = eval.puzzle_eval::<StorageDepotEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    angle: Deg(eval.arm_angle() as f32),
                    arm_extension: eval.arm_extension(),
                    holding: eval.currently_holding(),
                    stations: eval.station_crates(),
                    desired: eval.desired_crate(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    angle: Deg(0.0),
                    arm_extension: 0.0,
                    holding: 0,
                    stations: &[0; 8],
                    desired: None,
                },
            );
        }
    }
}

//===========================================================================//

struct EvalData<'a> {
    angle: Deg<f32>,
    arm_extension: f32,
    holding: u32,
    stations: &'a [u32],
    desired: Option<u32>,
}

//===========================================================================//
