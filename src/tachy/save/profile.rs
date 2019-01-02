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
use super::converse::{Conversation, ConversationProgress};
use super::progress::{CircuitNamesIter, PuzzleProgress};
use super::puzzle::Puzzle;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

//===========================================================================//

const DATA_FILE_NAME: &str = "profile.toml";

//===========================================================================//

#[derive(Default, Deserialize, Serialize)]
struct ProfileData {
    conversation: Option<Conversation>,
    puzzle: Option<Puzzle>,
}

impl ProfileData {
    fn try_load(path: &Path) -> io::Result<ProfileData> {
        toml::from_slice(&fs::read(path)?).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("{}", err))
        })
    }
}

//===========================================================================//

pub struct Profile {
    name: String,
    base_path: PathBuf,
    data: ProfileData,
    needs_save: bool,
    conversations: HashMap<Conversation, ConversationProgress>,
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

        // Load conversation progress:
        let mut conversations =
            HashMap::<Conversation, ConversationProgress>::new();
        for conv in Conversation::all() {
            let conv_path = base_path.join(format!("{:?}.toml", conv));
            if !conv_path.exists() {
                continue;
            }
            let progress = ConversationProgress::create_or_load(&conv_path)?;
            conversations.insert(conv, progress);
        }

        // Load puzzle progress:
        let mut puzzles = HashMap::<Puzzle, PuzzleProgress>::new();
        for puzzle in Puzzle::all() {
            let puzzle_path = base_path.join(format!("{:?}", puzzle));
            if !puzzle_path.exists() {
                continue;
            }
            let progress = PuzzleProgress::create_or_load(&puzzle_path)?;
            puzzles.insert(puzzle, progress);
        }

        // Create profile:
        let mut profile = Profile {
            name,
            base_path: base_path.to_path_buf(),
            data,
            needs_save,
            conversations,
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
        for (_, progress) in self.puzzles.iter_mut() {
            progress.save()?;
        }
        for (_, progress) in self.conversations.iter_mut() {
            progress.save()?;
        }
        Ok(())
    }

    pub fn name(&self) -> &str { &self.name }

    pub fn current_conversation(&self) -> Conversation {
        self.data.conversation.unwrap_or(Conversation::first())
    }

    pub fn set_current_conversation(&mut self, conv: Conversation) {
        self.data.conversation = Some(conv);
        self.needs_save = true;
    }

    pub fn conversation_progress(&self, conv: Conversation) -> usize {
        self.conversations.get(&conv).map_or(0, ConversationProgress::progress)
    }

    pub fn is_conversation_unlocked(&self, conv: Conversation) -> bool {
        self.conversations.contains_key(&conv) || conv == Conversation::first()
    }

    pub fn is_conversation_complete(&self, conv: Conversation) -> bool {
        self.conversations
            .get(&conv)
            .map_or(false, ConversationProgress::is_complete)
    }

    pub fn set_conversation_choice(&mut self, conv: Conversation,
                                   key: String, value: String) {
        if !self.conversations.contains_key(&conv) {
            let path = self.base_path.join(format!("{:?}.toml", conv));
            self.conversations.insert(conv, ConversationProgress::new(path));
        }
        self.conversations.get_mut(&conv).unwrap().set_choice(key, value);
    }

    pub fn current_puzzle(&self) -> Puzzle {
        self.data.puzzle.unwrap_or(Puzzle::first())
    }

    pub fn set_current_puzzle(&mut self, puzzle: Puzzle) {
        self.data.puzzle = Some(puzzle);
        self.needs_save = true;
    }

    pub fn is_puzzle_unlocked(&self, puzzle: Puzzle) -> bool {
        self.puzzles.contains_key(&puzzle) || puzzle == Puzzle::first()
    }

    pub fn is_puzzle_solved(&self, puzzle: Puzzle) -> bool {
        self.puzzles.get(&puzzle).map_or(false, PuzzleProgress::is_solved)
    }

    pub fn puzzle_graph_points(&self, puzzle: Puzzle) -> &[(i32, i32)] {
        if let Some(ref progress) = self.puzzles.get(&puzzle) {
            progress.graph_points()
        } else {
            &[]
        }
    }

    pub fn first_circuit_name_for_current_puzzle(&self) -> Option<String> {
        self.circuit_names(self.current_puzzle()).next().map(str::to_string)
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
        if let Some(progress) = self.puzzles.get(&puzzle) {
            progress.load_circuit(circuit_name)
        } else {
            Err(format!("No such circuit: {:?}", circuit_name))
        }
    }

    pub fn save_circuit(&mut self, puzzle: Puzzle, circuit_name: &str,
                        circuit_data: &CircuitData)
                        -> Result<(), String> {
        if !self.puzzles.contains_key(&puzzle) {
            let puzzle_path = self.base_path.join(format!("{:?}", puzzle));
            let progress = PuzzleProgress::create_or_load(&puzzle_path)?;
            self.puzzles.insert(puzzle, progress);
        }
        let progress = self.puzzles.get_mut(&puzzle).unwrap();
        progress.save_circuit(circuit_name, circuit_data)
    }

    pub fn copy_circuit(&mut self, puzzle: Puzzle, old_name: &str,
                        new_name: &str)
                        -> Result<(), String> {
        if let Some(progress) = self.puzzles.get_mut(&puzzle) {
            progress.copy_circuit(old_name, new_name)
        } else {
            Err(format!("No such circuit: {:?}", old_name))
        }
    }

    pub fn delete_circuit(&mut self, puzzle: Puzzle, circuit_name: &str)
                          -> Result<(), String> {
        if let Some(progress) = self.puzzles.get_mut(&puzzle) {
            progress.delete_circuit(circuit_name)
        } else {
            Err(format!("No such circuit: {:?}", circuit_name))
        }
    }

    pub fn rename_circuit(&mut self, puzzle: Puzzle, old_name: &str,
                          new_name: &str)
                          -> Result<(), String> {
        if let Some(progress) = self.puzzles.get_mut(&puzzle) {
            progress.rename_circuit(old_name, new_name)
        } else {
            Err(format!("No such circuit: {:?}", old_name))
        }
    }
}

//===========================================================================//
