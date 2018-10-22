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

use super::prefs::Prefs;
use app_dirs::{self, AppDataType, AppDirsError, AppInfo};
use std::fs;
use std::path::PathBuf;

//===========================================================================//

const PREFS_FILE_NAME: &str = "prefs.toml";

//===========================================================================//

#[allow(dead_code)]
pub struct SaveDir {
    base_path: PathBuf,
    prefs: Prefs,
    // TODO: profiles: Vec<String>,
}

impl SaveDir {
    pub fn create_or_load(path: &Option<PathBuf>) -> Result<SaveDir, String> {
        let base_path = match path {
            Some(p) => p.clone(),
            None => {
                get_default_save_dir_path()
                    .map_err(|err| {
                        format!("Could not find save data directory: {:?}",
                                err)
                    })?
            }
        };
        debug_log!("Using save data directory: {:?}", base_path);
        if !base_path.exists() {
            fs::create_dir_all(&base_path)
                .map_err(|err| {
                    format!("Could not create save data directory: {:?}", err)
                })?;
        }
        let prefs = Prefs::create_or_load(&base_path.join(PREFS_FILE_NAME))?;
        Ok(SaveDir { base_path, prefs })
    }

    pub fn prefs(&self) -> &Prefs { &self.prefs }

    // TODO: fn load_profile(name: &str)
}

//===========================================================================//

const APP_INFO: AppInfo = AppInfo {
    name: "Tachyomancer",
    author: "mdsteele",
};

fn get_default_save_dir_path() -> Result<PathBuf, String> {
    app_dirs::app_root(AppDataType::UserData, &APP_INFO)
        .map_err(|err| match err {
                     AppDirsError::Io(error) => format!("IO error: {}", error),
                     AppDirsError::NotSupported => {
                         "App dir not supported".to_string()
                     }
                     AppDirsError::InvalidAppInfo => {
                         "App info invalid".to_string()
                     }
                 })
}

//===========================================================================//
