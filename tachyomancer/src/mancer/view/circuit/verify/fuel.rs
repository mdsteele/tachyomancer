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
use tachy::geom::{AsFloat, Rect, RectSize};
use tachy::state::{CircuitEval, FuelEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 300;
const VIEW_HEIGHT: i32 = 160;

const FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct FuelVerifyView {
    rect: Rect<f32>,
}

impl FuelVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(FuelVerifyView { rect: rect.as_f32() })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        resources.fonts().roman().draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (self.rect.x, self.rect.y),
            &format!("Tanks: {:?}", data.tank_amounts),
        );
        resources.fonts().roman().draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (self.rect.x, self.rect.y + 30.0),
            &format!("Finished: {}", data.num_batches_finished),
        );
    }
}

impl PuzzleVerifyView for FuelVerifyView {
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
            let eval = eval.puzzle_eval::<FuelEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    tank_amounts: eval.tank_amounts(),
                    num_batches_finished: eval.num_batches_finished(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData { tank_amounts: [0, 0], num_batches_finished: 0 },
            );
        };
    }
}

//===========================================================================//

struct EvalData {
    tank_amounts: [u32; 2],
    num_batches_finished: u32,
}

//===========================================================================//
