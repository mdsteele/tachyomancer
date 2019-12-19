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
use tachy::state::{CircuitEval, TurretEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 216;
const VIEW_HEIGHT: i32 = 192;

const QUADRENTS: &[(f32, f32)] =
    &[(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)];

//===========================================================================//

pub struct TurretVerifyView {
    rect: Rect<f32>,
}

impl TurretVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(TurretVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        let cx = self.rect.x + 96.0;
        let cy = self.rect.y + 96.0;
        let field_matrix = matrix * Matrix4::trans2(cx, cy);
        // FIeld:
        for &(sx, sy) in QUADRENTS.iter() {
            resources.shaders().diagram().draw(
                &(field_matrix * Matrix4::scale2(sx, sy)),
                Rect::new(0.0, 0.0, 96.0, 96.0),
                Rect::new(0.0, 0.0, 0.75, 0.75),
                resources.textures().diagram_turret(),
            );
        }
        // Sector numbers:
        for sector in 0..8 {
            let r = 83.0;
            let (sin, cos) = (Deg(45.0) * (sector as f32)).sin_cos();
            resources.fonts().roman().draw_style(
                &field_matrix,
                20.0,
                Align::MidCenter,
                (r * sin, -r * cos),
                &Color4::PURPLE3,
                0.0,
                &format!("{}", sector),
            );
        }
        // Enemies:
        for &(sector, dist, _) in data.enemies.iter() {
            let theta = Deg(-90.0) + Deg(45.0) * (sector as f32);
            let r = 22.0 + 74.0 * (dist as f32) / 256.0;
            resources.shaders().diagram().draw(
                &(field_matrix
                    * Matrix4::trans2(r * theta.cos(), r * theta.sin())
                    * Matrix4::from_angle_z(theta)),
                Rect::new(-16.0, -16.0, 32.0, 32.0),
                Rect::new(0.0, 0.75, 0.25, 0.25),
                resources.textures().diagram_turret(),
            );
        }
        // Pulse blast:
        if data.cannon_cooldown >= 0.95 {
            resources.shaders().diagram().draw(
                &(field_matrix
                    * Matrix4::from_angle_z(data.turret_angle)
                    * Matrix4::trans2(0.0, -42.0)),
                Rect::new(-16.0, -24.0, 32.0, 48.0),
                Rect::new(0.75, 0.375, 0.25, 0.375),
                resources.textures().diagram_turret(),
            );
        }
        // Cannon:
        resources.shaders().diagram().draw(
            &(field_matrix * Matrix4::from_angle_z(data.turret_angle)),
            Rect::new(-16.0, -24.0, 32.0, 48.0),
            Rect::new(0.75, 0.0, 0.25, 0.375),
            resources.textures().diagram_turret(),
        );
        // Cooldown meter:
        let margin = 5.0;
        let bar_height = 90.0;
        let missing_bar_height = data.cannon_cooldown * bar_height;
        self.draw_bar_caps(resources, matrix, 0.0, bar_height + margin * 2.0);
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(
                self.rect.right() - 20.0,
                self.rect.y + margin + missing_bar_height,
                32.0,
                bar_height - missing_bar_height,
            ),
            Rect::new(0.5, 0.75, 0.25, 0.25),
            resources.textures().diagram_turret(),
        );
        // Health meter:
        self.draw_bar_caps(resources, matrix, 112.0, 80.0);
        for row in 0..data.base_health {
            resources.shaders().diagram().draw(
                matrix,
                Rect::new(
                    self.rect.right() - 20.0,
                    self.rect.bottom() - margin - 14.0 * ((row + 1) as f32),
                    32.0,
                    32.0,
                ),
                Rect::new(0.75, 0.75, 0.25, 0.25),
                resources.textures().diagram_turret(),
            );
        }
    }

    fn draw_bar_caps(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        top: f32,
        total_height: f32,
    ) {
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(self.rect.right() - 20.0, self.rect.y + top, 32.0, 16.0),
            Rect::new(0.25, 0.75, 0.25, 0.125),
            resources.textures().diagram_turret(),
        );
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(
                self.rect.right() - 20.0,
                self.rect.y + top + total_height - 16.0,
                32.0,
                16.0,
            ),
            Rect::new(0.25, 0.875, 0.25, 0.125),
            resources.textures().diagram_turret(),
        );
    }
}

impl PuzzleVerifyView for TurretVerifyView {
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
            let eval = eval.puzzle_eval::<TurretEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    turret_angle: Deg(eval.turret_angle() as f32),
                    cannon_cooldown: eval.cannon_cooldown(),
                    enemies: eval.enemies(),
                    base_health: eval.base_health(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    turret_angle: Deg(0.0),
                    cannon_cooldown: 0.0,
                    enemies: &[],
                    base_health: 5,
                },
            );
        }
    }
}

//===========================================================================//

struct EvalData<'a> {
    turret_angle: Deg<f32>,
    cannon_cooldown: f32,           // 0.0 to 1.0
    enemies: &'a [(u32, u32, u32)], // (sector, distance, speed)
    base_health: u32,
}

//===========================================================================//
