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
use tachy::state::{CircuitEval, ReactorEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 160;
const VIEW_HEIGHT: i32 = 160;

const FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct ReactorVerifyView {
    rect: Rect<i32>,
}

impl ReactorVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(ReactorVerifyView { rect })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        let left = self.rect.x as f32;
        let top = self.rect.y as f32;
        let font = resources.fonts().roman();
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top),
            &format!("Power: {}", data.power),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 30.0),
            &format!("Target: {}", data.target),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 60.0),
            &format!("Rods: {:?}", data.rods),
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
