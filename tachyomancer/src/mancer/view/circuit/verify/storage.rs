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
use tachy::state::{CircuitEval, StorageDepotEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 300;
const VIEW_HEIGHT: i32 = 160;

const FONT_SIZE: f32 = 20.0;

//===========================================================================//

struct EvalData {
    position: u32,
    angle: u32,
    holding: u32,
    stations: Vec<u32>,
    desired: Option<u32>,
}

//===========================================================================//

pub struct StorageDepotVerifyView {
    rect: Rect<i32>,
}

impl StorageDepotVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(StorageDepotVerifyView { rect })
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
        let data = if let Some(eval) = circuit_eval {
            let eval = eval.puzzle_eval::<StorageDepotEval>();
            EvalData {
                position: eval.current_position(),
                angle: eval.current_angle(),
                holding: eval.currently_holding(),
                stations: eval.station_crates().to_vec(),
                desired: eval.desired_crate(),
            }
        } else {
            EvalData {
                position: 0,
                angle: 0,
                holding: 0,
                stations: vec![0; 8],
                desired: None,
            }
        };
        // TODO: Draw a robot arm and eight numbered positions in a circle.
        let left = self.rect.x as f32;
        let top = self.rect.y as f32;
        let font = resources.fonts().roman();
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top),
            &format!("Pos: {}", data.position),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 30.0),
            &format!("Deg: {}", data.angle),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 60.0),
            &format!("Held: {}", data.holding),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 90.0),
            &format!("{:?}", data.stations),
        );
        font.draw(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 120.0),
            &format!("Wanted: {:?}", data.desired),
        );
    }
}

//===========================================================================//
