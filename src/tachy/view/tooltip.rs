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

use super::paragraph::Paragraph;
use cgmath::{Matrix4, Point2};
use tachy::geom::{Color3, Color4, Rect, RectSize};
use tachy::gui::{ClockEventData, Resources};
use tachy::save::Prefs;

//===========================================================================//

const TOOLTIP_FONT_SIZE: f32 = 20.0;
const TOOLTIP_HOVER_TIME: f64 = 0.5;
const TOOLTIP_LINE_HEIGHT: f32 = 22.0;
const TOOLTIP_INNER_MARGIN: f32 = 10.0;
const TOOLTIP_MAX_WIDTH: f32 = 380.0;
const TOOLTIP_OUTER_MARGIN: f32 = 14.0;

//===========================================================================//

pub struct Tooltip<T> {
    window_size: RectSize<i32>,
    hover: Option<(T, Point2<i32>, f64)>,
    paragraph: Option<Paragraph>,
}

impl<T: PartialEq> Tooltip<T> {
    pub fn new(window_size: RectSize<i32>) -> Tooltip<T> {
        Tooltip {
            window_size,
            hover: None,
            paragraph: None,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        if let Some((_, pt, _)) = self.hover {
            if let Some(ref paragraph) = self.paragraph {
                let width = paragraph.width() + 2.0 * TOOLTIP_INNER_MARGIN;
                let height = paragraph.height() + 2.0 * TOOLTIP_INNER_MARGIN;
                let left = (pt.x as f32) -
                    width * (pt.x as f32) / (self.window_size.width as f32);
                let top = if pt.y > self.window_size.height / 2 {
                    (pt.y as f32) - (height + TOOLTIP_OUTER_MARGIN)
                } else {
                    (pt.y as f32) + TOOLTIP_OUTER_MARGIN
                };
                let rect = Rect::new(left, top, width, height);
                let ui = resources.shaders().ui();
                ui.draw_box2(matrix,
                             &rect,
                             &Color4::ORANGE2,
                             &Color4::CYAN2,
                             &Color3::PURPLE0.with_alpha(0.9));
                paragraph.draw(resources,
                               matrix,
                               (rect.x + TOOLTIP_INNER_MARGIN,
                                rect.y + TOOLTIP_INNER_MARGIN));
            }
        }
    }

    pub fn start_hover(&mut self, tag: T, pt: Point2<i32>) {
        if let Some((ref hover_tag, ref mut hover_pt, _)) = self.hover {
            if &tag == hover_tag {
                *hover_pt = pt;
                return;
            }
        }
        self.hover = Some((tag, pt, 0.0));
        self.paragraph = None;
    }

    pub fn stop_hover(&mut self, tag: &T) {
        if let Some((ref hover_tag, _, _)) = self.hover {
            if hover_tag != tag {
                return;
            }
        }
        self.stop_hover_all();
    }

    pub fn stop_hover_all(&mut self) {
        self.hover = None;
        self.paragraph = None;
    }

    pub fn tick<F>(&mut self, tick: &ClockEventData, prefs: &Prefs, func: F)
    where
        F: FnOnce(&T) -> String,
    {
        if self.paragraph.is_some() {
            return;
        }
        if let Some((ref tag, _, ref mut time)) = self.hover {
            *time = (*time + tick.elapsed).min(TOOLTIP_HOVER_TIME);
            if *time >= TOOLTIP_HOVER_TIME {
                self.paragraph = Some(Paragraph::compile(TOOLTIP_FONT_SIZE,
                                                         TOOLTIP_LINE_HEIGHT,
                                                         TOOLTIP_MAX_WIDTH,
                                                         prefs,
                                                         &func(tag)));
            }
        }
    }
}

//===========================================================================//
