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

mod beacon;
mod grapple;
mod heliostat;
mod lander;
mod mining;
mod robotarm;
mod sensors;
mod shared;

use self::shared::{FabricationVerifyView, PuzzleVerifyView};
use super::tray::TraySlide;
use crate::mancer::font::Align;
use crate::mancer::gui::{Cursor, Event, Resources, Ui};
use crate::mancer::shader::UiShader;
use cgmath::{vec2, Deg, Matrix4, Point2};
use tachy::geom::{AsFloat, Color4, MatrixExt, Rect, RectSize};
use tachy::save::Puzzle;
use tachy::state::{
    CircuitEval, FabricateEggTimerEval, FabricateHalveEval, FabricateIncEval,
    FabricateMulEval, FabricateStopwatchEval, FabricateXorEval,
    TutorialAddEval, TutorialDemuxEval, TutorialMuxEval, TutorialOrEval,
    TutorialSumEval,
};

//===========================================================================//

const TRAY_FLIP_HORZ: bool = true;
const TRAY_INNER_MARGIN: i32 = 20;
const TRAY_TAB_FONT_SIZE: f32 = 16.0;
const TRAY_TAB_HEIGHT: f32 = 60.0;
const TRAY_TAB_TEXT: &str = "STATUS";

//===========================================================================//

pub struct VerificationTray {
    rect: Rect<i32>,
    subview: Box<dyn PuzzleVerifyView>,
    slide: TraySlide,
}

impl VerificationTray {
    pub fn new(
        window_size: RectSize<i32>,
        current_puzzle: Puzzle,
    ) -> VerificationTray {
        let right_bottom = Point2::new(
            window_size.width - TRAY_INNER_MARGIN,
            window_size.height - TRAY_INNER_MARGIN,
        );
        let subview = match current_puzzle {
            Puzzle::AutomateBeacon => {
                self::beacon::BeaconVerifyView::new(right_bottom)
            }
            Puzzle::AutomateGrapple => {
                self::grapple::GrappleVerifyView::new(right_bottom)
            }
            Puzzle::AutomateHeliostat => {
                self::heliostat::HeliostatVerifyView::new(right_bottom)
            }
            Puzzle::AutomateMiningRobot => {
                self::mining::MiningRobotVerifyView::new(right_bottom)
            }
            Puzzle::AutomateReactor => {
                // TODO: Make a verification view for AutomateReactor
                self::shared::NullVerifyView::new()
            }
            Puzzle::AutomateRobotArm => {
                self::robotarm::RobotArmVerifyView::new(right_bottom)
            }
            Puzzle::AutomateSensors => {
                self::sensors::SensorsVerifyView::new(right_bottom)
            }
            Puzzle::CommandLander => {
                self::lander::LanderVerifyView::new(right_bottom)
            }
            Puzzle::FabricateEggTimer => FabricationVerifyView::<
                FabricateEggTimerEval,
            >::new(right_bottom),
            Puzzle::FabricateHalve => {
                FabricationVerifyView::<FabricateHalveEval>::new(right_bottom)
            }
            Puzzle::FabricateInc => {
                FabricationVerifyView::<FabricateIncEval>::new(right_bottom)
            }
            Puzzle::FabricateMul => {
                FabricationVerifyView::<FabricateMulEval>::new(right_bottom)
            }
            Puzzle::FabricateStopwatch => FabricationVerifyView::<
                FabricateStopwatchEval,
            >::new(right_bottom),
            Puzzle::FabricateXor => {
                FabricationVerifyView::<FabricateXorEval>::new(right_bottom)
            }
            Puzzle::SandboxBehavior => self::shared::NullVerifyView::new(),
            Puzzle::SandboxEvent => self::shared::NullVerifyView::new(),
            Puzzle::TutorialAdd => {
                FabricationVerifyView::<TutorialAddEval>::new(right_bottom)
            }
            Puzzle::TutorialDemux => {
                FabricationVerifyView::<TutorialDemuxEval>::new(right_bottom)
            }
            Puzzle::TutorialMux => {
                FabricationVerifyView::<TutorialMuxEval>::new(right_bottom)
            }
            Puzzle::TutorialOr => {
                FabricationVerifyView::<TutorialOrEval>::new(right_bottom)
            }
            Puzzle::TutorialSum => {
                FabricationVerifyView::<TutorialSumEval>::new(right_bottom)
            }
        };
        let subview_size = subview.size();
        let rect = if subview_size.is_empty() {
            Rect::new(window_size.width, window_size.height, 0, 0)
        } else {
            let size = subview_size.expand(TRAY_INNER_MARGIN);
            Rect::new(
                window_size.width - size.width,
                window_size.height - size.height,
                size.width,
                size.height + 20,
            )
        };
        VerificationTray { rect, subview, slide: TraySlide::new(rect.width) }
    }

