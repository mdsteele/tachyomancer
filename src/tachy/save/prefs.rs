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
use std::io;
use std::path::{Path, PathBuf};
use toml;

//===========================================================================//

const DEFAULT_SOUND_VOLUME_PERCENT: i32 = 80;

//===========================================================================//

#[derive(Default, Deserialize, Serialize)]
struct PrefsData {
    antialiasing: Option<bool>,
    current_profile: Option<String>,
    fullscreen: Option<bool>,
    resolution: Option<(u32, u32)>,
    sound_volume: Option<i32>,
}

impl PrefsData {
    fn try_load(path: &Path) -> io::Result<PrefsData> {
        toml::from_slice(&fs::read(path)?).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("{}", err))
        })
    }
}

//===========================================================================//

pub struct Prefs {
    path: PathBuf,
    data: PrefsData,
    needs_save: bool,
}

impl Prefs {
    pub fn create_or_load(path: &Path) -> Result<Prefs, String> {
        let mut needs_save = false;
        let data = if path.exists() {
            match PrefsData::try_load(path) {
                Ok(prefs) => prefs,
                Err(err) => {
                    debug_log!("Could not read prefs file: {}", err);
                    PrefsData::default()
                }
            }
        } else {
            needs_save = true;
            PrefsData::default()
        };
        let mut prefs = Prefs {
            path: path.to_path_buf(),
            data,
            needs_save,
        };
        prefs.save()?;
        Ok(prefs)
    }

    pub fn save(&mut self) -> Result<(), String> {
        if !self.needs_save {
            return Ok(());
        }
        debug_log!("Saving prefs to {:?}", self.path);
        let data =
            toml::to_vec(&self.data)
                .map_err(|err| format!("Could not serialize prefs: {}", err))?;
        fs::write(&self.path, data)
            .map_err(|err| format!("Could not write prefs file: {}", err))?;
        self.needs_save = false;
        return Ok(());
    }

    pub fn antialiasing(&self) -> bool {
        self.data.antialiasing.unwrap_or(false)
    }

    pub fn set_antialiasing(&mut self, antialiasing: bool) {
        self.data.antialiasing = Some(antialiasing);
        self.needs_save = true;
    }

    pub fn current_profile(&self) -> Option<&str> {
        self.data.current_profile.as_ref().map(String::as_str)
    }

    pub fn set_current_profile(&mut self, profile: Option<String>) {
        self.data.current_profile = profile;
        self.needs_save = true;
    }

    pub fn fullscreen(&self) -> bool { self.data.fullscreen.unwrap_or(true) }

    pub fn resolution(&self) -> Option<(u32, u32)> { self.data.resolution }

    pub fn sound_volume_percent(&self) -> i32 {
        self.data
            .sound_volume
            .unwrap_or(DEFAULT_SOUND_VOLUME_PERCENT)
            .max(0)
            .min(100)
    }

    pub fn set_sound_volume_percent(&mut self, percent: i32) {
        self.data.sound_volume = Some(percent.max(0).min(100));
        self.needs_save = true;
    }
}

//===========================================================================//
