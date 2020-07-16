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
use tachy::state::{CircuitEval, EnrichmentEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 300;
const VIEW_HEIGHT: i32 = 160;

const FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct EnrichmentVerifyView {
    rect: Rect<f32>,
}

impl EnrichmentVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(EnrichmentVerifyView { rect: rect.as_f32() })
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
            &format!("{:?}", &data.tank_contents[..8]),
        );
        resources.fonts().roman().draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (self.rect.x, self.rect.y + 30.0),
            &format!("{:?}", &data.tank_contents[8..]),
        );
        resources.fonts().roman().draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (self.rect.x, self.rect.y + 60.0),
            &format!("Goal: {:?}", data.current_goal),
        );
    }
}

impl PuzzleVerifyView for EnrichmentVerifyView {
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
            let eval = eval.puzzle_eval::<EnrichmentEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    tank_contents: eval.tank_contents(),
                    current_goal: eval.current_goal(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData { tank_contents: &[0; 16], current_goal: None },
            );
        };
    }
}

//===========================================================================//

struct EvalData<'a> {
    tank_contents: &'a [u8],
    current_goal: Option<(u8, u32)>,
}

//===========================================================================//
