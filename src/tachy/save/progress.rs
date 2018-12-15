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

use super::circuit::CircuitData;
use super::encode::{decode_name, encode_name};
use std::collections::{BTreeSet, btree_set};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

//===========================================================================//

// Note: this file name needs to have a period (or other special character) in
// the non-extension part to ensure that it cannot conflict with any encoded
// circuit name.
const DATA_FILE_NAME: &str = "puzzle.progress.toml";

//===========================================================================//

#[derive(Default, Deserialize, Serialize)]
pub struct PuzzleProgressData {
    graph: Option<Vec<(i32, i32)>>,
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
    circuit_names: BTreeSet<String>,
    needs_save: bool,
}

impl PuzzleProgress {
    pub fn create_or_load(base_path: &Path) -> Result<PuzzleProgress, String> {
        // Create directory if needed:
        if !base_path.exists() {
            debug_log!("Creating puzzle directory at {:?}", base_path);
            fs::create_dir_all(&base_path)
                .map_err(|err| {
                    format!("Could not create puzzle directory at {:?}: {}",
                            base_path,
                            err)
                })?;
        }

        // Load progress data:
        let mut needs_save = false;
        let data_path = base_path.join(DATA_FILE_NAME);
        let data = if data_path.exists() {
            match PuzzleProgressData::try_load(&data_path) {
                Ok(data) => data,
                Err(err) => {
                    debug_log!("Could not read puzzle progress \
                                data file from {:?}: {}",
                               data_path,
                               err);
                    PuzzleProgressData::default()
                }
            }
        } else {
            needs_save = true;
            PuzzleProgressData::default()
        };

        // Get circuit names:
        let mut circuit_names = BTreeSet::<String>::new();
        let entries = base_path
            .read_dir()
            .map_err(|err| {
                format!("Could not read contents of puzzle directory {:?}: {}",
                        base_path,
                        err)
            })?;
        for entry_result in entries {
            let entry = entry_result
                .map_err(|err| {
                    format!("Error while reading contents of \
                             puzzle directory {:?}: {}",
                            base_path,
                            err)
                })?;
            let entry_path = entry.path();
            if entry_path.extension() != Some("toml".as_ref()) ||
                entry_path.file_name() == Some(DATA_FILE_NAME.as_ref())
            {
                continue;
            }
            if let Some(encoded) = entry_path.file_stem() {
                let circuit_name = decode_name(encoded);
                if !circuit_name.is_empty() {
                    circuit_names.insert(circuit_name);
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
            fs::write(&data_path, data_toml)
                .map_err(|err| {
                             format!("Could not write puzzle progress \
                                      data file to {:?}: {}",
                                     data_path,
                                     err)
                         })?;
            self.needs_save = false;
        }
        Ok(())
    }

    pub fn is_solved(&self) -> bool { !self.graph_points().is_empty() }

    pub fn graph_points(&self) -> &[(i32, i32)] {
        if let Some(ref points) = self.data.graph {
            points.as_slice()
        } else {
            &[]
        }
    }

    pub fn circuit_names(&self) -> CircuitNamesIter {
        CircuitNamesIter::new(&self.circuit_names)
    }

    pub fn has_circuit_name(&self, name: &str) -> bool {
        self.circuit_names.contains(name)
    }

    pub fn save_circuit(&mut self, circuit_name: &str,
                        circuit_data: &CircuitData)
                        -> Result<(), String> {
        let circuit_path = self.base_path
            .join(encode_name(circuit_name))
            .with_extension("toml");
        debug_log!("Saving circuit {:?} to {:?}", circuit_name, circuit_path);
        circuit_data.save(&circuit_path)?;
        self.circuit_names.insert(circuit_name.to_string());
        Ok(())
    }
}

//===========================================================================//

pub struct CircuitNamesIter<'a> {
    inner: Option<btree_set::Iter<'a, String>>,
}

impl<'a> CircuitNamesIter<'a> {
    fn new(names: &'a BTreeSet<String>) -> CircuitNamesIter<'a> {
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
            inner.next().map(String::as_str)
        } else {
            None
        }
    }
}

//===========================================================================//
