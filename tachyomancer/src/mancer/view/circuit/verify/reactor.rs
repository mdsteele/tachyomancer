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
use tachy::geom::{AsFloat, Color4, Rect, RectSize};
use tachy::state::{CircuitEval, ReactorEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 160;
const VIEW_HEIGHT: i32 = 140;

//===========================================================================//

pub struct ReactorVerifyView {
    rect: Rect<f32>,
}

impl ReactorVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(ReactorVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        // Control rods:
        for index in 0..3 {
            let mut x = self.rect.x + 95.0;
            let y = self.rect.bottom() - 114.0 + 26.0 * (index as f32);
            let extend = data.rods[index];
            for _ in 0..extend {
                resources.shaders().diagram().draw(
                    matrix,
                    Rect::new(x, y, 16.0, 32.0),
                    Rect::new(0.75, 0.5, 0.125, 0.25),
                    resources.textures().diagram_reactor(),
                );
                x += 16.0;
            }
            resources.shaders().diagram().draw(
                matrix,
                Rect::new(x, y, 16.0, 32.0),
                Rect::new(0.875, 0.5, 0.125, 0.25),
                resources.textures().diagram_reactor(),
            );
        }
        // Reactor:
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(self.rect.x, self.rect.bottom() - 128.0, 96.0, 96.0),
            Rect::new(0.0, 0.0, 0.75, 0.75),
            resources.textures().diagram_reactor(),
        );
        resources.shaders().diagram().draw(
            matrix,
            Rect::new(self.rect.x, self.rect.bottom() - 32.0, 128.0, 32.0),
            Rect::new(0.0, 0.75, 1.0, 0.25),
            resources.textures().diagram_reactor(),
        );
        // Power readings:
        resources.fonts().led().draw_style(
            matrix,
            16.0,
            Align::MidCenter,
            (self.rect.x + 37.0, self.rect.bottom() - 15.0),
            &Color4::YELLOW4,
            0.0,
            &format!("{}", data.target),
        );
        resources.fonts().led().draw_style(
            matrix,
            16.0,
            Align::MidCenter,
            (self.rect.x + 99.0, self.rect.bottom() - 15.0),
            &Color4::YELLOW4,
            0.0,
            &format!("{}", data.power),
        );
    }
}

impl PuzzleVerifyView for ReactorVerifyView {
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
            let eval = eval.puzzle_eval::<ReactorEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    power: eval.current_power(),
                    target: eval.target_power(),
                    rods: eval.rod_values(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData { power: 0, target: 0, rods: &[0, 0, 0] },
            );
        };
    }
}

//===========================================================================//

struct EvalData<'a> {
    power: u32,
    target: u32,
    rods: &'a [u32],
}

//===========================================================================//
