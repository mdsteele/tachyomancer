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
        let mut builder = ConversationBuilder::new(conv, profile);
        let _ = match conv {
            Conversation::WakeUp => {
                ConversationBubble::make_wake_up(profile, &mut builder)
            }
            Conversation::RestorePower => {
                ConversationBubble::make_restore_power(profile, &mut builder)
            }
        };
        builder.build()
    }

    fn make_wake_up(profile: &Profile, builder: &mut ConversationBuilder)
                    -> Result<(), ()> {
        builder.esra("Hello, world!\n\nLorem ipsum dolor sit amet, \
                      consectetur adipiscing elit, sed do eiusmod \
                      tempor incididunt ut labore et dolore magna \
                      aliqua.  Ut enim ad minim veniam, quis \
                      nostrud exercitation ullamco laboris nisi ut \
                      aliquip ex ea commodo consequat.");
        builder
            .choice(profile, "greeting")
            .option("hello", "\"Hello to you, too.\"")
            .option("confused", "\"Huh?  Where am I?\"")
            .option("silent", "(Say nothing)")
            .done()?;
        builder.puzzle(profile, Puzzle::TutorialOr)?;
        builder.you("\"How's that look?\"");
        builder.esra("Congrats!");
        Ok(())
    }

    fn make_restore_power(profile: &Profile,
                          builder: &mut ConversationBuilder)
                          -> Result<(), ()> {
        builder.esra("Now that you've been trained on the basics of circuit \
                      design, the first task we need to accomplish is \
                      restoring additional ship power.  That will allow us \
                      to safely rouse additional crew members from cryosleep, \
                      and to start bringing other ship systems back online.");
        builder.puzzle(profile, Puzzle::AutomateHeliostat)?;
        builder.esra("Excellent work.");
        Ok(())
    }
}

//===========================================================================//

struct ConversationBuilder {
    conv: Conversation,
    progress: usize,
    bubbles: Vec<ConversationBubble>,
}

impl ConversationBuilder {
    fn new(conv: Conversation, profile: &Profile) -> ConversationBuilder {
        ConversationBuilder {
            conv,
            progress: profile.conversation_progress(conv),
            bubbles: Vec::new(),
        }
    }

    fn build(self) -> Vec<ConversationBubble> { self.bubbles }

    fn choice<'a, 'b>(&'a mut self, profile: &'b Profile, key: &str)
                      -> ChoiceBuilder<'a, 'b> {
        ChoiceBuilder::new(self, profile, key)
    }

    fn esra(&mut self, text: &str) {
        self.npc(ConversationPortrait::Esra, text);
    }

    fn npc(&mut self, portrait: ConversationPortrait, text: &str) {
        self.bubbles
            .push(ConversationBubble::NpcSpeech(portrait, text.to_string()));
    }

    fn puzzle(&mut self, profile: &Profile, puzzle: Puzzle) -> Result<(), ()> {
        self.bubbles.push(ConversationBubble::Puzzle(puzzle));
        if profile.is_puzzle_solved(puzzle) {
            Ok(())
        } else {
            Err(())
        }
    }

    fn you(&mut self, text: &str) {
        self.bubbles.push(ConversationBubble::YouSpeech(text.to_string()));
    }
}

//===========================================================================//

#[must_use = "must call done() on ChoiceBuilder"]
struct ChoiceBuilder<'a, 'b> {
    conversation: &'a mut ConversationBuilder,
    profile: &'b Profile,
    key: String,
    choices: Vec<(String, String)>,
}

impl<'a, 'b> ChoiceBuilder<'a, 'b> {
    fn new(conversation: &'a mut ConversationBuilder, profile: &'b Profile,
           key: &str)
           -> ChoiceBuilder<'a, 'b> {
        ChoiceBuilder {
            conversation,
            profile,
            key: key.to_string(),
            choices: Vec::new(),
        }
    }

    fn option(mut self, value: &str, label: &str) -> ChoiceBuilder<'a, 'b> {
        self.choices.push((value.to_string(), label.to_string()));
        self
    }

    fn done(self) -> Result<(), ()> {
        debug_assert!(!self.choices.is_empty());
        let choice =
            self.profile
                .get_conversation_choice(self.conversation.conv, &self.key);
        if choice.is_some() ||
            self.conversation.progress > self.conversation.bubbles.len()
        {
            let bubble = ConversationBubble::YouSpeech(self.get_label(choice));
            self.conversation.bubbles.push(bubble);
            Ok(())
        } else {
            let bubble = ConversationBubble::YouChoice(self.key, self.choices);
            self.conversation.bubbles.push(bubble);
            Err(())
        }
    }

    fn get_label(&self, opt_choice: Option<&str>) -> String {
        if let Some(choice) = opt_choice {
            for &(ref value, ref label) in self.choices.iter() {
                if value == choice {
                    return label.clone();
                }
            }
        }
        return self.choices[0].1.clone();
    }
}

//===========================================================================//
