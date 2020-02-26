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
use super::profile::{is_valid_profile_name, Profile};
use super::score::GlobalScoresDir;
use directories::ProjectDirs;
use std::collections::{btree_set, BTreeSet};
use std::fs;
use std::path::PathBuf;
use unicase::UniCase;

//===========================================================================//

// Note: this dir name needs to have a period (or other special character) to
// ensure that it cannot conflict with any encoded profile name.
const GLOBAL_SCORES_DIR_NAME: &str = "global.scores";

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
            None => get_default_save_dir_path().map_err(|err| {
                format!("Could not find save data directory: {}", err)
            })?,
        };
        debug_log!("Using save data directory: {:?}", base_path);
        if !base_path.exists() {
            fs::create_dir_all(&base_path).map_err(|err| {
                format!("Could not create save data directory: {}", err)
            })?;
        }

        // Load prefs.
        let prefs_path = base_path.join(PREFS_FILE_NAME);
        let mut prefs = Prefs::create_or_load(&prefs_path)?;

        // Load list of profiles.
        let mut profile_names = BTreeSet::<UniCase<String>>::new();
        let entries = base_path.read_dir().map_err(|err| {
            format!("Could not read contents of save data directory: {}", err)
        })?;
        for entry_result in entries {
            let entry = entry_result.map_err(|err| {
                format!(
                    "Error while reading contents of save data \
                     directory: {}",
                    err
                )
            })?;
            let entry_path = entry.path();
            if !entry_path.is_dir()
                || entry_path.file_name()
                    == Some(GLOBAL_SCORES_DIR_NAME.as_ref())
            {
                continue;
            }
            let profile_name = decode_name(&entry.file_name());
            if is_valid_profile_name(&profile_name) {
                profile_names.insert(UniCase::new(profile_name));
            }
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

        Ok(SaveDir { base_path, prefs, profile_names })
    }

    pub fn prefs(&self) -> &Prefs {
        &self.prefs
    }

    pub fn prefs_mut(&mut self) -> &mut Prefs {
        &mut self.prefs
    }

    pub fn save(&mut self) -> Result<(), String> {
        self.prefs.save()
    }

    pub fn profile_names(&self) -> ProfileNamesIter {
        ProfileNamesIter { inner: self.profile_names.iter() }
    }

    /// Returns true if a profile with the given name already exists, ignoring
    /// case.
    pub fn has_profile(&self, name: &str) -> bool {
        // It would be nice to avoid the string copy here (and elsewhere in
        // this file) if UniCase ever implements the Borrow trait
        // (https://github.com/seanmonstar/unicase/issues/22).
        self.profile_names.contains(&UniCase::new(name.to_string()))
    }

    /// Returns true if the current profile has the given name, ignoring case.
    pub fn current_profile_is(&self, name: &str) -> bool {
        match self.prefs.current_profile() {
            None => false,
            Some(other) => UniCase::new(name) == UniCase::new(other),
        }
    }

    /// If the given profile name exists (ignoring case), returns the name of
    /// that profile (in its actual case); otherwise, returns the given name
    /// unchanged.
    fn canonicalize_profile_name(&self, name: &str) -> String {
        match self.profile_names.get(&UniCase::new(name.to_string())) {
            Some(unicase_name) => unicase_name.clone().into_inner(),
            None => name.to_string(),
        }
    }

    pub fn load_current_profile_if_any(
        &self,
    ) -> Result<Option<Profile>, String> {
        if let Some(name) = self.prefs.current_profile() {
            let path = self.base_path.join(encode_name(name));
            let profile = Profile::create_or_load(name.to_string(), &path)?;
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    pub fn load_profile(&self, name: &str) -> Result<Profile, String> {
        if !is_valid_profile_name(name) {
            return Err(format!("Invalid profile name: {:?}", name));
        }
        if !self.has_profile(name) {
            return Err(format!("No such profile: {:?}", name));
        }
        let name = self.canonicalize_profile_name(name);
        let path = self.base_path.join(encode_name(&name));
        let profile = Profile::create_or_load(name, &path)?;
        return Ok(profile);
    }

    pub fn create_or_load_and_set_profile(
        &mut self,
        name: &str,
    ) -> Result<Profile, String> {
        if !is_valid_profile_name(name) {
            return Err(format!("Invalid profile name: {:?}", name));
        }
        let name = self.canonicalize_profile_name(name);
        let is_current_profile = self.current_profile_is(&name);
        let path = self.base_path.join(encode_name(&name));
        let profile = Profile::create_or_load(name, &path)?;
        if !is_current_profile {
            self.prefs.set_current_profile(Some(profile.name().to_string()));
            self.prefs.save()?;
        }
        self.profile_names.insert(UniCase::new(profile.name().to_string()));
        return Ok(profile);
    }

    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        if !is_valid_profile_name(name) {
            return Err(format!("Invalid profile name: {:?}", name));
        }
        if !self.has_profile(name) {
            return Err(format!("No such profile: {:?}", name));
        }
        let name = self.canonicalize_profile_name(name);
        let is_current_profile = self.current_profile_is(&name);
        self.profile_names.remove(&UniCase::new(name.clone()));
        if is_current_profile {
            self.prefs.set_current_profile(
                self.profile_names().next().map(str::to_string),
            );
        }
        let path = self.base_path.join(encode_name(&name));
        debug_log!("Deleting profile {:?} from {:?}", name, path);
        fs::remove_dir_all(&path).map_err(|err| {
            format!(
                "Could not delete profile {:?} data from {:?}: {}",
                name, path, err
            )
        })?;
        return Ok(());
    }

    pub fn create_or_load_global_scores(
        &mut self,
    ) -> Result<GlobalScoresDir, String> {
        let path = self.base_path.join(GLOBAL_SCORES_DIR_NAME);
        GlobalScoresDir::create_or_load(&path)
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

fn get_default_save_dir_path() -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("games", "mdsteele", "Tachyomancer")
        .ok_or_else(|| "No valid home directory found.".to_string())?;
    Ok(project_dirs.data_dir().to_path_buf())
}

//===========================================================================//
