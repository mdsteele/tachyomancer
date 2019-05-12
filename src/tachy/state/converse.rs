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

use super::cutscene::Cutscene;
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
    Cutscene(Cutscene),
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
        builder.cutscene(Cutscene::Intro);
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
        builder.esra("The LTF core is badly damaged, and we simply don't \
                      have the raw materials available to repair it.  The \
                      backup reactor is probably repairable, but not without \
                      physical access to it, and because of the damaged \
                      sections of the ship, you can't safely reach it from \
                      where you are now.");
        builder.esra("Therefore, my recommendation is that you begin by \
                      repairing the heliostat controller so we can get proper \
                      output from the few remaining solar panels.  We won't \
                      get very much power from that, but it's a start.");
        let choice = builder
            .choice(profile, "start")
            .option("sgtm", "Sounds like a good plan.")
            .option("how-much", "How much power will we get?")
            .done()?;
        if choice == "how-much" {
            builder.esra("Probably enough to rouse one additional crew \
                          member from cryosleep, and provide life support \
                          for the both of you, but not much more than that.");
        } else {
            builder.esra("As I said, it's a start.");
        }
        builder.esra("The heliostat position sensors are still working, and \
                      can automatically calculate the optimal mirror \
                      position at any given time.  However, the motor control \
                      board that actually moves the mirror into that position \
                      needs to be replaced.");
        builder.esra("I'll upload the relevant specifications to your \
                      terminal.  Let me know when you have a working design, \
                      and then we can get it installed.");
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

    fn cutscene(&mut self, cutscene: Cutscene) {
        self.bubbles.push(ConversationBubble::Cutscene(cutscene));
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

    fn done(self) -> Result<String, ()> {
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
