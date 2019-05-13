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
use super::tray::TraySlide;
use cgmath::{Deg, Matrix4, vec2};
use tachy::font::Align;
use tachy::geom::{AsFloat, Color4, MatrixExt, Rect, RectSize};
use tachy::gui::{Cursor, Event, Resources, Ui};
use tachy::save::{Prefs, Puzzle};
use tachy::shader::UiShader;

//===========================================================================//

const PARAGRAPH_FONT_SIZE: f32 = 18.0;
const PARAGRAPH_LINE_HEIGHT: f32 = 20.0;
const PARAGRAPH_MAX_WIDTH: f32 = 320.0;
const PARAGRAPH_MIN_HEIGHT: i32 = 150;

const TRAY_FLIP_HORZ: bool = true;
const TRAY_INNER_MARGIN: i32 = 20;
const TRAY_TAB_FONT_SIZE: f32 = 16.0;
const TRAY_TAB_HEIGHT: f32 = 50.0;
const TRAY_TAB_TEXT: &str = "SPEC";

//===========================================================================//

pub struct SpecificationTray {
    rect: Rect<i32>,
    paragraph: Paragraph,
    slide: TraySlide,
}

impl SpecificationTray {
    pub fn new(window_size: RectSize<i32>, current_puzzle: Puzzle,
               prefs: &Prefs)
               -> SpecificationTray {
        let paragraph = Paragraph::compile(PARAGRAPH_FONT_SIZE,
                                           PARAGRAPH_LINE_HEIGHT,
                                           PARAGRAPH_MAX_WIDTH,
                                           prefs,
                                           current_puzzle.instructions());
        let tray_width = (paragraph.width().ceil() as i32) +
            2 * TRAY_INNER_MARGIN;
        let tray_height = (paragraph.height().ceil() as i32)
            .max(PARAGRAPH_MIN_HEIGHT) +
            2 * TRAY_INNER_MARGIN;
        let rect = Rect::new(window_size.width - tray_width,
                             -8,
                             tray_width,
                             tray_height);
        SpecificationTray {
            rect,
            paragraph,
            slide: TraySlide::new(rect.width),
        }
    }

    fn slid_rect(&self) -> Rect<i32> {
        self.rect + vec2(self.slide.distance(), 0)
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let matrix = matrix *
            Matrix4::trans2(self.slide.distance() as f32, 0.0);
        let ui = resources.shaders().ui();
        let rect = self.rect.as_f32();
        let tab_rect =
            UiShader::tray_tab_rect(rect, TRAY_TAB_HEIGHT, TRAY_FLIP_HORZ);
        ui.draw_tray(&matrix,
                     &rect,
                     TRAY_TAB_HEIGHT,
                     TRAY_FLIP_HORZ,
                     &Color4::ORANGE2,
                     &Color4::CYAN2,
                     &Color4::PURPLE0_TRANSLUCENT);

        let tab_matrix = matrix *
            Matrix4::trans2(tab_rect.x + 0.5 * tab_rect.width,
                            tab_rect.y + 0.5 * tab_rect.height) *
            Matrix4::from_angle_z(Deg(90.0));
        let font = resources.fonts().roman();
        font.draw(&tab_matrix,
                  TRAY_TAB_FONT_SIZE,
                  Align::MidCenter,
                  (0.0, -2.0),
                  TRAY_TAB_TEXT);

        self.paragraph.draw(resources,
                            &matrix,
                            (rect.x + TRAY_INNER_MARGIN as f32,
                             rect.y + TRAY_INNER_MARGIN as f32));
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui) -> bool {
        match event {
            Event::ClockTick(tick) => {
                self.slide.on_tick(tick);
            }
            Event::MouseDown(mouse) => {
                let rel_mouse_pt = mouse.pt - vec2(self.slide.distance(), 0);
                let tab_rect = UiShader::tray_tab_rect(self.rect.as_f32(),
                                                       TRAY_TAB_HEIGHT,
                                                       TRAY_FLIP_HORZ);
                if tab_rect.contains_point(rel_mouse_pt.as_f32()) {
                    self.slide.toggle();
                    return true;
                } else if self.rect.contains_point(rel_mouse_pt) {
                    return true;
                }
            }
            Event::MouseMove(mouse) |
            Event::MouseUp(mouse) => {
                let rel_mouse_pt = mouse.pt - vec2(self.slide.distance(), 0);
                let tab_rect = UiShader::tray_tab_rect(self.rect.as_f32(),
                                                       TRAY_TAB_HEIGHT,
                                                       TRAY_FLIP_HORZ);
                if self.rect.contains_point(rel_mouse_pt) ||
                    tab_rect.contains_point(rel_mouse_pt.as_f32())
                {
                    ui.cursor().request(Cursor::default());
                }
            }
            Event::Multitouch(touch)
                if self.slid_rect().contains_point(touch.pt) => {
                return true;
            }
            Event::Scroll(scroll)
                if self.slid_rect().contains_point(scroll.pt) => {
                return true;
            }
            _ => {}
        }
        return false;
    }
}

//===========================================================================//
