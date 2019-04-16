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
use cgmath::{Matrix4, Point2};
use tachy::geom::{AsFloat, Color4, Rect, RectSize};
use tachy::gui::{Event, Resources};
use tachy::save::Puzzle;
use tachy::state::CircuitEval;

//===========================================================================//

const TRAY_INNER_MARGIN: i32 = 20;

//===========================================================================//

pub struct VerificationTray {
    rect: Rect<i32>,
    subview: Box<PuzzleVerifyView>,
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
        let tray_size = if subview_size.is_empty() {
            RectSize::new(0, 0)
        } else {
            subview_size.expand(TRAY_INNER_MARGIN)
        };
        let rect = Rect::new(window_size.width - tray_size.width,
                             window_size.height - tray_size.height,
                             tray_size.width,
                             tray_size.height);
        VerificationTray { rect, subview }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                circuit_eval: Option<&CircuitEval>) {
        if self.rect.is_empty() {
            return;
        }
        let ui = resources.shaders().ui();
        ui.draw_box2(matrix,
                     &self.rect.as_f32(),
                     &Color4::ORANGE2,
                     &Color4::CYAN2,
                     &Color4::PURPLE0.with_alpha(0.8));
        self.subview.draw(resources, matrix, circuit_eval);
    }

    pub fn on_event(&mut self, event: &Event) -> bool {
        match event {
            Event::MouseDown(mouse) if self.rect.contains_point(mouse.pt) => {
                true
            }
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                true
            }
            _ => false,
        }
    }
}

//===========================================================================//
