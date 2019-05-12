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

use cgmath::{self, Point2};
use tachy::font::Align;
use tachy::geom::{AsFloat, Rect, RectSize};
use tachy::gui::{Event, Keycode, Resources, Ui};
use tachy::state::{CutsceneScript, Theater};

//===========================================================================//

pub enum CutsceneAction {
    Finished,
}

//===========================================================================//

pub struct CutsceneView {
    size: RectSize<f32>,
    theater: Theater,
}

impl CutsceneView {
    pub fn new(window_size: RectSize<i32>) -> CutsceneView {
        CutsceneView {
            size: window_size.as_f32(),
            theater: Theater::new(),
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
        resources
            .shaders()
            .solid()
            .fill_rect(&matrix, (color.r, color.g, color.b), rect);
        if cutscene.is_paused() {
            resources.fonts().roman().draw(&matrix,
                                           20.0,
                                           Align::TopCenter,
                                           (0.5 * self.size.width, 8.0),
                                           "Click to unpause");
        }
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui,
                    cutscene: &mut CutsceneScript)
                    -> Option<CutsceneAction> {
        match event {
            Event::ClockTick(tick) => {
                let done = cutscene.tick(tick.elapsed, ui, &mut self.theater);
                if done {
                    return Some(CutsceneAction::Finished);
                }
            }
            Event::KeyDown(key) if key.code == Keycode::Escape => {
                // TODO: Only skip if we press escape twice in a row.
                cutscene.skip(&mut self.theater);
            }
            Event::MouseDown(_) => {
                cutscene.unpause();
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
