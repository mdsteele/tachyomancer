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

use super::encode::{decode_name, encode_name};
use super::prefs::Prefs;
use super::profile::Profile;
use app_dirs::{self, AppDataType, AppDirsError, AppInfo};
use std::collections::{BTreeSet, btree_set};
use std::fs;
use std::path::PathBuf;
use unicase::UniCase;

//===========================================================================//

const PREFS_FILE_NAME: &str = "prefs.toml";

//===========================================================================//

pub struct SaveDir {
    base_path: PathBuf,
    prefs: Prefs,
    profile_names: BTreeSet<UniCase<String>>,
}

impl SaveDir {
    pub fn create_or_load(path: &Option<PathBuf>) -> Result<SaveDir, String> {
        // Get or create save dir.
        let base_path: PathBuf = match path {
            Some(p) => p.clone(),
            None => {
                get_default_save_dir_path()
                    .map_err(|err| {
                        format!("Could not find save data directory: {}", err)
                    })?
            }
        };
        debug_log!("Using save data directory: {:?}", base_path);
        if !base_path.exists() {
            fs::create_dir_all(&base_path)
                .map_err(|err| {
                    format!("Could not create save data directory: {}", err)
                })?;
        }

        // Load prefs.
        let prefs_path = base_path.join(PREFS_FILE_NAME);
        let mut prefs = Prefs::create_or_load(&prefs_path)?;

        // Load list of profiles.
        let mut profile_names = BTreeSet::<UniCase<String>>::new();
        let entries = base_path
            .read_dir()
            .map_err(|err| {
                format!("Could not read contents of save data directory: {}",
                        err)
            })?;
        for entry_result in entries {
            let entry = entry_result
                .map_err(|err| {
                    format!("Error while reading contents of save data \
                             directory: {}",
                            err)
                })?;
            if !entry.path().is_dir() {
                continue;
            }
            let profile_name = decode_name(&entry.file_name());
            profile_names.insert(UniCase::new(profile_name));
        }

        // Repair prefs.current_profile if necessary.
        let has_current_profile;
        if let Some(profile) = prefs.current_profile() {
            has_current_profile =
                profile_names.contains(&UniCase::new(profile.to_string()));
            if !has_current_profile {
                debug_log!("Profile {:?} does not exist", profile);
            }
        } else {
            has_current_profile = profile_names.is_empty();
        }
        if !has_current_profile {
            if profile_names.is_empty() {
                debug_log!("Setting current profile to None");
                prefs.set_current_profile(None);
            } else {
                let name: &str = &profile_names.iter().next().unwrap();
                debug_log!("Defaulting current profile to {:?}", name);
                prefs.set_current_profile(Some(name.to_string()));
            }
            prefs.save()?;
        }

        Ok(SaveDir {
               base_path,
               prefs,
               profile_names,
           })
    }

    pub fn prefs(&self) -> &Prefs { &self.prefs }

    pub fn prefs_mut(&mut self) -> &mut Prefs { &mut self.prefs }

    pub fn save(&mut self) -> Result<(), String> { self.prefs.save() }

    pub fn profile_names(&self) -> ProfileNamesIter {
        ProfileNamesIter { inner: self.profile_names.iter() }
    }

    pub fn has_profile(&self, name: &str) -> bool {
        // It would be nice to avoid the string copy here (and elsewhere in
        // this file) if UniCase ever implements the Borrow trait
        // (https://github.com/seanmonstar/unicase/issues/22).
        self.profile_names.contains(&UniCase::new(name.to_string()))
    }

    pub fn load_current_profile_if_any(&self)
                                       -> Result<Option<Profile>, String> {
        if let Some(name) = self.prefs.current_profile() {
            let path = self.base_path.join(encode_name(name));
            let profile = Profile::create_or_load(name.to_string(), &path)?;
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    pub fn create_or_load_profile(&mut self, name: String)
                                  -> Result<Profile, String> {
        let is_current_profile = self.prefs.current_profile() == Some(&name);
        let path = self.base_path.join(encode_name(&name));
        let profile = Profile::create_or_load(name, &path)?;
        if !is_current_profile {
            self.prefs.set_current_profile(Some(profile.name().to_string()));
            self.prefs.save()?;
        }
        self.profile_names.insert(UniCase::new(profile.name().to_string()));
        Ok(profile)
    }
}

//===========================================================================//

pub struct ProfileNamesIter<'a> {
    inner: btree_set::Iter<'a, UniCase<String>>,
}

impl<'a> Iterator for ProfileNamesIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.inner.next().map(UniCase::as_ref)
    }
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
