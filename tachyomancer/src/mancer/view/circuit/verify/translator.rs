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
use tachy::geom::{Color4, Rect, RectSize};
use tachy::state::{CircuitEval, TranslatorEval};

//===========================================================================//

const VIEW_WIDTH: i32 = 300;
const VIEW_HEIGHT: i32 = 160;

const FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct TranslatorVerifyView {
    rect: Rect<i32>,
}

impl TranslatorVerifyView {
    pub fn new(right_bottom: Point2<i32>) -> Box<dyn PuzzleVerifyView> {
        let rect = Rect::new(
            right_bottom.x - VIEW_WIDTH,
            right_bottom.y - VIEW_HEIGHT,
            VIEW_WIDTH,
            VIEW_HEIGHT,
        );
        Box::new(TranslatorVerifyView { rect })
    }

    fn draw_data(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        data: &EvalData,
    ) {
        let left = self.rect.x as f32;
        let right = self.rect.right() as f32;
        let top = self.rect.y as f32;
        resources.fonts().alien().draw_chars(
            matrix,
            FONT_SIZE,
            Align::TopRight,
            (right, top),
            &Color4::WHITE,
            0.0,
            data.bytes_read,
        );
        resources.fonts().alien().draw_chars(
            matrix,
            FONT_SIZE,
            Align::TopLeft,
            (left, top + 30.0),
            &Color4::WHITE,
            0.0,
            data.translation_buffer,
        );
        if let Some(bytes) = data.pending_translation {
            resources.fonts().roman().draw_chars(
                matrix,
                FONT_SIZE,
                Align::TopLeft,
                (left, top + 60.0),
                &Color4::WHITE,
                0.0,
                bytes,
            );
        }
        resources.fonts().roman().draw_chars(
            matrix,
            FONT_SIZE,
            Align::TopRight,
            (right, top + 90.0),
            &Color4::WHITE,
            0.0,
            data.bytes_printed,
        );
    }
}

impl PuzzleVerifyView for TranslatorVerifyView {
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
            let eval = eval.puzzle_eval::<TranslatorEval>();
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    bytes_read: eval.bytes_read(),
                    bytes_printed: eval.bytes_printed(),
                    translation_buffer: eval.translation_buffer(),
                    pending_translation: eval.pending_translation(),
                },
            );
        } else {
            self.draw_data(
                resources,
                matrix,
                &EvalData {
                    bytes_read: b"",
                    bytes_printed: b"",
                    translation_buffer: b"",
                    pending_translation: None,
                },
            );
        };
    }
}

//===========================================================================//

struct EvalData<'a> {
    bytes_read: &'a [u8],
    bytes_printed: &'a [u8],
    translation_buffer: &'a [u8],
    pending_translation: Option<&'a [u8]>,
}

//===========================================================================//
