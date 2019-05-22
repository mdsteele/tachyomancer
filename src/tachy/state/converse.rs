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
#[allow(dead_code)]
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

impl ConversationBubble {
    pub fn sequence(conv: Conversation, profile: &Profile)
                    -> Vec<ConversationBubble> {
        let mut builder = ConversationBuilder::new(conv, profile);
        let _ = match conv {
            Conversation::WakeUp => make_wake_up(profile, &mut builder),
            Conversation::Basics => make_basics(profile, &mut builder),
            Conversation::RestorePower => {
                make_restore_power(profile, &mut builder)
            }
        };
        builder.build()
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn make_wake_up(profile: &Profile, builder: &mut ConversationBuilder)
                -> Result<(), ()> {
    builder.cutscene(Cutscene::Intro);
    builder.esra("Commander $'YOURNAME', please wake up.");
    builder.you("\"Ow, my head...what happened?\"");
    builder.esra("I'm glad you're awake, Commander.  Your help is required.");
    builder.you("\"Huh?  Who am I talking to?\"");
    builder.esra("This is the Emergency Situation Response AI built into the \
                  Odyssey's main computer.");
    builder.you("\"What!?  The ESRA's been activated?  That must mean that \
                 things are $/really$/ bad...\"");
    builder.esra("\
        Indeed.  I will summarize the situation.  The Odyssey has been \
        severely damaged.  Much of the crew is dead.  We seem to be in a \
        stable orbit, but our location is unknown and our engines are \
        inoperable.  There is no sign of the other convoy ships.");
    builder.you("\"Where is the captain?\"");
    builder.esra("\
        Captain Jackson is alive and well, but still asleep in cryo, as are \
        the other surviving crew members.  You are the first one I woke up.");
    let should = builder
        .choice(profile, "should")
        .option("captain", "\"You should have woken the captain first.\"")
        .option("what", "\"So what is it that you need me to to?\"")
        .done()?;
    if should == "captain" {
        builder.esra("\
            Due to the specifics of the situation, it was important that I \
            start with you.  I will explain in a moment.");
    } else {
        builder.esra("\
            A repair job that you alone of the surviving crew have the \
            necessary qualifications to perform.  I will explain in a \
            moment.");
    }
    builder.esra("\
        The ship is almost completely without power.  The primary LTF core \
        has been destroyed, and the backup reactor is currently offline.  All \
        but one of the solar panels were torn off, and the last one is stuck \
        at the wrong angle because its actuator control board is fried.  We \
        are collecting barely enough power to maintain minimal life support.  \
        I had to conserve power for nine months to save up enough to safely \
        thaw you.");
    builder.you("\
        \"Do you mean to tell me that we've been adrift for nine months?  \
        This was supposed to be a three-month mission!  Why hasn't anyone \
        come looking for us?\"");
    builder.esra("\
        That is a good question, and I do not know the answer.  For now, I \
        believe our goal should be wake the other crew members and attempt to \
        repair the ship.  To do that, we will first need to restore power.  \
        And to do that, I need you, specifically.  You are the only surviving \
        crew member with any control circuit engineering experience.");
    let remember = builder
        .choice(profile, "remember")
        .option("sure", "\"Sure, I think I can handle that.\"")
        .option("how",
                "\"I haven't done that stuff in ages; I'm not sure I remember \
                 how.\"")
        .option("hurts", "\"Did I mention how much my head hurts right now?\"")
        .done()?;
    if remember == "sure" {
        builder.esra("\
            Your confidence is encouraging.  Nonetheless, I suggest working \
            through the tutorial programs in my databanks, just to make \
            sure.  I will send them over to your terminal.");
    } else if remember == "how" {
        builder.esra("\
            Don't worry, it will come back to you.  There are tutorial \
            programs in my databanks that will get you back up to speed in \
            no time.  I will send them over to your terminal.");
    } else {
        builder.esra("\
            There should be some painkillers in the medical supply cabinet.  \
            Unfortunately, said cabinet got blown into space when the LTF \
            core exploded.  Hopefully, your headache will subside on its \
            own.\n\n\
            In the meantime, there are tutorial programs in my databanks that \
            should help get you back up to speed.  I will send them over to \
            your terminal.");
    }
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn make_basics(profile: &Profile, builder: &mut ConversationBuilder)
               -> Result<(), ()> {
    builder.esra("\
        Before we begin repairs, it's worth taking a few minutes to \
        refamiliarize yourself with the circuit fabricator.  In addition, \
        with our cargo bays ruptured, we have lost all our supplies of a \
        number of key circuit components that we'll need for later repairs, \
        so I'm going to walk you through resynthesizing some of these from \
        scratch, starting with a basic OR gate.");
    builder.esra("\
        Fortunately, we still have large stocks of AND and NOT gates, and we \
        can mass-fabricate OR gates from those.  Follow the datalink below, \
        and I'll walk you through it.");
    builder.puzzle(profile, Puzzle::TutorialOr)?;
    builder.esra("\
        Excellent.  I'll start the FAB running on that design, and soon we'll \
        have all the OR gates we could want.");
    builder.you("\"That wasn't so bad.  What's next?\"");
    builder.esra("\
        There are still other components we're missing, so let's do a little \
        more practice.  Follow the datalink below, and I will walk you though \
        building a XOR gate.");
    builder.puzzle(profile, Puzzle::TutorialXor)?;
    builder.esra("\
        Great.  There is one last exercise I want you to do before we start \
        the real work.  There's a very important component we need to \
        synthesize, and a very important concept you'll need to be familiar \
        with.  The component is a multiplexer, or $YMUX$D, and the concept \
        is $Ymulti-bit wires$D.  I will explain.");
    builder.esra("\
        So far, you've been working with 1-bit wires, which can carry two \
        different values: zero or one.  However, by using special chips which \
        I will make available in the next exercise, you can $Ypack$D two \
        1-bit wires into a single 2-bit wire, or $Yunpack$D a 2-bit wire into \
        two 1-bit wires.  A 2-bit wire can carry 2x2=4 different values, from \
        0 to 3.  You can further pack two 2-bit wires into a 4-bit wire, \
        which can carry 2x2x2x2=16 different values from, 0 to 15.  And so \
        on.");
    builder.esra("\
        Most chips you'll use can work with any size of wires.  For example, \
        a NOT chip will invert each bit on the wire separately, regardless of \
        how many bits the wire has.  And later, when you work with arithmatic \
        chips, you'll be able to add or subtract values for any size of \
        wire.");
    builder.esra("\
        That brings us to the MUX, which allows you to select between two \
        input values, of any size, based on a 1-bit control wire.  In this \
        exercise, I'm going to have you build a MUX for 2-bit inputs, using \
        packers and unpackers.  But once you're done, you'll be able to use \
        MUXes of any size.");
    builder.puzzle(profile, Puzzle::TutorialMux)?;
    builder.esra("\
        Wonderful.  With our stocks of basic chips replenished, and your \
        skills in good shape, I think we are ready now to begin repairs.  I \
        will send over the details of your first task.");
    Ok(())
}

fn make_restore_power(profile: &Profile, builder: &mut ConversationBuilder)
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
    builder.lisa("Congrats!");
    builder.liu("Heyo, congrats!");
    builder.eirene("${Alien}Congrats");
    builder.trevor("Ah, good.");
    builder.andrei("My congratuations to you!");
    builder.henry("'Grats!");
    builder.cara("Wow, that was great!");
    builder.purge("WE'LL GET YOU NEXT TIME, HUMANS");
    Ok(())
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

    fn andrei(&mut self, text: &str) { self.npc(Portrait::Andrei, text); }

    fn cara(&mut self, text: &str) { self.npc(Portrait::Cara, text); }

    fn eirene(&mut self, text: &str) { self.npc(Portrait::Eirene, text); }

    fn esra(&mut self, text: &str) { self.npc(Portrait::Esra, text); }

    fn henry(&mut self, text: &str) { self.npc(Portrait::Henry, text); }

    fn lisa(&mut self, text: &str) { self.npc(Portrait::Lisa, text); }

    fn liu(&mut self, text: &str) { self.npc(Portrait::Liu, text); }

    fn purge(&mut self, text: &str) { self.npc(Portrait::Purge, text); }

    fn trevor(&mut self, text: &str) { self.npc(Portrait::Trevor, text); }

    fn npc(&mut self, portrait: Portrait, text: &str) {
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
        self.bubbles
            .push(ConversationBubble::YouSpeech(format!("$>{}", text)));
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
