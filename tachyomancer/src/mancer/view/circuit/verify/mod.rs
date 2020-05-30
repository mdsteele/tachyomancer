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
mod fuel;
mod grapple;
mod heliostat;
mod injector;
mod lander;
mod mining;
mod reactor;
mod robotarm;
mod sensors;
mod shared;
mod shields;
mod sonar;
mod storage;
mod translator;
mod turret;

use self::shared::{FabricationVerifyView, PuzzleVerifyView};
use super::super::tooltip::TooltipSink;
use super::tray::TraySlide;
use crate::mancer::font::Align;
use crate::mancer::gui::{Cursor, Event, Resources, Ui};
use crate::mancer::shader::UiShader;
use cgmath::{vec2, Deg, Matrix4, Point2};
use tachy::geom::{AsFloat, Color4, MatrixExt, Rect, RectSize};
use tachy::save::Puzzle;
use tachy::state::{
    CircuitEval, FABRICATE_COUNTER_DATA, FABRICATE_EGG_TIMER_DATA,
    FABRICATE_HALVE_DATA, FABRICATE_INC_DATA, FABRICATE_MUL_DATA,
    FABRICATE_QUEUE_DATA, FABRICATE_STACK_DATA, FABRICATE_STOPWATCH_DATA,
    FABRICATE_XOR_DATA, TUTORIAL_ADD_DATA, TUTORIAL_AMP_DATA,
    TUTORIAL_CLOCK_DATA, TUTORIAL_DEMUX_DATA, TUTORIAL_MUX_DATA,
    TUTORIAL_OR_DATA, TUTORIAL_RAM_DATA, TUTORIAL_SUM_DATA,
};

//===========================================================================//

const TRAY_EXTRA_HIDDEN_HEIGHT: i32 = 20;
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
            Puzzle::AutomateCollector => {
                // TODO: Make a verification view for AutomateCollector
                self::shared::NullVerifyView::new()
            }
            Puzzle::AutomateDrillingRig => {
                // TODO: Make a verification view for AutomateDrillingRig
                self::shared::NullVerifyView::new()
            }
            Puzzle::AutomateFuelSynthesis => {
                self::fuel::FuelVerifyView::new(right_bottom)
            }
            Puzzle::AutomateGeigerCounter => {
                // TODO: Make a verification view for AutomateGeigerCounter
                self::shared::NullVerifyView::new()
            }
            Puzzle::AutomateGrapple => {
                self::grapple::GrappleVerifyView::new(right_bottom)
            }
            Puzzle::AutomateHeliostat => {
                self::heliostat::HeliostatVerifyView::new(right_bottom)
            }
            Puzzle::AutomateIncubator => {
                // TODO: Make a verification view for AutomateIncubator
                self::shared::NullVerifyView::new()
            }
            Puzzle::AutomateInjector => {
                self::injector::InjectorVerifyView::new(right_bottom)
            }
            Puzzle::AutomateMiningRobot => {
                self::mining::MiningRobotVerifyView::new(right_bottom)
            }
            Puzzle::AutomateReactor => {
                self::reactor::ReactorVerifyView::new(right_bottom)
            }
            Puzzle::AutomateResonator => {
                // TODO: Make a verification view for AutomateResonator
                self::shared::NullVerifyView::new()
            }
            Puzzle::AutomateRobotArm => {
                self::robotarm::RobotArmVerifyView::new(right_bottom)
            }
            Puzzle::AutomateSensors => {
                self::sensors::SensorsVerifyView::new(right_bottom)
            }
            Puzzle::AutomateSonar => {
                self::sonar::SonarVerifyView::new(right_bottom)
            }
            Puzzle::AutomateStorageDepot => {
                self::storage::StorageDepotVerifyView::new(right_bottom)
            }
            Puzzle::AutomateTranslator => {
                self::translator::TranslatorVerifyView::new(right_bottom)
            }
            Puzzle::AutomateXUnit => {
                // TODO: Make a verification view for AutomateXUnit
                self::shared::NullVerifyView::new()
            }
            Puzzle::CommandLander => {
                self::lander::LanderVerifyView::new(right_bottom)
            }
            Puzzle::CommandShields => {
                self::shields::ShieldsVerifyView::new(right_bottom)
            }
            Puzzle::CommandTurret => {
                self::turret::TurretVerifyView::new(right_bottom)
            }
            Puzzle::FabricateCounter => FabricationVerifyView::new(
                right_bottom,
                FABRICATE_COUNTER_DATA,
            ),
            Puzzle::FabricateEggTimer => FabricationVerifyView::new(
                right_bottom,
                FABRICATE_EGG_TIMER_DATA,
            ),
            Puzzle::FabricateHalve => {
                FabricationVerifyView::new(right_bottom, FABRICATE_HALVE_DATA)
            }
            Puzzle::FabricateInc => {
                FabricationVerifyView::new(right_bottom, FABRICATE_INC_DATA)
            }
            Puzzle::FabricateMul => {
                FabricationVerifyView::new(right_bottom, FABRICATE_MUL_DATA)
            }
            Puzzle::FabricateQueue => {
                FabricationVerifyView::new(right_bottom, FABRICATE_QUEUE_DATA)
            }
            Puzzle::FabricateStack => {
                FabricationVerifyView::new(right_bottom, FABRICATE_STACK_DATA)
            }
            Puzzle::FabricateStopwatch => FabricationVerifyView::new(
                right_bottom,
                FABRICATE_STOPWATCH_DATA,
            ),
            Puzzle::FabricateXor => {
                FabricationVerifyView::new(right_bottom, FABRICATE_XOR_DATA)
            }
            Puzzle::SandboxBehavior => self::shared::NullVerifyView::new(),
            Puzzle::SandboxEvent => self::shared::NullVerifyView::new(),
            Puzzle::TutorialAdd => {
                FabricationVerifyView::new(right_bottom, TUTORIAL_ADD_DATA)
            }
            Puzzle::TutorialAmp => {
                FabricationVerifyView::new(right_bottom, TUTORIAL_AMP_DATA)
            }
            Puzzle::TutorialClock => {
                FabricationVerifyView::new(right_bottom, TUTORIAL_CLOCK_DATA)
            }
            Puzzle::TutorialDemux => {
                FabricationVerifyView::new(right_bottom, TUTORIAL_DEMUX_DATA)
            }
            Puzzle::TutorialMux => {
                FabricationVerifyView::new(right_bottom, TUTORIAL_MUX_DATA)
            }
            Puzzle::TutorialOr => {
                FabricationVerifyView::new(right_bottom, TUTORIAL_OR_DATA)
            }
            Puzzle::TutorialRam => {
                FabricationVerifyView::new(right_bottom, TUTORIAL_RAM_DATA)
            }
            Puzzle::TutorialSum => {
                FabricationVerifyView::new(right_bottom, TUTORIAL_SUM_DATA)
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
                size.height + TRAY_EXTRA_HIDDEN_HEIGHT,
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

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        tooltip: &mut dyn TooltipSink<()>,
    ) -> bool {
        match event {
            Event::ClockTick(tick) => self.slide.on_clock_tick(tick, ui),
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
                    tooltip.hover_none(ui);
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
