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
use strum::IntoEnumIterator;

//===========================================================================//

#[allow(dead_code)]
pub enum Prereq {
    Complete(Conversation),
    All(&'static [Prereq]),
    Any(&'static [Prereq]),
    Choice(
        Conversation,
        &'static str,    // key
        &'static str,    // value
        &'static Prereq, // then
        &'static Prereq, // else
    ),
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, IntoStaticStr, PartialEq)]
pub enum Chapter {
    Odyssey,
    Planetfall,
    Calliope,
    Orpheus,
    Lorelei,
}

impl Chapter {
    pub fn title(&self) -> &'static str {
        self.into()
    }
}

//===========================================================================//

#[derive(
    Clone, Copy, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Serialize,
)]
pub enum Conversation {
    // Odyssey start:
    WakeUp,
    Basics,
    RestorePower,
    MoreComponents,
    // Odyssey Henry track:
    StepTwo,
    ReactorSpecs,
    WhereAreWe,
    CaptainAwake,
    // Odyssey Lisa track:
    CaptainsCall,
    LowVisibility,
    AnIdea,
    MorePower,
    // Odyssey finish:
    SensorResults,
    Descent,
    // Planetfall:
    AdvancedCircuits,
    UnexpectedCompany,
    // Calliope:
    Memory,
    // Orpheus:
    KeepingTime,
    // Lorelei:
    CatchingUp,
}

impl Conversation {
    /// Returns the first conversation in the game, which is always unlocked.
    pub fn first() -> Conversation {
        Conversation::WakeUp
    }

    /// Returns an iterator over all conversations.
    pub fn all() -> ConversationIter {
        Conversation::iter()
    }

    pub fn title(self) -> &'static str {
        match self {
            // Odyssey start:
            Conversation::WakeUp => "Wakeup Call",
            Conversation::Basics => "Circuit Basics",
            Conversation::RestorePower => "Restoring Power",
            Conversation::MoreComponents => "More Components",
            // Odyssey Henry track:
            Conversation::StepTwo => "Step Two",
            Conversation::ReactorSpecs => "Reactor Specs",
            Conversation::WhereAreWe => "Where Are We?",
            Conversation::CaptainAwake => "Captain Awake",
            // Odyssey Lisa track:
            Conversation::CaptainsCall => "Captain's Call",
            Conversation::LowVisibility => "Low Visibility",
            Conversation::AnIdea => "An Idea",
            Conversation::MorePower => "More Power",
            // Odyssey finish:
            Conversation::SensorResults => "Sensor Results",
            Conversation::Descent => "Descent",
            // Planetfall:
            Conversation::AdvancedCircuits => "Advanced Circuits",
            Conversation::UnexpectedCompany => "Unexpected Company",
            // Calliope:
            Conversation::Memory => "Memory",
            // Orpheus:
            Conversation::KeepingTime => "Keeping Time",
            // Lorelei:
            Conversation::CatchingUp => "Catching Up",
        }
    }

    pub fn chapter(self) -> Chapter {
        match self {
            Conversation::WakeUp
            | Conversation::Basics
            | Conversation::RestorePower
            | Conversation::MoreComponents
            | Conversation::StepTwo
            | Conversation::ReactorSpecs
            | Conversation::WhereAreWe
            | Conversation::CaptainAwake
            | Conversation::CaptainsCall
            | Conversation::LowVisibility
            | Conversation::AnIdea
            | Conversation::MorePower
            | Conversation::SensorResults
            | Conversation::Descent => Chapter::Odyssey,
            Conversation::AdvancedCircuits
            | Conversation::UnexpectedCompany => Chapter::Planetfall,
            Conversation::Memory => Chapter::Calliope,
            Conversation::KeepingTime => Chapter::Orpheus,
            Conversation::CatchingUp => Chapter::Lorelei,
        }
    }

    pub fn prereq(self) -> &'static Prereq {
        match self {
            // Odyssey intro:
            Conversation::WakeUp => &Prereq::All(&[]),
            Conversation::Basics => &Prereq::Complete(Conversation::WakeUp),
            Conversation::RestorePower | Conversation::MoreComponents => {
                &Prereq::Complete(Conversation::Basics)
            }
            // Odyssey Henry track:
            Conversation::StepTwo => &Prereq::Choice(
                Conversation::RestorePower,
                "who",
                "henry",
                &Prereq::Complete(Conversation::RestorePower),
                &Prereq::Any(&[]),
            ),
            Conversation::ReactorSpecs | Conversation::WhereAreWe => {
                &Prereq::Complete(Conversation::StepTwo)
            }
            Conversation::CaptainAwake => {
                &Prereq::Complete(Conversation::ReactorSpecs)
            }
            // Odyssey Lisa track:
            Conversation::CaptainsCall => &Prereq::Choice(
                Conversation::RestorePower,
                "who",
                "henry",
                &Prereq::Any(&[]),
                &Prereq::Complete(Conversation::RestorePower),
            ),
            Conversation::LowVisibility | Conversation::AnIdea => {
                &Prereq::Complete(Conversation::CaptainsCall)
            }
            Conversation::MorePower => &Prereq::Complete(Conversation::AnIdea),
            // Odyssey finish:
            Conversation::SensorResults => &Prereq::Any(&[
                Prereq::All(&[
                    Prereq::Complete(Conversation::WhereAreWe),
                    Prereq::Complete(Conversation::CaptainAwake),
                ]),
                Prereq::All(&[
                    Prereq::Complete(Conversation::LowVisibility),
                    Prereq::Complete(Conversation::MorePower),
                ]),
            ]),
            Conversation::Descent => {
                &Prereq::Complete(Conversation::SensorResults)
            }
            _ => &Prereq::Complete(Conversation::Descent), // TODO
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

    pub fn create_or_load(
        path: &Path,
    ) -> Result<ConversationProgress, String> {
        let mut needs_save = false;
        let data = if path.exists() {
            match ConversationProgressData::try_load(&path) {
                Ok(data) => data,
                Err(err) => {
                    debug_log!(
                        "Could not read conversation progress \
                         file from {:?}: {}",
                        path,
                        err
                    );
                    ConversationProgressData::default()
                }
            }
        } else {
            needs_save = true;
            ConversationProgressData::default()
        };

        Ok(ConversationProgress { path: path.to_path_buf(), data, needs_save })
    }

    pub fn save(&mut self) -> Result<(), String> {
        if self.needs_save {
            debug_log!("Saving conversation progress to {:?}", &self.path);
            let data_toml = self.data.serialize_toml()?;
            fs::write(&self.path, data_toml).map_err(|err| {
                format!(
                    "Could not write conversation progress \
                     file to {:?}: {}",
                    self.path, err
                )
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

    pub fn is_complete(&self) -> bool {
        self.data.complete.unwrap_or(false)
    }

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
