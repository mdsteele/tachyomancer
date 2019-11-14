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
use crate::mancer::gui::{Event, Resources, Ui};
use crate::mancer::save::Prefs;
use cgmath::{Matrix4, Point2};
use tachy::geom::{Color3, Color4, Rect, RectSize};

//===========================================================================//

const TOOLTIP_FONT_SIZE: f32 = 20.0;
const TOOLTIP_HOVER_TIME: f64 = 0.5;
const TOOLTIP_LINE_HEIGHT: f32 = 22.0;
const TOOLTIP_INNER_MARGIN: f32 = 10.0;
const TOOLTIP_MAX_WIDTH: f32 = 380.0;
const TOOLTIP_OUTER_MARGIN: f32 = 14.0;

//===========================================================================//

pub trait TooltipSink<T> {
    fn hover_tag(&mut self, pt: Point2<i32>, ui: &mut Ui, tag: T);

    fn hover_none(&mut self, ui: &mut Ui);
}

//===========================================================================//

pub struct TooltipRef<'a, T, F> {
    tooltip: &'a mut Tooltip<T>,
    func: F,
}

impl<'a, S, T, F> TooltipSink<S> for TooltipRef<'a, T, F>
where
    T: PartialEq,
    F: Fn(S) -> T,
{
    fn hover_tag(&mut self, pt: Point2<i32>, ui: &mut Ui, tag: S) {
        self.tooltip.hover_tag(pt, ui, (self.func)(tag));
    }

    fn hover_none(&mut self, ui: &mut Ui) {
        self.tooltip.hover_none(ui);
    }
}

//===========================================================================//

pub struct Tooltip<T> {
    window_size: RectSize<i32>,
    hover: Option<(T, Point2<i32>, f64)>,
    paragraph: Option<Paragraph>,
    locked: bool,
}

impl<T: PartialEq> Tooltip<T> {
    pub fn new(window_size: RectSize<i32>) -> Tooltip<T> {
        Tooltip { window_size, hover: None, paragraph: None, locked: true }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        if let Some((_, pt, _)) = self.hover {
            if let Some(ref paragraph) = self.paragraph {
                let width = paragraph.width() + 2.0 * TOOLTIP_INNER_MARGIN;
                let height = paragraph.height() + 2.0 * TOOLTIP_INNER_MARGIN;
                let left = (pt.x as f32)
                    - width * (pt.x as f32) / (self.window_size.width as f32);
                let top = if pt.y > self.window_size.height / 2 {
                    (pt.y as f32) - (height + TOOLTIP_OUTER_MARGIN)
                } else {
                    (pt.y as f32) + TOOLTIP_OUTER_MARGIN
                };
                let rect = Rect::new(left, top, width, height);
                let ui = resources.shaders().ui();
                ui.draw_box2(
                    matrix,
                    &rect,
                    &Color4::ORANGE2,
                    &Color4::CYAN2,
                    &Color3::PURPLE0.with_alpha(0.9),
                );
                paragraph.draw(
                    resources,
                    matrix,
                    (
                        rect.x + TOOLTIP_INNER_MARGIN,
                        rect.y + TOOLTIP_INNER_MARGIN,
                    ),
                );
            }
        }
    }

    pub fn on_event<F>(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        prefs: &Prefs,
        func: F,
    ) where
        F: FnOnce(&T) -> String,
    {
        self.locked = false;
        match event {
            Event::ClockTick(tick) => {
                if self.paragraph.is_none() {
                    if let Some((ref tag, _, ref mut time)) = self.hover {
                        *time = (*time + tick.elapsed).min(TOOLTIP_HOVER_TIME);
                        if *time >= TOOLTIP_HOVER_TIME {
                            self.paragraph = Some(Paragraph::compile(
                                TOOLTIP_FONT_SIZE,
                                TOOLTIP_LINE_HEIGHT,
                                TOOLTIP_MAX_WIDTH,
                                prefs,
                                &func(tag),
                            ));
                            ui.request_redraw();
                        }
                    }
                }
            }
            Event::KeyDown(_) => self.hover_none(ui),
            Event::MouseDown(_) => self.hover_none(ui),
            Event::MouseMove(mouse) if mouse.left || mouse.right => {
                self.hover_none(ui);
            }
            Event::Multitouch(_) => self.hover_none(ui),
            Event::Scroll(_) => self.hover_none(ui),
            Event::Unfocus => self.hover_none(ui),
            _ => {}
        }
    }

    pub fn sink<A, F: Fn(A) -> T>(&mut self, func: F) -> TooltipRef<T, F> {
        TooltipRef { tooltip: self, func }
    }
}

impl<T: PartialEq> TooltipSink<T> for Tooltip<T> {
    fn hover_tag(&mut self, pt: Point2<i32>, ui: &mut Ui, tag: T) {
        if self.locked {
            return;
        }
        self.locked = true;
        if let Some((ref hover_tag, ref mut hover_pt, _)) = self.hover {
            if &tag == hover_tag {
                if *hover_pt != pt {
                    *hover_pt = pt;
                    if self.paragraph.is_some() {
                        ui.request_redraw();
                    }
                }
                return;
            }
        }
        self.hover = Some((tag, pt, 0.0));
        if self.paragraph.is_some() {
            self.paragraph = None;
            ui.request_redraw();
        }
    }

    fn hover_none(&mut self, ui: &mut Ui) {
        if self.locked {
            return;
        }
        self.locked = true;
        self.hover = None;
        if self.paragraph.is_some() {
            self.paragraph = None;
            ui.request_redraw();
        }
    }
}

//===========================================================================//
