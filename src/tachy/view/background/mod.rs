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

mod odyssey;
mod planetfall;
mod shared;

pub use self::odyssey::OdysseyBackgroundView;
pub use self::planetfall::PlanetfallBackgroundView;
pub use self::shared::BackgroundView;
use tachy::geom::RectSize;
use tachy::save::Chapter;

//===========================================================================//

pub fn background_for_chapter(chapter: Chapter, screen_size: RectSize<f32>)
                              -> Box<BackgroundView> {
    match chapter {
        Chapter::Odyssey => Box::new(OdysseyBackgroundView::new(screen_size)),
        // TODO: other chapter backgrounds
        _ => Box::new(PlanetfallBackgroundView::new(screen_size)),
    }
}

//===========================================================================//
