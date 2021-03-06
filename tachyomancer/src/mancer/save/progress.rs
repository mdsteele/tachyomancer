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
use std::collections::{btree_set, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tachy::save::{CircuitData, ScoreCurve};
use unicase::UniCase;

//===========================================================================//

/// Maximum permitted number of characters in a circuit name.
pub const CIRCUIT_NAME_MAX_CHARS: usize = 20;

// Note: this file name needs to have a period (or other special character) in
// the non-extension part to ensure that it cannot conflict with any encoded
// circuit name.
const DATA_FILE_NAME: &str = "puzzle.progress.toml";

//===========================================================================//

pub fn is_valid_circuit_name(name: &str) -> bool {
    !name.is_empty() && name.chars().count() <= CIRCUIT_NAME_MAX_CHARS
}

//===========================================================================//

#[derive(Default, Deserialize, Serialize)]
pub struct PuzzleProgressData {
    graph: Option<ScoreCurve>,
}

impl PuzzleProgressData {
    fn try_load(path: &Path) -> io::Result<PuzzleProgressData> {
        toml::from_slice(&fs::read(path)?).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("{}", err))
        })
    }

    fn serialize_toml(&self) -> Result<Vec<u8>, String> {
        toml::to_vec(self).map_err(|err| {
            format!("Could not serialize puzzle progress data: {}", err)
        })
    }
}

//===========================================================================//

pub struct PuzzleProgress {
    base_path: PathBuf,
    data: PuzzleProgressData,
    circuit_names: BTreeSet<UniCase<String>>,
    needs_save: bool,
}

impl PuzzleProgress {
    pub fn create_or_load(base_path: &Path) -> Result<PuzzleProgress, String> {
        // Create directory if needed:
        if !base_path.exists() {
            debug_log!("Creating puzzle directory at {:?}", base_path);
            fs::create_dir_all(&base_path).map_err(|err| {
                format!(
                    "Could not create puzzle directory at {:?}: {}",
                    base_path, err
                )
            })?;
        }

        // Load progress data:
        let mut needs_save = false;
        let data_path = base_path.join(DATA_FILE_NAME);
        let data = if data_path.exists() {
            match PuzzleProgressData::try_load(&data_path) {
                Ok(data) => data,
                Err(err) => {
                    debug_log!(
                        "Could not read puzzle progress \
                         data file from {:?}: {}",
                        data_path,
                        err
                    );
                    PuzzleProgressData::default()
                }
            }
        } else {
            needs_save = true;
            PuzzleProgressData::default()
        };

        // Get circuit names:
        let mut circuit_names = BTreeSet::<UniCase<String>>::new();
        let entries = base_path.read_dir().map_err(|err| {
            format!(
                "Could not read contents of puzzle directory {:?}: {}",
                base_path, err
            )
        })?;
        for entry_result in entries {
            let entry = entry_result.map_err(|err| {
                format!(
                    "Error while reading contents of \
                     puzzle directory {:?}: {}",
                    base_path, err
                )
            })?;
            let entry_path = entry.path();
            if entry_path.extension() != Some("toml".as_ref())
                || entry_path.file_name() == Some(DATA_FILE_NAME.as_ref())
            {
                continue;
            }
            if let Some(encoded) = entry_path.file_stem() {
                let circuit_name = decode_name(encoded);
                if is_valid_circuit_name(&circuit_name) {
                    circuit_names.insert(UniCase::new(circuit_name));
                }
            }
        }

        let progress = PuzzleProgress {
            base_path: base_path.to_path_buf(),
            data,
            circuit_names,
            needs_save,
        };
        Ok(progress)
    }

    pub fn save(&mut self) -> Result<(), String> {
        if self.needs_save {
            let data_path = self.base_path.join(DATA_FILE_NAME);
            debug_log!("Saving puzzle progress to {:?}", data_path);
            let data_toml = self.data.serialize_toml()?;
            fs::write(&data_path, data_toml).map_err(|err| {
                format!(
                    "Could not write puzzle progress \
                     data file to {:?}: {}",
                    data_path, err
                )
            })?;
            self.needs_save = false;
        }
        Ok(())
    }

    pub fn is_solved(&self) -> bool {
        !self.local_scores().is_empty()
    }

    pub fn local_scores(&self) -> &ScoreCurve {
        self.data.graph.as_ref().unwrap_or(ScoreCurve::EMPTY)
    }

    pub fn reset_local_scores(&mut self) {
        if self.data.graph.is_some() {
            self.data.graph = None;
            self.needs_save = true;
        }
    }

    pub fn record_score(&mut self, area: i32, score: u32) {
        if let Some(ref mut graph) = self.data.graph {
            graph.insert((area, score));
        } else {
            self.data.graph =
                Some(ScoreCurve::with_scores(vec![(area, score)]));
        }
        self.needs_save = true;
    }

    pub fn circuit_names(&self) -> CircuitNamesIter {
        CircuitNamesIter::new(&self.circuit_names)
    }

    /// Returns true if there is a circuit with the given name for this puzzle.
    /// For the purposes of name collisions, circuit names are treated as
    /// case-insensitive, for better compatibility with case-insensitive
    /// filesystems.
    pub fn has_circuit_name(&self, name: &str) -> bool {
        // It would be nice to avoid the string copy here (and elsewhere in
        // this file) if UniCase ever implements the Borrow trait
        // (https://github.com/seanmonstar/unicase/issues/22).
        self.circuit_names.contains(&UniCase::new(name.to_string()))
    }

