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

use std::mem;
use tachy::save::{Conversation, Profile, Puzzle};

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationPortrait {
    Esra,
}

//===========================================================================//

pub enum ConversationBubble {
    YouSpeech(String),
    YouChoice(String, Vec<(String, String)>),
    NpcSpeech(ConversationPortrait, String),
    Puzzle(Puzzle),
}

impl ConversationBubble {
    pub fn sequence(conv: Conversation, profile: &Profile)
                    -> Vec<ConversationBubble> {
        let mut builder = ConversationBuilder::new();
        let progress = profile.conversation_progress(conv);
        match conv {
            Conversation::WakeUp => {
                builder.esra("Hello, world!\n\nLorem ipsum dolor sit amet, \
                              consectetur adipiscing elit, sed do eiusmod \
                              tempor incididunt ut labore et dolore magna \
                              aliqua.  Ut enim ad minim veniam, quis \
                              nostrud exercitation ullamco laboris nisi ut \
                              aliquip ex ea commodo consequat.");
                if progress >= 1 {
                    builder
                        .choice("greeting")
                        .option("hello", "\"Hello to you, too.\"")
                        .option("confused", "\"Huh?  Where am I?\"")
                        .option("silent", "(Say nothing)");
                }
                if progress >= 2 {
                    builder.puzzle(Puzzle::TutorialOr);
                }
                if progress >= 3 {
                    builder.you("\"How's that look?\"");
                }
                if progress >= 4 {
                    builder.esra("Congrats!");
                }
            }
        }
        builder.build()
    }
}

//===========================================================================//

struct ConversationBuilder {
    bubbles: Vec<ConversationBubble>,
}

impl ConversationBuilder {
    fn new() -> ConversationBuilder {
        ConversationBuilder { bubbles: Vec::new() }
    }

    fn build(self) -> Vec<ConversationBubble> { self.bubbles }

    fn choice(&mut self, key: &str) -> ChoiceBuilder {
        ChoiceBuilder::new(self, key)
    }

    fn esra(&mut self, text: &str) {
        self.npc(ConversationPortrait::Esra, text);
    }

    fn npc(&mut self, portrait: ConversationPortrait, text: &str) {
        self.bubbles
            .push(ConversationBubble::NpcSpeech(portrait, text.to_string()));
    }

    fn puzzle(&mut self, puzzle: Puzzle) {
        self.bubbles.push(ConversationBubble::Puzzle(puzzle));
    }

    fn you(&mut self, text: &str) {
        self.bubbles.push(ConversationBubble::YouSpeech(text.to_string()));
    }
}

//===========================================================================//

struct ChoiceBuilder<'a> {
    conversation: &'a mut ConversationBuilder,
    key: String,
    choices: Vec<(String, String)>,
}

impl<'a> ChoiceBuilder<'a> {
    fn new(conversation: &'a mut ConversationBuilder, key: &str)
           -> ChoiceBuilder<'a> {
        ChoiceBuilder {
            conversation,
            key: key.to_string(),
            choices: Vec::new(),
        }
    }

    fn option(&mut self, value: &str, label: &str) -> &mut ChoiceBuilder<'a> {
        self.choices.push((value.to_string(), label.to_string()));
        self
    }
}

impl<'a> Drop for ChoiceBuilder<'a> {
    fn drop(&mut self) {
        debug_assert!(!self.choices.is_empty());
        let key = mem::replace(&mut self.key, String::new());
        let choices = mem::replace(&mut self.choices, Vec::new());
        let bubble = ConversationBubble::YouChoice(key, choices);
        self.conversation.bubbles.push(bubble);
    }
}

//===========================================================================//
