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

use std::fs;
use std::path::{Path, PathBuf};

//===========================================================================//

#[allow(dead_code)]
pub struct Profile {
    name: String,
    path: PathBuf,
    // TODO: puzzle progress
}

impl Profile {
    pub fn create_or_load(name: String, path: &Path)
                          -> Result<Profile, String> {
        if !path.exists() {
            debug_log!("Creating profile {:?} at {:?}", name, path);
            fs::create_dir_all(&path)
                .map_err(|err| {
                    format!("Could not create profile directory: {:?}", err)
                })?;
        } else {
            debug_log!("Loading profile {:?} from {:?}", name, path);
        }

        Ok(Profile {
               name,
               path: path.to_path_buf(),
           })
    }

    pub fn name(&self) -> &str { &self.name }
}

//===========================================================================//