    fn slid_rect(&self) -> Rect<i32> {
        self.rect + vec2(self.slide.distance(), 0)
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        circuit_eval: Option<&CircuitEval>,
    ) {
        if self.rect.is_empty() {
            return;
        }
        let matrix =
            matrix * Matrix4::trans2(self.slide.distance() as f32, 0.0);
        let ui = resources.shaders().ui();
        let rect = self.rect.as_f32();
        let tab_rect =
            UiShader::tray_tab_rect(rect, TRAY_TAB_HEIGHT, TRAY_FLIP_HORZ);
        ui.draw_tray(
            &matrix,
            &rect,
            TRAY_TAB_HEIGHT,
            TRAY_FLIP_HORZ,
            &Color4::ORANGE2,
            &Color4::CYAN2,
            &Color4::PURPLE0_TRANSLUCENT,
        );

        let tab_matrix = matrix
            * Matrix4::trans2(
                tab_rect.x + 0.5 * tab_rect.width,
                tab_rect.y + 0.5 * tab_rect.height,
            )
            * Matrix4::from_angle_z(Deg(90.0));
        let font = resources.fonts().roman();
        font.draw(
            &tab_matrix,
            TRAY_TAB_FONT_SIZE,
            Align::MidCenter,
            (0.0, -2.0),
            TRAY_TAB_TEXT,
        );

        self.subview.draw(resources, &matrix, circuit_eval);
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui) -> bool {
        match event {
            Event::ClockTick(tick) => {
                self.slide.on_tick(tick, ui);
            }
            Event::MouseDown(mouse) => {
                let rel_mouse_pt = mouse.pt - vec2(self.slide.distance(), 0);
                let tab_rect = UiShader::tray_tab_rect(
                    self.rect.as_f32(),
                    TRAY_TAB_HEIGHT,
                    TRAY_FLIP_HORZ,
                );
                if tab_rect.contains_point(rel_mouse_pt.as_f32()) {
                    self.slide.toggle();
                    return true;
                } else if self.rect.contains_point(rel_mouse_pt) {
                    return true;
                }
            }
            Event::MouseMove(mouse) | Event::MouseUp(mouse) => {
                let rel_mouse_pt = mouse.pt - vec2(self.slide.distance(), 0);
                let tab_rect = UiShader::tray_tab_rect(
                    self.rect.as_f32(),
                    TRAY_TAB_HEIGHT,
                    TRAY_FLIP_HORZ,
                );
                if self.rect.contains_point(rel_mouse_pt)
                    || tab_rect.contains_point(rel_mouse_pt.as_f32())
                {
                    ui.cursor().request(Cursor::default());
                }
            }
            Event::Multitouch(touch)
                if self.slid_rect().contains_point(touch.pt) =>
            {
                return true;
            }
            Event::Scroll(scroll)
                if self.slid_rect().contains_point(scroll.pt) =>
            {
                return true;
            }
            _ => {}
        }
        return false;
    }
}

//===========================================================================//
