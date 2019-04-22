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

mod heliostat;
mod robotarm;
mod shared;
mod tutorial;

use self::shared::PuzzleVerifyView;
use super::tray::TraySlide;
use cgmath::{Deg, Matrix4, Point2, vec2};
use tachy::font::Align;
use tachy::geom::{AsFloat, Color4, MatrixExt, Rect, RectSize};
use tachy::gui::{Event, Resources};
use tachy::save::Puzzle;
use tachy::shader::UiShader;
use tachy::state::CircuitEval;

//===========================================================================//

const TRAY_FLIP_HORZ: bool = true;
const TRAY_INNER_MARGIN: i32 = 20;
const TRAY_TAB_FONT_SIZE: f32 = 16.0;
const TRAY_TAB_HEIGHT: f32 = 60.0;
const TRAY_TAB_TEXT: &str = "STATUS";

//===========================================================================//

pub struct VerificationTray {
    rect: Rect<i32>,
    subview: Box<PuzzleVerifyView>,
    slide: TraySlide,
}

impl VerificationTray {
    pub fn new(window_size: RectSize<i32>, current_puzzle: Puzzle)
               -> VerificationTray {
        let right_bottom = Point2::new(window_size.width - TRAY_INNER_MARGIN,
                                       window_size.height - TRAY_INNER_MARGIN);
        let subview = match current_puzzle {
            Puzzle::TutorialOr => {
                self::tutorial::TutorialOrVerifyView::new(right_bottom)
            }
            Puzzle::AutomateHeliostat => {
                self::heliostat::HeliostatVerifyView::new(right_bottom)
            }
            Puzzle::AutomateReactor => {
                // TODO: Make a verification view for AutomateReactor
                self::shared::NullVerifyView::new()
            }
            Puzzle::AutomateRobotArm => {
                self::robotarm::RobotArmVerifyView::new(right_bottom)
            }
            Puzzle::SandboxBehavior => self::shared::NullVerifyView::new(),
            Puzzle::SandboxEvent => self::shared::NullVerifyView::new(),
        };
        let subview_size = subview.size();
        let rect = if subview_size.is_empty() {
            Rect::new(window_size.width, window_size.height, 0, 0)
        } else {
            let size = subview_size.expand(TRAY_INNER_MARGIN);
            Rect::new(window_size.width - size.width,
                      window_size.height - size.height,
                      size.width,
                      size.height + 20)
        };
        VerificationTray {
            rect,
            subview,
            slide: TraySlide::new(rect.width),
        }
    }

    fn slid_rect(&self) -> Rect<i32> {
        self.rect + vec2(self.slide.distance(), 0)
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                circuit_eval: Option<&CircuitEval>) {
        if self.rect.is_empty() {
            return;
        }
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
                     &Color4::PURPLE0.with_alpha(0.8));

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

        self.subview.draw(resources, &matrix, circuit_eval);
    }

    pub fn on_event(&mut self, event: &Event) -> bool {
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