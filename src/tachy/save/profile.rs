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
use super::puzzle::Puzzle;
use std::collections::{BTreeSet, HashMap, btree_set};
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

struct PuzzleProgress {
    // TODO: store puzzle unsolved/solved and optimization graph
    circuit_names: BTreeSet<String>,
}

impl PuzzleProgress {
    fn new() -> PuzzleProgress {
        PuzzleProgress { circuit_names: BTreeSet::new() }
    }

    fn circuit_names(&self) -> CircuitNamesIter {
        CircuitNamesIter::new(&self.circuit_names)
    }

    fn has_circuit_name(&self, name: &str) -> bool {
        self.circuit_names.contains(name)
    }
}

//===========================================================================//

pub struct Profile {
    name: String,
    base_path: PathBuf,
    data: ProfileData,
    needs_save: bool,
    puzzles: HashMap<Puzzle, PuzzleProgress>,
}

impl Profile {
    pub fn create_or_load(name: String, base_path: &Path)
                          -> Result<Profile, String> {
        // Create directory if needed:
        if !base_path.exists() {
            debug_log!("Creating profile {:?} at {:?}", name, base_path);
            fs::create_dir_all(&base_path)
                .map_err(|err| {
                    format!("Could not create profile {:?} \
                             directory at {:?}: {}",
                            name,
                            base_path,
                            err)
                })?;
        } else {
            debug_log!("Loading profile {:?} from {:?}", name, base_path);
        }

        // Load profile data:
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

        // Load puzzle progress:
        let mut puzzles = HashMap::<Puzzle, PuzzleProgress>::new();
        for puzzle in Puzzle::all() {
            let puzzle_path = base_path.join(format!("{:?}", puzzle));
            if !puzzle_path.exists() {
                continue;
            }
            let mut progress = PuzzleProgress::new();
            let entries = puzzle_path
                .read_dir()
                .map_err(|err| {
                    format!(
                        "Could not read contents of profile \
                         {:?} puzzle {:?} directory: {}",
                        name,
                        puzzle,
                        err
                    )
                })?;
            for entry_result in entries {
                let entry = entry_result
                    .map_err(|err| {
                        format!("Error while reading contents of profile \
                                 {:?} puzzle {:?} directory: {}",
                                name,
                                puzzle,
                                err)
                    })?;
                let entry_path = entry.path();
                if entry_path.extension() != Some("toml".as_ref()) {
                    continue;
                }
                if let Some(encoded) = entry_path.file_stem() {
                    let circuit_name = decode_name(encoded);
                    if !circuit_name.is_empty() {
                        progress.circuit_names.insert(circuit_name);
                    }
                }
            }
            puzzles.insert(puzzle, progress);
        }

        // Create profile:
        let mut profile = Profile {
            name,
            base_path: base_path.to_path_buf(),
            data,
            needs_save,
            puzzles,
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

    pub fn circuit_names(&self, puzzle: Puzzle) -> CircuitNamesIter {
        if let Some(ref progress) = self.puzzles.get(&puzzle) {
            progress.circuit_names()
        } else {
            CircuitNamesIter::empty()
        }
    }

    pub fn has_circuit_name(&self, puzzle: Puzzle, name: &str) -> bool {
        if let Some(ref progress) = self.puzzles.get(&puzzle) {
            progress.has_circuit_name(name)
        } else {
            false
        }
    }

    pub fn load_circuit(&self, puzzle: Puzzle, circuit_name: &str)
                        -> Result<CircuitData, String> {
        if !self.has_circuit_name(puzzle, circuit_name) {
            return Err(format!("No such circuit: {:?}", circuit_name));
        }
        let puzzle_path = self.base_path.join(format!("{:?}", puzzle));
        let circuit_path =
            puzzle_path.join(encode_name(circuit_name)).with_extension("toml");
        debug_log!("Loading circuit {:?} from {:?}",
                   circuit_name,
                   circuit_path);
        CircuitData::load(&circuit_path)
    }

    pub fn save_circuit(&mut self, puzzle: Puzzle, circuit_name: &str,
                        data: &CircuitData)
                        -> Result<(), String> {
        let puzzle_path = self.base_path.join(format!("{:?}", puzzle));
        if !puzzle_path.exists() {
            debug_log!("Creating puzzle {:?} directory at {:?}",
                       puzzle,
                       puzzle_path);
            fs::create_dir_all(&puzzle_path)
                .map_err(|err| {
                    format!("Could not create puzzle directory at {:?}: {}",
                            puzzle_path,
                            err)
                })?;
        }
        let circuit_path =
            puzzle_path.join(encode_name(circuit_name)).with_extension("toml");
        debug_log!("Saving circuit {:?} to {:?}", circuit_name, circuit_path);
        data.save(&circuit_path)?;
        self.puzzles
            .entry(puzzle)
            .or_insert_with(PuzzleProgress::new)
            .circuit_names
            .insert(circuit_name.to_string());
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

    fn empty() -> CircuitNamesIter<'static> {
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
