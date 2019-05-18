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
use cgmath::{self, Matrix4, Point2};
use std::collections::BTreeMap;
use tachy::font::Align;
use tachy::geom::{AsFloat, Color3, Color4, Rect, RectSize};
use tachy::gui::{AudioQueue, Event, Keycode, Resources, Sound, Ui};
use tachy::save::Prefs;
use tachy::state::{CutsceneScript, Portrait, Theater};
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const CLICKS_TO_SHOW_SKIP: i32 = 3;
const TIME_BETWEEN_CLICKS: f64 = 0.4; // seconds
const TIME_TO_HIDE_SKIP: f64 = 2.0; // seconds

const TALK_FONT_SIZE: f32 = 20.0;
const TALK_INNER_MARGIN: i32 = 12;
const TALK_LINE_HEIGHT: f32 = 22.0;
const TALK_MAX_PARAGRAPH_WIDTH: f32 = 460.0;
const TALK_PORTRAIT_HEIGHT: i32 = 85;
const TALK_PORTRAIT_WIDTH: i32 = 68;

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
    skip_clicks: i32,
    skip_click_time: f64,
    bg_color: Color3,
    talk_bubbles: BTreeMap<i32, TalkBubble>,
    next_talk_bubble_tag: i32,
}

impl CutsceneView {
    pub fn new(window_size: RectSize<i32>) -> CutsceneView {
        CutsceneView {
            size: window_size.as_f32(),
            skip_clicks: 0,
            skip_click_time: 0.0,
            bg_color: Color3::BLACK,
            talk_bubbles: BTreeMap::new(),
            next_talk_bubble_tag: 0,
        }
    }

    pub fn init(&mut self, ui: &mut Ui,
                (cutscene, prefs): (&mut CutsceneScript, &Prefs)) {
        cutscene.tick(0.0, &mut TheaterImpl::new(self, ui.audio(), prefs));
    }

