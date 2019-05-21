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

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::usize;

//===========================================================================//

#[allow(dead_code)]
pub enum Prereq {
    Complete(Conversation),
    All(&'static [Prereq]),
    Any(&'static [Prereq]),
    Choice(Conversation,
           &'static str,
           &'static str,
           &'static Prereq,
           &'static Prereq),
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Conversation {
    WakeUp,
    RestorePower,
}

const ALL_CONVERSATIONS: &[Conversation] =
    &[Conversation::WakeUp, Conversation::RestorePower];

impl Conversation {
    /// Returns the first conversation in the game, which is always unlocked.
    pub fn first() -> Conversation { Conversation::WakeUp }

    /// Returns an iterator over all conversations.
    pub fn all() -> AllConversationsIter { AllConversationsIter { index: 0 } }

    pub fn title(self) -> &'static str {
        match self {
            Conversation::WakeUp => "Wakeup Call",
            Conversation::RestorePower => "Restoring Power",
        }
    }

    pub fn prereq(self) -> &'static Prereq {
        match self {
            Conversation::WakeUp => &Prereq::All(&[]),
            Conversation::RestorePower => {
                &Prereq::Complete(Conversation::WakeUp)
            }
        }
    }
}

//===========================================================================//

#[derive(Default, Deserialize, Serialize)]
struct ConversationProgressData {
    complete: Option<bool>,
    progress: Option<usize>,
    choices: Option<BTreeMap<String, String>>,
}

impl ConversationProgressData {
    fn try_load(path: &Path) -> io::Result<ConversationProgressData> {
        toml::from_slice(&fs::read(path)?).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("{}", err))
        })
    }

    fn serialize_toml(&self) -> Result<Vec<u8>, String> {
        toml::to_vec(self).map_err(|err| {
            format!("Could not serialize conversation progress: {}", err)
        })
    }
}

//===========================================================================//

pub struct ConversationProgress {
    path: PathBuf,
    data: ConversationProgressData,
    needs_save: bool,
}

impl ConversationProgress {
    pub fn new(path: PathBuf) -> ConversationProgress {
        ConversationProgress {
            path,
            data: ConversationProgressData::default(),
            needs_save: true,
        }
    }

    pub fn create_or_load(path: &Path)
                          -> Result<ConversationProgress, String> {
        let mut needs_save = false;
        let data = if path.exists() {
            match ConversationProgressData::try_load(&path) {
                Ok(data) => data,
                Err(err) => {
                    debug_log!("Could not read conversation progress \
                                file from {:?}: {}",
                               path,
                               err);
                    ConversationProgressData::default()
                }
            }
        } else {
            needs_save = true;
            ConversationProgressData::default()
        };

        let progress = ConversationProgress {
            path: path.to_path_buf(),
            data,
            needs_save,
        };
        Ok(progress)
    }

    pub fn save(&mut self) -> Result<(), String> {
        if self.needs_save {
            debug_log!("Saving conversation progress to {:?}", &self.path);
            let data_toml = self.data.serialize_toml()?;
            fs::write(&self.path, data_toml)
                .map_err(|err| {
                             format!("Could not write conversation progress \
                                      file to {:?}: {}",
                                     self.path,
                                     err)
                         })?;
            self.needs_save = false;
        }
        Ok(())
    }

    pub fn progress(&self) -> usize {
        if self.is_complete() {
            usize::MAX
        } else {
            self.data.progress.unwrap_or(0)
        }
    }

    pub fn increment_progress(&mut self) {
        if !self.is_complete() {
            if let Some(ref mut index) = self.data.progress {
                *index += 1;
            } else {
                self.data.progress = Some(1);
            }
            self.needs_save = true;
        }
    }

    pub fn reset_progress(&mut self) {
        if self.progress() > 0 {
            self.data.complete = None;
            self.data.progress = None;
            self.data.choices = None;
            self.needs_save = true;
        }
    }

    pub fn is_complete(&self) -> bool { self.data.complete.unwrap_or(false) }

    pub fn mark_complete(&mut self) {
        if !self.is_complete() {
            self.data.complete = Some(true);
            self.data.progress = None;
            self.needs_save = true;
        }
    }

    pub fn get_choice(&self, key: &str) -> Option<&str> {
        if let Some(ref choices) = self.data.choices {
            if let Some(choice) = choices.get(key) {
                Some(choice)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_choice(&mut self, key: String, value: String) {
        if self.data.choices.is_none() {
            self.data.choices = Some(BTreeMap::new());
        }
        self.data.choices.as_mut().unwrap().insert(key, value);
        self.needs_save = true;
    }
}

//===========================================================================//

pub struct AllConversationsIter {
    index: usize,
}

impl<'a> Iterator for AllConversationsIter {
    type Item = Conversation;

    fn next(&mut self) -> Option<Conversation> {
        if self.index < ALL_CONVERSATIONS.len() {
            let conv = ALL_CONVERSATIONS[self.index];
            self.index += 1;
            Some(conv)
        } else {
            None
        }
    }
}

//===========================================================================//