    pub fn load_circuit(
        &self,
        circuit_name: &str,
    ) -> Result<CircuitData, String> {
        let circuit_name_uni = UniCase::new(circuit_name.to_string());
        let circuit_path = match self.circuit_names.get(&circuit_name_uni) {
            Some(name) => self.circuit_path(&name),
            None => {
                return Err(format!("No such circuit: {:?}", circuit_name))
            }
        };
        debug_log!(
            "Loading circuit {:?} from {:?}",
            circuit_name,
            circuit_path
        );
        CircuitData::load(&circuit_path)
    }

    pub fn save_circuit(
        &mut self,
        circuit_name: &str,
        circuit_data: &CircuitData,
    ) -> Result<(), String> {
        if !is_valid_circuit_name(circuit_name) {
            return Err(format!("Invalid circuit name: {:?}", circuit_name));
        }
        let circuit_name_uni = UniCase::new(circuit_name.to_string());
        let circuit_path = match self.circuit_names.get(&circuit_name_uni) {
            Some(name) => self.circuit_path(&name),
            None => self.circuit_path(circuit_name),
        };
        debug_log!("Saving circuit {:?} to {:?}", circuit_name, circuit_path);
        circuit_data.save(&circuit_path)?;
        self.circuit_names.insert(circuit_name_uni);
        Ok(())
    }

    pub fn copy_circuit(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), String> {
        let old_name_uni = UniCase::new(old_name.to_string());
        let old_path = match self.circuit_names.get(&old_name_uni) {
            Some(name) => self.circuit_path(&name),
            None => return Err(format!("No such circuit: {:?}", old_name)),
        };
        if !is_valid_circuit_name(new_name) {
            return Err(format!("Invalid circuit name: {:?}", new_name));
        }
        let new_name_uni = UniCase::new(new_name.to_string());
        if self.circuit_names.contains(&new_name_uni) {
            return Err(format!("Circuit already exists: {:?}", new_name));
        }
        let new_path = self.circuit_path(new_name);
        if new_path.exists() {
            return Err(format!("Path already exists: {:?}", new_path));
        }
        debug_log!("Copying circuit from {:?} to {:?}", old_path, new_path);
        fs::copy(&old_path, &new_path).map_err(|err| {
            format!(
                "Could not copy circuit file {:?} to {:?}: {}",
                old_path, new_path, err
            )
        })?;
        self.circuit_names.insert(new_name_uni);
        Ok(())
    }

    pub fn delete_circuit(
        &mut self,
        circuit_name: &str,
    ) -> Result<(), String> {
        let circuit_name_uni = UniCase::new(circuit_name.to_string());
        let circuit_path = match self.circuit_names.get(&circuit_name_uni) {
            Some(name) => self.circuit_path(&name),
            None => {
                return Err(format!("No such circuit: {:?}", circuit_name))
            }
        };
        debug_log!(
            "Deleting circuit {:?} at {:?}",
            circuit_name,
            circuit_path
        );
        fs::remove_file(&circuit_path).map_err(|err| {
            format!(
                "Could not delete circuit file {:?}: {}",
                circuit_path, err
            )
        })?;
        self.circuit_names.remove(&circuit_name_uni);
        Ok(())
    }

    pub fn rename_circuit(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), String> {
        let old_name_uni = UniCase::new(old_name.to_string());
        let old_path = match self.circuit_names.get(&old_name_uni) {
            Some(name) => self.circuit_path(&name),
            None => return Err(format!("No such circuit: {:?}", old_name)),
        };
        if new_name == old_name {
            return Ok(());
        }
        if !is_valid_circuit_name(new_name) {
            return Err(format!("Invalid circuit name: {:?}", new_name));
        }
        let new_name_uni = UniCase::new(new_name.to_string());
        let new_path = self.circuit_path(&new_name);
        if new_name_uni != old_name_uni {
            if self.circuit_names.contains(&new_name_uni) {
                return Err(format!("Circuit already exists: {:?}", new_name));
            }
            // We already know there's not another circuit with this name, so
            // there shouldn't be a file at the new path, but in case there is
            // somehow, we do an extra check here to avoid clobbering it.
            // However, we omit this safety check if the two names differ only
            // by case, because otherwise a case-insensitive filesystem would
            // report that the new path already exists.
            if new_path.exists() {
                return Err(format!("Path already exists: {:?}", new_path));
            }
        }
        debug_log!("Moving circuit from {:?} to {:?}", old_path, new_path);
        fs::rename(&old_path, &new_path).map_err(|err| {
            format!(
                "Could not move circuit file {:?} to {:?}: {}",
                old_path, new_path, err
            )
        })?;
        self.circuit_names.remove(&old_name_uni);
        self.circuit_names.insert(new_name_uni);
        Ok(())
    }

    fn circuit_path(&self, circuit_name: &str) -> PathBuf {
        self.base_path.join(encode_name(circuit_name)).with_extension("toml")
    }
}

//===========================================================================//

pub struct CircuitNamesIter<'a> {
    inner: Option<btree_set::Iter<'a, UniCase<String>>>,
}

impl<'a> CircuitNamesIter<'a> {
    fn new(names: &'a BTreeSet<UniCase<String>>) -> CircuitNamesIter<'a> {
        CircuitNamesIter { inner: Some(names.iter()) }
    }

    pub(super) fn empty() -> CircuitNamesIter<'static> {
        CircuitNamesIter { inner: None }
    }
}

impl<'a> Iterator for CircuitNamesIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if let Some(ref mut inner) = self.inner {
            inner.next().map(UniCase::as_ref)
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for CircuitNamesIter<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        if let Some(ref mut inner) = self.inner {
            inner.next_back().map(UniCase::as_ref)
        } else {
            None
        }
    }
}

//===========================================================================//
