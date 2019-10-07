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
use tachy::save::ScoreCurveMap;

//===========================================================================//

const CACHE_FILE_NAME: &str = "cache.toml";

//===========================================================================//

pub struct GlobalScoresDir {
    base_path: PathBuf,
}

impl GlobalScoresDir {
    pub(super) fn create_or_load(
        base_path: &Path,
    ) -> Result<GlobalScoresDir, String> {
        // Create directory if needed:
        if !base_path.exists() {
            debug_log!("Creating global scores directory at {:?}", base_path);
            fs::create_dir_all(&base_path).map_err(|err| {
                format!(
                    "Could not create global scores directory at {:?}: {}",
                    base_path, err
                )
            })?;
        } else {
            debug_log!("Loading global scores directory from {:?}", base_path);
        }

        Ok(GlobalScoresDir { base_path: base_path.to_path_buf() })
    }

    pub fn load_global_score_cache(&self) -> Result<ScoreCurveMap, String> {
        let path = self.base_path.join(CACHE_FILE_NAME);
        if path.exists() {
            let serialized = fs::read_to_string(&path).map_err(|err| {
                format!(
                    "Could not read global scores cache from {:?}: {}",
                    path, err
                )
            })?;
            ScoreCurveMap::deserialize_from_string(&serialized)
        } else {
            Ok(ScoreCurveMap::new())
        }
    }
}

//===========================================================================//
