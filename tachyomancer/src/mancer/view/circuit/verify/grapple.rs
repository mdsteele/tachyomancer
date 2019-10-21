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
use tachy::state::{CircuitEval, GrappleEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 160;
const VIEW_HEIGHT: i32 = 160;

const FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct GrappleVerifyView {
    rect: Rect<i32>,
}

impl GrappleVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(GrappleVerifyView { rect })
    }
}

impl PuzzleVerifyView for GrappleVerifyView {
    fn size(&self) -> RectSize<i32> {
        RectSize::new(VIEW_WIDTH, VIEW_HEIGHT)
    }

    fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        opt_circuit_eval: Option<&CircuitEval>,
    ) {
        let (port_charge, stbd_charge, num_coils_fired) =
            if let Some(eval) = opt_circuit_eval {
                let eval = eval.puzzle_eval::<GrappleEval>();
                (
                    eval.current_port_charge(),
                    eval.current_stbd_charge(),
                    eval.num_coils_fired(),
                )
            } else {
                (0, 0, 0)
            };
        let left = self.rect.x as f32;
        let top = self.rect.y as f32;
        let font = resources.fonts().roman();
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top),
            &format!("Port charge: {}", port_charge),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 30.0),
            &format!("Stbd charge: {}", stbd_charge),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 60.0),
            &format!("Coils fired: {}", num_coils_fired),
        );
    }
}

//===========================================================================//
