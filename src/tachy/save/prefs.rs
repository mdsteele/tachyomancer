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
use std::path::Path;
use toml;

//===========================================================================//

#[derive(Default, Deserialize, Serialize)]
pub struct Prefs {
    fullscreen: Option<bool>,
    resolution: Option<(u32, u32)>,
    current_profile: Option<String>,
}

impl Prefs {
    pub fn create_or_load(path: &Path) -> Result<Prefs, String> {
        if path.exists() {
            match Prefs::try_load(path) {
                Ok(prefs) => Ok(prefs),
                Err(err) => {
                    debug_log!("Could not read prefs file: {}", err);
                    Ok(Prefs::default())
                }
            }
        } else {
            let prefs = Prefs::default();
            prefs.save(path)?;
            Ok(prefs)
        }
    }

    fn try_load(path: &Path) -> io::Result<Prefs> {
        toml::from_slice(&fs::read(path)?).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("{}", err))
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        debug_log!("Saving prefs to {:?}", path);
        let data =
            toml::to_vec(self)
                .map_err(|err| format!("Could not serialize prefs: {}", err))?;
        fs::write(path, data)
            .map_err(|err| format!("Could not write prefs file: {}", err))?;
        Ok(())
    }

    pub fn fullscreen(&self) -> bool { self.fullscreen.unwrap_or(true) }

    pub fn resolution(&self) -> Option<(u32, u32)> { self.resolution }

    pub fn current_profile(&self) -> Option<&str> {
        self.current_profile.as_ref().map(String::as_str)
    }

    pub fn set_current_profile(&mut self, profile: Option<String>) {
        self.current_profile = profile;
    }
}

//===========================================================================//
