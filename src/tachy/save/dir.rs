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
use super::profile::Profile;
use app_dirs::{self, AppDataType, AppDirsError, AppInfo};
use std::char;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;
use std::u32;
use std::u64;

//===========================================================================//

const PREFS_FILE_NAME: &str = "prefs.toml";

//===========================================================================//

pub struct SaveDir {
    base_path: PathBuf,
    prefs: Prefs,
    profile_names: Vec<String>,
}

impl SaveDir {
    pub fn create_or_load(path: &Option<PathBuf>) -> Result<SaveDir, String> {
        // Get or create save dir.
        let base_path: PathBuf = match path {
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

        // Load prefs.
        let prefs_path = base_path.join(PREFS_FILE_NAME);
        let mut prefs = Prefs::create_or_load(&prefs_path)?;

        // Load list of profiles.
        let mut profile_names = Vec::<String>::new();
        let entries = base_path
            .read_dir()
            .map_err(|err| {
                format!("Could not read contents of save data directory: {:?}",
                        err)
            })?;
        for entry_result in entries {
            let entry = entry_result
                .map_err(|err| {
                    format!("Error while reading contents of save data \
                             directory: {:?}",
                            err)
                })?;
            if !entry.path().is_dir() {
                continue;
            }
            profile_names.push(decode_profile_name(&entry.file_name()));
        }
        profile_names.sort();

        // Repair prefs.current_profile if necessary.
        let has_current_profile;
        if let Some(profile) = prefs.current_profile() {
            has_current_profile = profile_names.contains(&profile.to_string());
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
                let profile_name = profile_names[0].clone();
                debug_log!("Defaulting current profile to {:?}", profile_name);
                prefs.set_current_profile(Some(profile_name));
            }
            prefs.save(&prefs_path)?;
        }

        Ok(SaveDir {
               base_path,
               prefs,
               profile_names,
           })
    }

    pub fn prefs(&self) -> &Prefs { &self.prefs }

    pub fn profile_names(&self) -> &[String] { &self.profile_names }

    pub fn load_current_profile_if_any(&self)
                                       -> Result<Option<Profile>, String> {
        if let Some(name) = self.prefs.current_profile() {
            let path = self.base_path.join(encode_profile_name(name));
            let profile = Profile::create_or_load(name.to_string(), &path)?;
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    pub fn create_or_load_profile(&mut self, name: String)
                                  -> Result<Profile, String> {
        let is_current_profile = self.prefs.current_profile() == Some(&name);
        let path = self.base_path.join(encode_profile_name(&name));
        let profile = Profile::create_or_load(name, &path)?;
        if !is_current_profile {
            self.prefs.set_current_profile(Some(profile.name().to_string()));
            self.prefs.save(&self.base_path.join(PREFS_FILE_NAME))?;
        }
        Ok(profile)
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

fn decode_profile_name(file_name: &OsStr) -> String {
    let mut decoded = String::new();
    let string = file_name.to_string_lossy();
    let mut chars = string.chars();
    while let Some(chr) = chars.next() {
        if chr == ',' {
            let mut value: u64 = 0;
            let mut any_digits = false;
            let mut found_comma = false;
            while let Some(digit_chr) = chars.next() {
                if let Some(digit) = digit_chr.to_digit(16) {
                    any_digits = true;
                    value = value * 16 + (digit as u64);
                    if value > (u32::MAX as u64) {
                        break;
                    }
                } else if digit_chr == ',' {
                    found_comma = true;
                    break;
                } else {
                    break;
                }
            }
            if !found_comma {
                // Skip past next comma:
                while let Some(digit_chr) = chars.next() {
                    if digit_chr == ',' {
                        break;
                    }
                }
            } else if any_digits && value <= (u32::MAX as u64) {
                if let Some(decoded_chr) = char::from_u32(value as u32) {
                    decoded.push(decoded_chr);
                    continue;
                }
            }
            decoded.push(char::REPLACEMENT_CHARACTER);
        } else if chr == '_' {
            decoded.push(' ');
        } else {
            decoded.push(chr);
        }
    }
    decoded
}

fn encode_profile_name(profile_name: &str) -> OsString {
    let mut encoded = String::new();
    for chr in profile_name.chars() {
        if chr.is_ascii_alphanumeric() || chr == '-' {
            encoded.push(chr);
        } else if chr == ' ' {
            encoded.push('_');
        } else {
            encoded.push_str(&format!(",{:x},", chr as u32));
        }
    }
    OsString::from(encoded)
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{decode_profile_name, encode_profile_name};
    use std::char;
    use std::ffi::OsString;

    #[test]
    fn profile_name_round_trip() {
        assert_eq!(encode_profile_name(""), OsString::from(""));
        assert_eq!(decode_profile_name(&OsString::from("")), "".to_string());

        assert_eq!(encode_profile_name("Jane Doe-99"),
                   OsString::from("Jane_Doe-99"));
        assert_eq!(decode_profile_name(&OsString::from("Jane_Doe-99")),
                   "Jane Doe-99".to_string());

        assert_eq!(encode_profile_name("Jane_Doe-99"),
                   OsString::from("Jane,5f,Doe-99"));
        assert_eq!(decode_profile_name(&OsString::from("Jane,5f,Doe-99")),
                   "Jane_Doe-99".to_string());

        assert_eq!(encode_profile_name(".."), OsString::from(",2e,,2e,"));
        assert_eq!(decode_profile_name(&OsString::from(",2e,,2e,")),
                   "..".to_string());

        assert_eq!(encode_profile_name("prefs.toml"),
                   OsString::from("prefs,2e,toml"));
        assert_eq!(decode_profile_name(&OsString::from("prefs,2e,toml")),
                   "prefs.toml".to_string());

        assert_eq!(encode_profile_name("/Users/janedoe/*"),
                   OsString::from(",2f,Users,2f,janedoe,2f,,2a,"));
        assert_eq!(decode_profile_name(&OsString::from(",2f,Users,2f,janedoe\
                                                        ,2f,,2a,")),
                   "/Users/janedoe/*".to_string());

        assert_eq!(encode_profile_name("Snowman \u{2603}"),
                   OsString::from("Snowman_,2603,"));
        assert_eq!(decode_profile_name(&OsString::from("Snowman_,2603,")),
                   "Snowman \u{2603}".to_string());
    }

    #[test]
    fn profile_name_decode_errors() {
        // No hex digits:
        assert_eq!(decode_profile_name(&OsString::from("Foo,,Bar")),
                   "Foo\u{fffd}Bar".to_string());
        // No closing comma:
        assert_eq!(decode_profile_name(&OsString::from("Foo,2f")),
                   "Foo\u{fffd}".to_string());
        // Invalid hex digits:
        assert_eq!(decode_profile_name(&OsString::from("Foo,efgh,Bar")),
                   "Foo\u{fffd}Bar".to_string());
        // Too large a value:
        assert_eq!(decode_profile_name(&OsString::from("Foo,1234567890,Bar")),
                   "Foo\u{fffd}Bar".to_string());
        // Invalid Unicode character:
        assert!(char::from_u32(0xd800).is_none());
        assert_eq!(decode_profile_name(&OsString::from("Foo,d800,Bar")),
                   "Foo\u{fffd}Bar".to_string());
    }
}

//===========================================================================//
