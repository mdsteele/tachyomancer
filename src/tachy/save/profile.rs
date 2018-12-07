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

use super::puzzle::Puzzle;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

//===========================================================================//

const DATA_FILE_NAME: &str = "profile.toml";

//===========================================================================//

#[derive(Default, Deserialize, Serialize)]
struct ProfileData {
    current_puzzle: Option<Puzzle>,
}

impl ProfileData {
    fn try_load(path: &Path) -> io::Result<ProfileData> {
        toml::from_slice(&fs::read(path)?).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("{}", err))
        })
    }
}

//===========================================================================//

#[allow(dead_code)]
pub struct Profile {
    name: String,
    base_path: PathBuf,
    data: ProfileData,
    needs_save: bool,
    // TODO: puzzle progress
}

impl Profile {
    pub fn create_or_load(name: String, base_path: &Path)
                          -> Result<Profile, String> {
        if !base_path.exists() {
            debug_log!("Creating profile {:?} at {:?}", name, base_path);
            fs::create_dir_all(&base_path)
                .map_err(|err| {
                    format!("Could not create profile {:?} \
                             directory at {:?}: {:?}",
                            name,
                            base_path,
                            err)
                })?;
        } else {
            debug_log!("Loading profile {:?} from {:?}", name, base_path);
        }

        let mut needs_save = false;
        let data_path = base_path.join(DATA_FILE_NAME);
        let data = if data_path.exists() {
            match ProfileData::try_load(&data_path) {
                Ok(data) => data,
                Err(err) => {
                    debug_log!("Could not read profile {:?} \
                                data file from {:?}: {}",
                               name,
                               data_path,
                               err);
                    ProfileData::default()
                }
            }
        } else {
            needs_save = true;
            ProfileData::default()
        };

        let mut profile = Profile {
            name,
            base_path: base_path.to_path_buf(),
            data,
            needs_save,
        };
        profile.save()?;
        Ok(profile)
    }

    pub fn save(&mut self) -> Result<(), String> {
        if self.needs_save {
            let data_path = self.base_path.join(DATA_FILE_NAME);
            debug_log!("Saving profile {:?} data to {:?}",
                       self.name,
                       data_path);
            let data = toml::to_vec(&self.data)
                .map_err(|err| {
                    format!("Could not serialize profile {:?} data: {}",
                            self.name,
                            err)
                })?;
            fs::write(&data_path, data)
                .map_err(|err| {
                             format!("Could not write profile {:?} \
                                      data file to {:?}: {}",
                                     self.name,
                                     data_path,
                                     err)
                         })?;
            self.needs_save = false;
        }
        Ok(())
    }

    pub fn name(&self) -> &str { &self.name }

    pub fn current_puzzle(&self) -> Puzzle {
        self.data.current_puzzle.unwrap_or(Puzzle::first())
    }

    pub fn set_current_puzzle(&mut self, puzzle: Puzzle) {
        self.data.current_puzzle = Some(puzzle);
        self.needs_save = true;
    }
}

//===========================================================================//
