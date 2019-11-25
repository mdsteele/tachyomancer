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
use cgmath::{Matrix4, Point2};
use tachy::geom::{AsFloat, Color3, Rect, RectSize};
use tachy::state::{CircuitEval, ShieldsEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 192;
const VIEW_HEIGHT: i32 = 320;

//===========================================================================//

pub struct ShieldsVerifyView {
    rect: Rect<f32>,
}

impl ShieldsVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(ShieldsVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        // Ship:
        let mid_x = self.rect.x + 0.5 * self.rect.width;
        let ship_left = mid_x - 24.0;
        let ship_top = self.rect.bottom() - 48.0;
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(ship_left, ship_top, 48.0, 48.0),
            Rect::new(0.25, 0.0, 0.375, 0.375),
            resources.textures().diagram_shields(),
        );
        // Shields:
        if data.shield_is_up {
            resources.shaders().diagram().draw(
                matrix,
                Rect::new(ship_left, ship_top, 48.0, 48.0),
                Rect::new(0.25, 0.375, 0.375, 0.375),
                resources.textures().diagram_shields(),
            );
        }
        // Enemy:
        let enemy_bottom = ship_top - (data.enemy_dist as f32);
        resources.shaders().diagram().draw_tinted(
            matrix,
            Rect::new(ship_left, enemy_bottom - 48.0, 48.0, 48.0),
            &Color3::WHITE.with_alpha(1.0 - data.enemy_explosion),
            Rect::new(0.625, 0.0, 0.375, 0.375),
            resources.textures().diagram_shields(),
        );
        // Beam:
        if data.beam_cooldown != 0 {
            resources.shaders().diagram().draw(
                matrix,
                Rect::new(
                    mid_x - 12.0,
                    enemy_bottom,
                    32.0,
                    data.enemy_dist as f32,
                ),
                Rect::new(0.25, 0.75, 0.25, 0.25),
                resources.textures().diagram_shields(),
            );
        }
        // Torpedoes:
        for &(dist, _) in data.torpedoes.iter() {
            resources.shaders().diagram().draw(
                matrix,
                Rect::new(
                    mid_x - 20.0,
                    ship_top - 16.0 - (dist as f32),
                    32.0,
                    32.0,
                ),
                Rect::new(0.0, 0.75, 0.25, 0.25),
                resources.textures().diagram_shields(),
            );
        }
        // Enemy health:
        self.draw_health_bar(
            resources,
            matrix,
            0.0,
            BarY::Top(0.0),
            data.enemy_health,
            ShieldsEval::enemy_max_health(),
        );
        // Ship health:
        self.draw_health_bar(
            resources,
            matrix,
            0.0,
            BarY::Bottom(self.rect.height),
            data.ship_health,
            ShieldsEval::ship_max_health(),
        );
        // Shield power:
        self.draw_shield_bar(
            resources,
            matrix,
            self.rect.width - 32.0,
            BarY::Bottom(self.rect.height),
            data.shield_power as f32,
            100.0,
        );
    }

    fn draw_health_bar(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        left: f32,
        bar_y: BarY,
        health: f32,
        max_health: f32,
    ) {
        let margin = 5.0;
        let chunk_height = 18.0;
        let bar_height = chunk_height * max_health;
        let total_height = 2.0 * margin + bar_height;
        let top = match bar_y {
            BarY::Top(top) => top,
            BarY::Bottom(bottom) => bottom - total_height,
        };
        self.draw_bar_caps(resources, matrix, left, top, total_height);
        let chunk_left = self.rect.x + left;
        let mut chunk_top =
            self.rect.y + top + total_height - margin - chunk_height;
        let mut remaining = health;
        while remaining > 1.0 {
            resources.shaders().diagram().draw(
                matrix,
                Rect::new(chunk_left, chunk_top, 32.0, chunk_height),
                Rect::new(0.0, 0.5, 0.25, chunk_height / 128.0),
                resources.textures().diagram_shields(),
            );
            chunk_top -= chunk_height;
            remaining -= 1.0;
        }
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(
                chunk_left,
                chunk_top + (1.0 - remaining) * chunk_height,
                32.0,
                remaining * chunk_height,
            ),
            Rect::new(
                0.0,
                0.5 + (1.0 - remaining) * (chunk_height / 128.0),
                0.25,
                remaining * (chunk_height / 128.0),
            ),
            resources.textures().diagram_shields(),
        );
    }

    fn draw_shield_bar(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        left: f32,
        bar_y: BarY,
        shields: f32,
        max_shields: f32,
    ) {
        let margin = 6.0;
        let bar_height = max_shields;
        let total_height = 2.0 * margin + bar_height;
        let top = match bar_y {
            BarY::Top(top) => top,
            BarY::Bottom(bottom) => bottom - total_height,
        };
        self.draw_bar_caps(resources, matrix, left, top, total_height);
        let missing_bar_height = bar_height * (1.0 - shields / max_shields);
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(
                self.rect.x + left,
                self.rect.y + top + margin + missing_bar_height,
                32.0,
                bar_height - missing_bar_height,
            ),
            Rect::new(0.0, 0.25, 0.25, 0.25),
            resources.textures().diagram_shields(),
        );
    }

    fn draw_bar_caps(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        left: f32,
        top: f32,
        total_height: f32,
    ) {
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(self.rect.x + left, self.rect.y + top, 32.0, 16.0),
            Rect::new(0.0, 0.0, 0.25, 0.125),
            resources.textures().diagram_shields(),
        );
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(
                self.rect.x + left,
                self.rect.y + top + total_height - 16.0,
                32.0,
                16.0,
            ),
            Rect::new(0.0, 0.125, 0.25, 0.125),
            resources.textures().diagram_shields(),
        );
    }
}

impl PuzzleVerifyView for ShieldsVerifyView {
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
            let eval = eval.puzzle_eval::<ShieldsEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    enemy_dist: eval.enemy_distance(),
                    enemy_health: eval.enemy_health(),
                    enemy_explosion: eval.enemy_explosion(),
                    ship_health: eval.ship_health(),
                    shield_power: eval.shield_power(),
                    shield_is_up: eval.shield_is_up(),
                    beam_cooldown: eval.beam_cooldown(),
                    torpedoes: eval.torpedoes(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    enemy_dist: ShieldsEval::initial_enemy_distance(),
                    enemy_health: ShieldsEval::enemy_max_health(),
                    enemy_explosion: 0.0,
                    ship_health: ShieldsEval::ship_max_health(),
                    shield_power: ShieldsEval::initial_shield_power(),
                    shield_is_up: false,
                    beam_cooldown: 0,
                    torpedoes: &[],
                },
            );
        };
    }
}

//===========================================================================//

struct EvalData<'a> {
    enemy_dist: u32,
    enemy_health: f32,
    enemy_explosion: f32,
    ship_health: f32,
    shield_power: u32,
    shield_is_up: bool,
    beam_cooldown: u32,
    torpedoes: &'a [(u32, u32)],
}

enum BarY {
    Top(f32),
    Bottom(f32),
}

//===========================================================================//
