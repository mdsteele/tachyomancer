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

#[derive(
    Clone, Copy, Debug, EnumString, Eq, Hash, IntoStaticStr, PartialEq,
)]
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

    /// Returns the first chapter in the game, which is always unlocked.
    pub const fn first() -> Chapter {
        Chapter::Odyssey
    }

    pub fn order_with_orpheus_first(orpheus_first: bool) -> Vec<Chapter> {
        let mut chapters = Vec::<Chapter>::with_capacity(5);
        chapters.push(Chapter::Odyssey);
        chapters.push(Chapter::Planetfall);
        if orpheus_first {
            chapters.push(Chapter::Orpheus);
            chapters.push(Chapter::Calliope);
        } else {
            chapters.push(Chapter::Calliope);
            chapters.push(Chapter::Orpheus);
        }
        chapters.push(Chapter::Lorelei);
        chapters
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
    Prototyping,
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
    ScoutReport,
    AdditionalChips,
    MorePrototypes,
    MakingFuel,
    OneMoreThing,
    WeFoundSomething,
    ANewProblem,
    IncomingSignal,
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
    pub const fn first() -> Conversation {
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
            Conversation::Prototyping => "Prototyping",
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
            Conversation::ScoutReport => "Scout Report",
            Conversation::AdditionalChips => "Additional Chips",
            Conversation::MorePrototypes => "More Prototypes",
            Conversation::MakingFuel => "Making Fuel",
            Conversation::OneMoreThing => "One More Thing",
            Conversation::WeFoundSomething => "We Found Something",
            Conversation::ANewProblem => "A New Problem",
            Conversation::IncomingSignal => "Incoming Signal",
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
            | Conversation::Prototyping
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
            | Conversation::ScoutReport
            | Conversation::AdditionalChips
            | Conversation::MorePrototypes
            | Conversation::MakingFuel
            | Conversation::OneMoreThing
            | Conversation::WeFoundSomething
            | Conversation::ANewProblem
            | Conversation::IncomingSignal
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
            Conversation::Prototyping => {
                &Prereq::Complete(Conversation::RestorePower)
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
            // Planetfall:
            Conversation::AdvancedCircuits => {
                &Prereq::Complete(Conversation::Descent)
            }
            Conversation::ScoutReport
            | Conversation::AdditionalChips
            | Conversation::MorePrototypes
            | Conversation::MakingFuel
            | Conversation::OneMoreThing => {
                &Prereq::Complete(Conversation::AdvancedCircuits)
            }
            Conversation::WeFoundSomething => {
                &Prereq::Complete(Conversation::ScoutReport)
            }
            Conversation::ANewProblem => {
                &Prereq::Complete(Conversation::MakingFuel)
            }
            Conversation::IncomingSignal => &Prereq::All(&[
                Prereq::Complete(Conversation::ScoutReport),
                Prereq::Complete(Conversation::MakingFuel),
                Prereq::Complete(Conversation::OneMoreThing),
                Prereq::Complete(Conversation::WeFoundSomething),
                Prereq::Complete(Conversation::ANewProblem),
            ]),
            Conversation::UnexpectedCompany => {
                &Prereq::Complete(Conversation::IncomingSignal)
            }
            // Calliope:
            Conversation::Memory => {
                &Prereq::Complete(Conversation::UnexpectedCompany) // TODO
            }
            // Orpheus:
            Conversation::KeepingTime => {
                &Prereq::Complete(Conversation::UnexpectedCompany) // TODO
            }
            // Lorelei:
            Conversation::CatchingUp => {
                &Prereq::Complete(Conversation::UnexpectedCompany) // TODO
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

    pub fn set_progress(&mut self, index: usize) {
        if !self.is_complete() {
            self.data.progress = Some(index);
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

#[cfg(test)]
mod tests {
    use super::Chapter;

    #[test]
    fn first_chapter() {
        assert_eq!(
            Chapter::order_with_orpheus_first(false)[0],
            Chapter::first()
        );
        assert_eq!(
            Chapter::order_with_orpheus_first(true)[0],
            Chapter::first()
        );
    }
}

//===========================================================================//
