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
use cgmath::{Matrix4, Point2};
use tachy::geom::{Rect, RectSize};
use tachy::state::{CircuitEval, TurretEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 160;
const VIEW_HEIGHT: i32 = 160;

const FONT_SIZE: f32 = 20.0;

//===========================================================================//

struct EvalData {
    turret_position: u32,
    turret_angle: u32,
    cannon_cooldown: u32,
    enemies: Vec<(u32, u32, u32)>,
    base_damage: u32,
}

//===========================================================================//

pub struct TurretVerifyView {
    rect: Rect<i32>,
}

impl TurretVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(TurretVerifyView { rect })
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
        let data = if let Some(eval) = opt_circuit_eval {
            let eval = eval.puzzle_eval::<TurretEval>();
            EvalData {
                turret_position: eval.turret_position(),
                turret_angle: eval.turret_angle(),
                cannon_cooldown: eval.cannon_cooldown(),
                enemies: eval.enemies().to_vec(),
                base_damage: eval.base_damage(),
            }
        } else {
            EvalData {
                turret_position: 0,
                turret_angle: 0,
                cannon_cooldown: 0,
                enemies: Vec::new(),
                base_damage: 0,
            }
        };
        // TODO: Draw a visual diagram of the turret
        let left = self.rect.x as f32;
        let top = self.rect.y as f32;
        let font = resources.fonts().roman();
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top),
            &format!("Position: {}", data.turret_position),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 30.0),
            &format!("Angle: {}°", data.turret_angle),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 60.0),
            &format!("Cooldown: {}", data.cannon_cooldown),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 90.0),
            &format!("Damage: {}", data.base_damage),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 120.0),
            &format!("{:?}", data.enemies),
        );
    }
}

//===========================================================================//
