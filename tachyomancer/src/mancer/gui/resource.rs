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

use crate::mancer::font::Fonts;
use crate::mancer::shader::Shaders;
use crate::mancer::texture::Textures;
use std::sync::{Arc, Mutex};
use tachy::geom::RectSize;
use tachy::save::{Puzzle, ScoreCurve, ScoreCurveMap};

//===========================================================================//

pub struct Resources {
    window_size: RectSize<i32>,
    fonts: Fonts,
    global_scores: Arc<Mutex<ScoreCurveMap>>,
    shaders: Shaders,
    textures: Textures,
}

impl Resources {
    pub(super) fn new(
        window_size: RectSize<i32>,
        global_scores: Arc<Mutex<ScoreCurveMap>>,
    ) -> Result<Resources, String> {
        let fonts = Fonts::new()?;
        let shaders = Shaders::new(window_size)?;
        let textures = Textures::new()?;
        Ok(Resources { window_size, fonts, global_scores, shaders, textures })
    }

    pub fn window_size(&self) -> RectSize<i32> {
        self.window_size
    }

    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    pub fn global_scores_for(&self, puzzle: Puzzle) -> ScoreCurve {
        self.global_scores.lock().unwrap().get(puzzle).clone()
    }

    pub fn shaders(&self) -> &Shaders {
        &self.shaders
    }

    pub fn textures(&self) -> &Textures {
        &self.textures
    }
}

//===========================================================================//
