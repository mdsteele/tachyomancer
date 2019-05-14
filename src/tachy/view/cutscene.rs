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

use cgmath::{self, Matrix4, Point2};
use tachy::font::Align;
use tachy::geom::{AsFloat, Color4, Rect, RectSize};
use tachy::gui::{Event, Keycode, Resources, Ui};
use tachy::state::{CutsceneScript, Theater};
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const CLICKS_TO_SHOW_SKIP: i32 = 3;
const TIME_BETWEEN_CLICKS: f64 = 0.4; // seconds
const TIME_TO_HIDE_SKIP: f64 = 2.0; // seconds

const MESSAGE_FONT_SIZE: f32 = 20.0;
const MESSAGE_INNER_MARGIN_HORZ: f32 = 10.0;
const MESSAGE_INNER_MARGIN_VERT: f32 = 6.0;
const MESSAGE_OUTER_MARGIN: f32 = 24.0;

#[cfg(any(target_os = "android", target_os = "ios"))]
const CONTINUE_MESSAGE: &str = "Tap anywhere to continue";
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const CONTINUE_MESSAGE: &str = "Click or press [ENTER] to continue";

#[cfg(any(target_os = "android", target_os = "ios"))]
const SKIP_MESSAGE: &str = "Long-press anywhere to skip";
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const SKIP_MESSAGE: &str = "Press [ESC] to skip";

//===========================================================================//

pub enum CutsceneAction {
    Finished,
}

//===========================================================================//

pub struct CutsceneView {
    size: RectSize<f32>,
    theater: Theater,
    skip_clicks: i32,
    skip_click_time: f64,
}

impl CutsceneView {
    pub fn new(window_size: RectSize<i32>) -> CutsceneView {
        CutsceneView {
            size: window_size.as_f32(),
            theater: Theater::new(),
            skip_clicks: 0,
            skip_click_time: 0.0,
        }
    }

    pub fn init(&mut self, ui: &mut Ui, cutscene: &mut CutsceneScript) {
        cutscene.tick(0.0, ui, &mut self.theater);
    }

    pub fn draw(&self, resources: &Resources, cutscene: &CutsceneScript) {
        let matrix = cgmath::ortho(0.0,
                                   self.size.width,
                                   self.size.height,
                                   0.0,
                                   -1.0,
                                   1.0);
        let rect = Rect::with_size(Point2::new(0.0, 0.0), self.size);
        let color = self.theater.background_color();
        resources.shaders().solid().fill_rect(&matrix, color, rect);

        if cutscene.is_paused() {
            self.draw_message(resources,
                              &matrix,
                              MESSAGE_OUTER_MARGIN,
                              CONTINUE_MESSAGE);
        }
        if self.skip_clicks >= CLICKS_TO_SHOW_SKIP {
            self.draw_message(resources,
                              &matrix,
                              self.size.height - MESSAGE_OUTER_MARGIN,
                              SKIP_MESSAGE);
        }
    }

    fn draw_message(&self, resources: &Resources, matrix: &Matrix4<f32>,
                    y_center: f32, message: &str) {
        let ui = resources.shaders().ui();
        let font = resources.fonts().roman();
        let bubble_height = MESSAGE_FONT_SIZE.ceil() +
            2.0 * MESSAGE_INNER_MARGIN_VERT;
        let bubble_width = (MESSAGE_FONT_SIZE * font.ratio()).ceil() *
            (message.width() as f32) +
            2.0 * MESSAGE_INNER_MARGIN_HORZ;
        let bubble_rect = Rect::new(0.5 * (self.size.width - bubble_width),
                                    y_center - 0.5 * bubble_height,
                                    bubble_width,
                                    bubble_height);
        ui.draw_bubble(matrix,
                       &bubble_rect,
                       &Color4::CYAN1,
                       &Color4::ORANGE1,
                       &Color4::PURPLE0_TRANSLUCENT);
        font.draw(matrix,
                  MESSAGE_FONT_SIZE,
                  Align::MidCenter,
                  (0.5 * self.size.width, y_center),
                  message);
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui,
                    cutscene: &mut CutsceneScript)
                    -> Option<CutsceneAction> {
        match event {
            Event::ClockTick(tick) => {
                if self.skip_clicks > 0 {
                    self.skip_click_time -= tick.elapsed;
                    if self.skip_click_time <= 0.0 {
                        self.skip_clicks = 0;
                        self.skip_click_time = 0.0;
                    }
                }
                if cutscene.tick(tick.elapsed, ui, &mut self.theater) {
                    return Some(CutsceneAction::Finished);
                }
            }
            Event::KeyDown(key) if key.code == Keycode::Escape => {
                self.maybe_skip(cutscene);
            }
            Event::KeyDown(key) if key.code == Keycode::Return => {
                self.unpause(cutscene);
            }
            Event::MouseDown(mouse) => {
                if mouse.left {
                    self.unpause(cutscene);
                } else if mouse.right &&
                           cfg!(any(target_os = "android",
                                    target_os = "ios"))
                {
                    self.maybe_skip(cutscene);
                }
            }
            _ => {}
        }
        return None;
    }

    fn maybe_skip(&mut self, cutscene: &mut CutsceneScript) {
        if self.skip_clicks >= CLICKS_TO_SHOW_SKIP {
            cutscene.skip(&mut self.theater);
            self.skip_clicks = 0;
            self.skip_click_time = 0.0;
        } else {
            self.skip_clicks = CLICKS_TO_SHOW_SKIP;
            self.skip_click_time = TIME_TO_HIDE_SKIP;
        }
    }

    fn unpause(&mut self, cutscene: &mut CutsceneScript) {
        cutscene.unpause();
        if self.skip_clicks >= CLICKS_TO_SHOW_SKIP {
            self.skip_click_time = TIME_TO_HIDE_SKIP;
        } else {
            self.skip_clicks += 1;
            if self.skip_clicks >= CLICKS_TO_SHOW_SKIP {
                self.skip_click_time = TIME_TO_HIDE_SKIP;
            } else {
                self.skip_click_time = TIME_BETWEEN_CLICKS;
            }
        }
    }
}

//===========================================================================//
