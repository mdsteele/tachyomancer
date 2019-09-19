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

use crate::tachy::gui::{ClockEventData, Ui};

//===========================================================================//

pub struct TraySlide {
    max_slide: f64,
    slide: f64,
    shown: bool,
}

impl TraySlide {
    pub fn new(width: i32) -> TraySlide {
        TraySlide { max_slide: (width + 1) as f64, slide: 0.0, shown: true }
    }

    pub fn toggle(&mut self) {
        self.shown = !self.shown;
    }

    pub fn distance(&self) -> i32 {
        self.slide.round() as i32
    }

    pub fn on_tick(&mut self, tick: &ClockEventData, ui: &mut Ui) {
        let goal = if self.shown { 0.0 } else { self.max_slide };
        let new_slide = track_towards(self.slide, goal, tick);
        if self.slide != new_slide {
            self.slide = new_slide;
            ui.request_redraw();
        }
    }
}

fn track_towards(current: f64, goal: f64, tick: &ClockEventData) -> f64 {
    let tracking_base: f64 = 0.00001; // smaller = faster tracking
    let difference = goal - current;
    if difference.abs() < 0.5 {
        goal
    } else {
        let change = difference * (1.0 - tracking_base.powf(tick.elapsed));
        current + change
    }
}

//===========================================================================//
