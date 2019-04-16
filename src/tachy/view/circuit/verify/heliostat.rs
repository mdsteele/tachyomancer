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
use cgmath::{Matrix4, Point2};
use tachy::font::Align;
use tachy::geom::{Rect, RectSize};
use tachy::gui::Resources;
use tachy::state::{CircuitEval, HeliostatEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 160;
const VIEW_HEIGHT: i32 = 160;

const FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct HeliostatVerifyView {
    rect: Rect<i32>,
}

impl HeliostatVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<PuzzleVerifyView> {
        let rect = Rect::new(right_bottom.x - VIEW_WIDTH,
                             right_bottom.y - VIEW_HEIGHT,
                             VIEW_WIDTH,
                             VIEW_HEIGHT);
        Box::new(HeliostatVerifyView { rect })
    }
}

impl PuzzleVerifyView for HeliostatVerifyView {
    fn size(&self) -> RectSize<i32> { RectSize::new(VIEW_WIDTH, VIEW_HEIGHT) }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            opt_circuit_eval: Option<&CircuitEval>) {
        let (energy, pos, opt) = if let Some(eval) = opt_circuit_eval {
            let eval = eval.puzzle_eval::<HeliostatEval>();
            (eval.current_energy(),
             eval.current_position(),
             eval.current_optimum())
        } else {
            (0, Point2::new(0, 0), Point2::new(0, 0))
        };
        // TODO: Draw a grid view of the heliostat coordinates, and a visual
        //   energy meter.
        let left = self.rect.x as f32;
        let top = self.rect.y as f32;
        let font = resources.fonts().roman();
        font.draw(matrix,
                  FONT_SIZE,
                  Align::TopLeft,
                  (left, top),
                  &format!("Energy: {}", energy));
        font.draw(matrix,
                  FONT_SIZE,
                  Align::TopLeft,
                  (left, top + 30.0),
                  &format!("Pos: ({}, {})", pos.x, pos.y));
        font.draw(matrix,
                  FONT_SIZE,
                  Align::TopLeft,
                  (left, top + 60.0),
                  &format!("Opt: ({}, {})", opt.x, opt.y));
    }
}

//===========================================================================//
