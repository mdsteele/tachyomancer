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

use super::super::cutscene::Cutscene;
use tachy::save::{Conversation, Profile, Puzzle};

//===========================================================================//

// These enum values must be in alphabetical order, and must match the PNG
// files in the src/tachy/texture/portrait directory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Portrait {
    Andrei = 0,
    Cara,
    Eirene,
    Esra,
    Henry,
    Lisa,
    Liu,
    Purge,
    Trevor,
}

//===========================================================================//

pub enum ConversationBubble {
    YouSpeech(String),
    YouChoice(String, Vec<(String, String)>),
    NpcSpeech(Portrait, String),
    Cutscene(Cutscene),
    Puzzle(Puzzle),
}

//===========================================================================//

pub(super) struct ConversationBuilder {
    conv: Conversation,
    progress: usize,
    bubbles: Vec<ConversationBubble>,
}

impl ConversationBuilder {
    pub(super) fn new(conv: Conversation, profile: &Profile)
                      -> ConversationBuilder {
        ConversationBuilder {
            conv,
            progress: profile.conversation_progress(conv),
            bubbles: Vec::new(),
        }
    }

    pub(super) fn build(self) -> Vec<ConversationBubble> { self.bubbles }

    pub(super) fn choice<'a, 'b>(&'a mut self, profile: &'b Profile,
                                 key: &str)
                                 -> ChoiceBuilder<'a, 'b> {
        ChoiceBuilder::new(self, profile, key)
    }

    pub(super) fn cutscene(&mut self, cutscene: Cutscene) {
        self.bubbles.push(ConversationBubble::Cutscene(cutscene));
    }

    pub(super) fn andrei(&mut self, text: &str) {
        self.npc(Portrait::Andrei, text);
    }

    pub(super) fn cara(&mut self, text: &str) {
        self.npc(Portrait::Cara, text);
    }

    pub(super) fn eirene(&mut self, text: &str) {
        self.npc(Portrait::Eirene, text);
    }

    pub(super) fn esra(&mut self, text: &str) {
        self.npc(Portrait::Esra, text);
    }

    pub(super) fn henry(&mut self, text: &str) {
        self.npc(Portrait::Henry, text);
    }

    pub(super) fn lisa(&mut self, text: &str) {
        self.npc(Portrait::Lisa, text);
    }

    pub(super) fn liu(&mut self, text: &str) { self.npc(Portrait::Liu, text); }

    pub(super) fn purge(&mut self, text: &str) {
        self.npc(Portrait::Purge, text);
    }

    pub(super) fn trevor(&mut self, text: &str) {
        self.npc(Portrait::Trevor, text);
    }

    fn npc(&mut self, portrait: Portrait, text: &str) {
        self.bubbles
            .push(ConversationBubble::NpcSpeech(portrait, text.to_string()));
    }

    pub(super) fn puzzle(&mut self, profile: &Profile, puzzle: Puzzle)
                         -> Result<(), ()> {
        self.bubbles.push(ConversationBubble::Puzzle(puzzle));
        if profile.is_puzzle_solved(puzzle) {
            Ok(())
        } else {
            Err(())
        }
    }

    pub(super) fn you(&mut self, text: &str) {
        self.bubbles
            .push(ConversationBubble::YouSpeech(format!("$>{}", text)));
    }
}

//===========================================================================//

#[must_use = "must call done() on ChoiceBuilder"]
pub(super) struct ChoiceBuilder<'a, 'b> {
    conversation: &'a mut ConversationBuilder,
    profile: &'b Profile,
    key: String,
    choices: Vec<(String, String)>,
}

impl<'a, 'b> ChoiceBuilder<'a, 'b> {
    pub(super) fn new(conversation: &'a mut ConversationBuilder,
                      profile: &'b Profile, key: &str)
                      -> ChoiceBuilder<'a, 'b> {
        ChoiceBuilder {
            conversation,
            profile,
            key: key.to_string(),
            choices: Vec::new(),
        }
    }

    pub(super) fn option(mut self, value: &str, label: &str)
                         -> ChoiceBuilder<'a, 'b> {
        self.choices.push((value.to_string(), label.to_string()));
        self
    }

    pub(super) fn done(self) -> Result<String, ()> {
        debug_assert!(!self.choices.is_empty());
        let choice =
            self.profile
                .get_conversation_choice(self.conversation.conv, &self.key);
        if choice.is_some() ||
            self.conversation.progress > self.conversation.bubbles.len()
        {
            let (value, label) = self.get_choice(choice);
            let bubble = ConversationBubble::YouSpeech(label);
            self.conversation.bubbles.push(bubble);
            Ok(value)
        } else {
            let bubble = ConversationBubble::YouChoice(self.key, self.choices);
            self.conversation.bubbles.push(bubble);
            Err(())
        }
    }

    fn get_choice(&self, opt_choice: Option<&str>) -> (String, String) {
        if let Some(choice) = opt_choice {
            for &(ref value, ref label) in self.choices.iter() {
                if value == choice {
                    return (value.clone(), label.clone());
                }
            }
        }
        return self.choices[0].clone();
    }
}

//===========================================================================//