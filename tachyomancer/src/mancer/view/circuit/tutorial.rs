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

use super::super::paragraph::Paragraph;
use crate::mancer::gui::Resources;
use crate::mancer::save::Prefs;
use cgmath::{Matrix4, Point2};
use tachy::geom::{AsFloat, Color4, Rect};

//===========================================================================//

const MARGIN: i32 = 10;
const PARAGRAPH_FONT_SIZE: f32 = 16.0;
const PARAGRAPH_LINE_HEIGHT: f32 = 19.0;
const PARAGRAPH_MAX_WIDTH: f32 = 260.0;

//===========================================================================//

pub struct TutorialBubble {
    paragraph: Paragraph,
}

impl TutorialBubble {
    pub fn new(prefs: &Prefs, format: &str) -> TutorialBubble {
        let paragraph = Paragraph::compile(
            PARAGRAPH_FONT_SIZE,
            PARAGRAPH_LINE_HEIGHT,
            PARAGRAPH_MAX_WIDTH,
            prefs,
            format,
        );
        TutorialBubble { paragraph }
    }

    pub fn width(&self) -> i32 {
        (self.paragraph.width().ceil() as i32) + 2 * MARGIN
    }

    pub fn height(&self) -> i32 {
        (self.paragraph.height().ceil() as i32) + 2 * MARGIN
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        topleft: Point2<i32>,
    ) {
        let ui = resources.shaders().ui();
        let rect =
            Rect::new(topleft.x, topleft.y, self.width(), self.height())
                .as_f32();
        ui.draw_bubble(
            matrix,
            &rect,
            &Color4::ORANGE1,
            &Color4::CYAN1,
            &Color4::PURPLE0_TRANSLUCENT,
        );
        self.paragraph.draw(
            resources,
            matrix,
            (rect.x + MARGIN as f32, rect.y + MARGIN as f32),
        );
    }
}

//===========================================================================//