    pub fn draw(&self, resources: &Resources, cutscene: &CutsceneScript) {
        let matrix = cgmath::ortho(0.0,
                                   self.size.width,
                                   self.size.height,
                                   0.0,
                                   -1.0,
                                   1.0);
        let rect = Rect::with_size(Point2::new(0.0, 0.0), self.size);
        resources.shaders().solid().fill_rect(&matrix, self.bg_color, rect);

        for bubble in self.talk_bubbles.values() {
            bubble.draw(resources, &matrix);
        }

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
                    (cutscene, prefs): (&mut CutsceneScript, &Prefs))
                    -> Option<CutsceneAction> {
        match event {
            Event::ClockTick(tick) => {
                for bubble in self.talk_bubbles.values_mut() {
                    bubble.tick(tick.elapsed);
                }
                if self.skip_clicks > 0 {
                    self.skip_click_time -= tick.elapsed;
                    if self.skip_click_time <= 0.0 {
                        self.skip_clicks = 0;
                        self.skip_click_time = 0.0;
                    }
                }
                if cutscene.tick(tick.elapsed,
                                 &mut TheaterImpl::new(self,
                                                       ui.audio(),
                                                       prefs))
                {
                    return Some(CutsceneAction::Finished);
                }
            }
            Event::KeyDown(key) if key.code == Keycode::Escape => {
                self.maybe_skip(ui, prefs, cutscene);
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
                    self.maybe_skip(ui, prefs, cutscene);
                }
            }
            _ => {}
        }
        return None;
    }

    fn maybe_skip(&mut self, ui: &mut Ui, prefs: &Prefs,
                  cutscene: &mut CutsceneScript) {
        if self.skip_clicks >= CLICKS_TO_SHOW_SKIP {
            cutscene.skip(&mut TheaterImpl::new(self, ui.audio(), prefs));
            self.skip_clicks = 0;
            self.skip_click_time = 0.0;
        } else {
            self.skip_clicks = CLICKS_TO_SHOW_SKIP;
            self.skip_click_time = TIME_TO_HIDE_SKIP;
        }
    }

    fn unpause(&mut self, cutscene: &mut CutsceneScript) {
        cutscene.unpause();
        for bubble in self.talk_bubbles.values_mut() {
            bubble.skip_to_end();
        }
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

struct TalkBubble {
    rect: Rect<i32>,
    portrait: Portrait,
    paragraph: Paragraph,
    millis: f64,
}

impl TalkBubble {
    fn new(window_size: RectSize<f32>, prefs: &Prefs, portrait: Portrait,
           (x_pos, y_pos): (i32, i32), format: &str)
           -> TalkBubble {
        let paragraph = Paragraph::compile(TALK_FONT_SIZE,
                                           TALK_LINE_HEIGHT,
                                           TALK_MAX_PARAGRAPH_WIDTH,
                                           prefs,
                                           format);
        let horz = 0.5 + (x_pos as f32) / 200.0;
        let vert = 0.5 + (y_pos as f32) / 200.0;
        let width = TALK_PORTRAIT_WIDTH + 3 * TALK_INNER_MARGIN +
            (paragraph.width().ceil() as i32);
        let height = TALK_PORTRAIT_HEIGHT
            .max(paragraph.height().ceil() as i32) +
            2 * TALK_INNER_MARGIN;
        let rect =
            Rect::new((horz * (window_size.width - (width as f32)))
                          .round() as i32,
                      (vert * (window_size.height - (height as f32)))
                          .round() as i32,
                      width,
                      height);
        TalkBubble {
            rect,
            portrait,
            paragraph,
            millis: 0.0,
        }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Draw bubble:
        let rect = self.rect.as_f32();
        let color = Color3::new(0.1, 0.5, 0.1);
        resources.shaders().solid().fill_rect(matrix, color, rect);

        // Draw portrait:
        let portrait_left_top = Point2::new(self.rect.x + TALK_INNER_MARGIN,
                                            self.rect.y + TALK_INNER_MARGIN);
        resources.textures().portraits().bind();
        resources
            .shaders()
            .portrait()
            .draw(matrix, self.portrait as u32, portrait_left_top.as_f32());

        // Draw paragraph:
        let left = (self.rect.x + TALK_PORTRAIT_WIDTH +
                        2 * TALK_INNER_MARGIN) as f32;
        let top = (self.rect.y + TALK_INNER_MARGIN) as f32;
        let millis = self.millis as usize;
        self.paragraph.draw_partial(resources, matrix, (left, top), millis);
    }

    fn tick(&mut self, elapsed: f64) { self.millis += elapsed * 1000.0; }

    fn skip_to_end(&mut self) {
        self.millis = self.paragraph.total_millis() as f64;
    }

    fn is_done(&self) -> bool {
        (self.millis as usize) >= self.paragraph.total_millis()
    }
}

//===========================================================================//

struct TheaterImpl<'a> {
    view: &'a mut CutsceneView,
    audio: &'a mut AudioQueue,
    prefs: &'a Prefs,
}

impl<'a> TheaterImpl<'a> {
    fn new(view: &'a mut CutsceneView, audio: &'a mut AudioQueue,
           prefs: &'a Prefs)
           -> TheaterImpl<'a> {
        TheaterImpl { view, audio, prefs }
    }
}

impl<'a> Theater for TheaterImpl<'a> {
    fn add_talk(&mut self, portrait: Portrait, pos: (i32, i32), format: &str)
                -> i32 {
        let tag = self.view.next_talk_bubble_tag;
        self.view.next_talk_bubble_tag += 1;
        let bubble =
            TalkBubble::new(self.view.size, self.prefs, portrait, pos, format);
        debug_assert!(!self.view.talk_bubbles.contains_key(&tag));
        self.view.talk_bubbles.insert(tag, bubble);
        tag
    }

    fn talk_is_done(&self, tag: i32) -> bool {
        debug_assert!(self.view.talk_bubbles.contains_key(&tag));
        self.view
            .talk_bubbles
            .get(&tag)
            .map(TalkBubble::is_done)
            .unwrap_or(true)
    }

    fn remove_talk(&mut self, tag: i32) {
        debug_assert!(self.view.talk_bubbles.contains_key(&tag));
        self.view.talk_bubbles.remove(&tag);
    }

    fn play_sound(&mut self, sound: Sound) { self.audio.play_sound(sound); }

    fn set_background_color(&mut self, color: Color3) {
        self.view.bg_color = color;
    }
}

//===========================================================================//
