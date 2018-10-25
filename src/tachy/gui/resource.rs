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

use tachy::shader::Shaders;
use tachy::texture::Textures;

//===========================================================================//

pub struct Resources {
    shaders: Shaders,
    textures: Textures,
}

impl Resources {
    pub(super) fn new() -> Result<Resources, String> {
        let shaders = Shaders::new()?;
        let textures = Textures::new()?;
        Ok(Resources { shaders, textures })
    }

    pub fn shaders(&self) -> &Shaders { &self.shaders }

    pub fn textures(&self) -> &Textures { &self.textures }
}

//===========================================================================//
