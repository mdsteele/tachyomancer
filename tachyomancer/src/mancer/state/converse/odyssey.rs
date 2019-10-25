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
use super::types::ConversationBuilder;
use crate::mancer::save::Profile;
use tachy::save::Puzzle;

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn wake_up(profile: &Profile, builder: &mut ConversationBuilder)
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
        .option("what", "\"So what is it that you need me to do?\"")
        .done()?;
    if should == "captain" {
        builder.esra("\
            I understand your concerns, Commander, but as I will explain in \
            a moment, it was important that I start with you.  In an \
            emergency situation when all crewmembers were in cryosleep or \
            otherwise incapacitated, I did have the authority to make that \
            call.");
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
pub(super) fn basics(profile: &Profile, builder: &mut ConversationBuilder)
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
    builder.puzzle(profile, Puzzle::TutorialMux)?;
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
    builder.puzzle(profile, Puzzle::TutorialAdd)?;
    builder.esra("\
        Wonderful.  With our stocks of basic chips replenished, and your \
        skills in good shape, I think we are ready now to begin repairs.  I \
        will send over the details of your first task.");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn restore_power(profile: &Profile,
                            builder: &mut ConversationBuilder)
                            -> Result<(), ()> {
    builder.esra("\
        Now that you've been trained on the basics of circuit design, the \
        first task we need to accomplish is restoring additional ship \
        power.  That will allow us to safely rouse additional crew members \
        from cryosleep, and to start bringing other ship systems back \
        online.");
    builder.esra("\
        The LTF core is badly damaged, and we simply don't have the raw \
        materials available to repair it.  The backup reactor is probably \
        repairable, but not without physical access to it, and because of the \
        damaged sections of the ship, you can't safely reach it from where \
        you are now.");
    builder.esra("\
        Therefore, my recommendation is that you begin by repairing the \
        heliostat controller so we can get proper output from the remaining \
        solar panel.  We won't get very much power from that, but it's a \
        start.");
    let start = builder
        .choice(profile, "start")
        .option("sgtm", "Sounds like a good plan.")
        .option("how-much", "How much power will we get?")
        .done()?;
    if start == "how-much" {
        builder.esra("\
            Probably enough to rouse one additional crew member from \
            cryosleep, and provide life support for the both of you, but not \
            much more than that.");
    } else {
        builder.esra("\
            As I said, it's a start.  It should allow us to rouse another \
            crew member from cryosleep, and with the help of a second person, \
            more repairs should become feasible.");
    }
    builder.esra("\
        The heliostat position sensors are still working, and can \
        automatically calculate the optimal mirror position at any given \
        time.  However, the motor control board that actually moves the \
        mirror into that position was destroyed, and we don't have a \
        schematic for it.  We need you to design a new one.  It should not \
        be too difficult for you, but as I have previously explained, you \
        are the only living member of the crew who can do this.");
    builder.esra("\
        I'll upload the relevant specifications to your terminal.  Let me \
        know when you have a working design, and then we can get it \
        installed.");
    builder.puzzle(profile, Puzzle::AutomateHeliostat)?;
    builder.esra("\
        Excellent work.  With that control board installed, our energy \
        collection is already orders of magnitude better, meager though it \
        still is.\n\n\
        And now, Commander, there is a very important decision that you must \
        make.");
    builder.you("\"What do you mean?\"");
    builder.esra("\
        We have enough power for now to awaken and provide life support for \
        one more crew member besides yourself.  The most obvious candidates \
        would be either Captain Lisa Jackson, or Chief Petty Officer Henry \
        Walker, who is head of Mechanical on this ship.  I am aware that this \
        is your first mission with this crew, so you may not know him well, \
        but he is the best-qualified person to help repair the backup \
        reactor.");
    builder.esra("\
        Practically speaking, my advice would be to start with Chief Walker, \
        so that we can get enough power restored to wake more crew members.  \
        However, you are the one in command here, and I know that you may \
        feel it more appropriate to start with the captain.\n\n\
        What is your decision, Commander?");
    let who = builder
        .choice(profile, "who")
        .option("lisa", "We should wake Captain Jackson first.")
        .option("henry", "We should wake Chief Walker first.")
        .done()?;
    if who == "henry" {
        builder.esra("\
            Acknowledged.  I will re-enable life support in his section of \
            the ship and start the process of thawing him out of cryo.  Once \
            he's up and about, I'll send you both a comm with my \
            recommendations for next repair steps.");
    } else {
        builder.esra("\
            Acknowledged, Commander.  In that case, I will re-enable life \
            support in the captain's quarters and start the process of \
            thawing her out of cryo.  Once she's up and about, I'll send you \
            a comm with my updated report on the situation.");
    }
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn more_components(profile: &Profile,
                              builder: &mut ConversationBuilder)
                              -> Result<(), ()> {
    builder.esra("\
        Our stocks of the most important circuit components have now been \
        restored.  However, there are still many other useful chips we could \
        fabricate that may help you with your future designs.");
    builder.esra("\
        I've taken the liberty of selecting a few possibilities that should \
        be within your current capabilities, although they may be a bit more \
        challenging than what you've already done.  You should consider these \
        tasks optional, but again, any that you can complete will provide new \
        parts that you can use for other tasks.");
    builder.puzzles(profile,
                    &[Puzzle::FabricateXor,
                      Puzzle::FabricateHalve,
                      Puzzle::FabricateMul])?;
    builder.esra("\
        Excellent work, Commander.  I will let you know in the future if \
        there are any other good opportunities for fabricating useful \
        components.");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn step_two(_profile: &Profile, builder: &mut ConversationBuilder)
                       -> Result<(), ()> {
    builder.henry("\
        \"Oi, that still smarts.  You know, I've been in cryo dozens of \
        times, and I'm almost positive that coming out of it isn't supposed \
        to hurt this much.\"");
    builder.you("\
        \"That's because under standard regulations, we're not supposed to \
        stay under for $/nine months straight$/.\"");
    builder.henry("\
        \"Oh!  Good morning, Commander!  I'm not quite sure what's been going \
        on while I was asleep, but I do seem to have missed out on some kind \
        of delightfully horrifying catastrophe.\"");
    builder.esra("\
        Hello to both of you.  I've just finished filling in Chief Walker on \
        the situation.  Thanks to the Commander's work, we have enough solar \
        power to sustain life support for the both of you, but no more than \
        that.  I've asked the Chief to look into repairing the backup \
        reactor so we can generate more power.");
    builder.henry("\
        \"Yeah, I've been in to take a look.  The LTF core got smashed up \
        pretty good, but the backup reactor seems salvagable.  I've even \
        still got parts here to fix most of it.  Problem is, the regulator \
        board got proper fried, and I don't know the first thing about \
        electronics.  Without a replacement, we're out of luck.\"");
    builder.esra("\
        That was my assessment as well.  Fortunately, Commander $'YOURNAME' \
        has some former experience with circuit design, and should be able to \
        fabricate a new regulator board from scratch.");
    builder.henry("\
        \"Is that right, Commander?  Fantastic!  I do have the specs right \
        here, so I'll get them sent over to you.  And if it's all right, I'll \
        go ahead and start on the mechanical repairs so the reactor will be \
        all ready for the new board whenever it's done?\"");
    builder.you("Yes, make it so, Chief.");
    builder.henry("\
        \"Aye aye, Commander.  I'll send you those specs and then get started \
        right away.\"");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn reactor_specs(profile: &Profile,
                            builder: &mut ConversationBuilder)
                            -> Result<(), ()> {
    builder.henry("\
        \"I have the reactor specs you requested, Commander.  It shouldn't be \
        too hard.  Basically, the reactor knows how much power it's currently \
        generating, and the ship knows how much power it needs.  Your circuit \
        just needs to adjust the reactor control rods to make those match.\"");
    builder.henry("\
        \"You can make it a bit more efficient by keeping the control rods as \
        even with each other as possible, but honestly, I wouldn't worry \
        about it.  As long as the three control rod numbers add up to the \
        requested power level, it'll work fine.\"");
    builder.puzzle(profile, Puzzle::AutomateReactor)?;
    builder.you("One reactor control board, coming up.");
    builder.henry("\
        \"Thank you, Commander!  I'm having your design fabricated right \
        now.  I've already got the other repairs on the reactor done, so I'll \
        get the new board slotted in and then get 'er warmed up.\"");
    builder.esra("\
        Congratulations to both of you.  Once the backup reactor comes \
        online, I will begin thawing out all the rest of the surviving crew, \
        starting with Captain Jackson.  I will brief her on our situation, \
        and then set up comm with the three of you once she's ready.");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn where_are_we(profile: &Profile,
                           builder: &mut ConversationBuilder)
                           -> Result<(), ()> {
    builder.henry("\"Commander?  Could I have a quick word with you?\"");
    builder.you("\"What's on your mind, Chief?\"");
    builder.henry("\"Do you...happen to know where we are right now?\"");
    let location = builder
        .choice(profile, "where")
        .option("noidea", "\"Not really, no.\"")
        .option("notkansas", "\"Well, I'm pretty sure it's not Kansas.\"")
        .done()?;
    if location == "notkansas" {
        builder.henry("\
            \"Well, it's not New Ithaca either, which is where we were \
            supposed to be.  I don't recognize this planet we're orbiting, \
            and we're not picking up a navsat signal anywhere.\"");
    } else {
        builder.henry("\
            \"Me neither, Commander.  Our convoy was bound for New Ithaca, \
            and we're definitely not there.  I don't recognize this planet \
            we're orbiting, and we're not picking up a navsat signal \
            anywhere.\"");
    }
    builder.you("\"Hmm.  ESRA, can you get a fix on our location?\"");
    builder.esra("\
        Unfortunately, no.  There has been no navsat signal nor fleet comm \
        for the entire nine months that the ship has been orbiting this \
        planet.  However, the long-range sensors are damaged and may simply \
        not be picking up the signal.");
    builder.you("\"Can we fix them?\"");
    builder.esra("\
        Diagnostics seem to indicate that the exterior units are working.  \
        However, the internal signal amplifiers are burnt out and need \
        replacement.");
    builder.henry("\
        \"Those'd be easy for me to swap out, Commander.  But, uh, I think \
        the spares were stored in one of the sections of the ship that got \
        torn off.\"");
    builder.esra("\
        Specifications for new amplifiers are available, Commander.  It \
        should be possible for you to fabricate new ones.");
    builder.puzzle(profile, Puzzle::AutomateSensors)?;
    builder.henry("\
        \"Okay!  Old ones out, new ones in...easy as pie.  Sensors should be \
        good to go!\"");
    builder.esra("\
        Searching...\n\n\
        $(500)O$(30)dd.  There don't seem to be any navsats in range.");
    builder.henry("\
        \"$/That$/ can't be right.\"");
    builder.you("\"No navsats at all?  Are you sure, ESRA?\"");
    builder.esra("\
        The sensors are working perfectly, but there is no carrier wave at \
        all.  That implies that the ship is well outside Joint Federation \
        space.");
    builder.henry("\
        \"I'll be honest, Commander, I have trouble believing we could've \
        travelled that far off course.  I've been checking out the engines \
        pretty carefully, and based on the level of normal wear and tear I'm \
        seeing, I'd have guessed we've gone less than half the distance of \
        our planned route.  That's nowhere near enough to take us outside \
        the navsat net.\"");
    builder.you("\
        \"All right.  ESRA, keep scanning, and try to figure out where we \
        are.  Let me know as soon as you have something.\"");
    builder.esra("Affirmative.");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn captain_awake(profile: &Profile,
                            builder: &mut ConversationBuilder)
                            -> Result<(), ()> {
    builder.henry("TODO");
    builder.puzzle(profile, Puzzle::AutomateGrapple)?;
    builder.henry("TODO");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn captains_call(_profile: &Profile,
                            builder: &mut ConversationBuilder)
                            -> Result<(), ()> {
    builder.lisa("\
        \"What a nightmare.  That cryo thaw hurt like a bugger, and now I \
        find out that my ship is in pieces, half of my crew is dead, and we \
        are $/all out of coffee$/.  Somebody, $/please$/ tell me you \
        have good news for me.\"");
    builder.you("\"Captain!  Glad to see you're awake.\"");
    builder.lisa("\
        \"It's good to see you too, $'YOURNAME'.  Bad luck, all this \
        happening on your first mission with us.  But the ESRA tells me that \
        it's all thanks to you that we've got power hardly at all.  Good \
        work.  Somehow, I had a feeling you'd come in handy.\"");
    builder.you("\"Glad I could help.  Wish it hadn't been necessary.\"");
    builder.lisa("\
        \"You and me both, Commander.  But we've got a lot of work ahead of \
        us, and a lot of good men and women still alive on this boat that \
        need us, so I hope you're ready to help more.\"\n\n\
        \"Also, frankly, you probably should have woken up the Chief instead \
        of me, because I don't have the expertise to fix the reactor.  But I \
        appreciate that you did what you thought was best.  We'll figure this \
        out somehow.\"");
    builder.esra("\
        Hello, Captain, Commander.  Glad to see that you are both well.  May \
        I give my report?");
    builder.lisa("\"Please do.\"");
    builder.esra("\
        We have enough solar power right now to sustain life support for the \
        both of you, but no more.  And of course, we don't have enough power \
        to restart a cryo cycle for either of you, so we can't wake any more \
        crew until we get more power.");
    builder.lisa("\
        \"All right, for now, let's focus on what we $/can$/ fix.  In fact, \
        better question: where $/are$/ we, anyway?\"");
    builder.esra("\
        Unfortunately, Captain, I don't know.  Main sensors have been \
        knocked out, so we're not able to get any locator signal from any \
        navsats that might be in range.  We're practically blind.  And I \
        don't recognize the planet we're orbiting from my databanks.");
    builder.lisa("\
        \"We'll, it's certainly not New Ithaca.  Hope the colony's doing okay \
        without us, because we're already nine months late for the \
        rendezvous.\"\n\n\
        \"Sensors, then, let's start with those.  Can we fix them?\"");
    builder.esra("\
        I believe so.  As far as I can tell from diagnostics, the exterior \
        units are actually fine; it's just the signal amplifiers that need \
        replacement.  Captain, you could probably reach them from where you \
        are, if Commander $'YOURNAME' can fabricate some replacements to \
        swap in.");
    builder.you("\"I think I can handle that.\"");
    builder.lisa("\
        \"Great.  ESRA, send the Commander the relevant specs.  I'll open up \
        the access panel and start swapping out the old parts.\"");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn low_visibility(profile: &Profile,
                             builder: &mut ConversationBuilder)
                             -> Result<(), ()> {
    builder.lisa("TODO");
    builder.puzzle(profile, Puzzle::AutomateSensors)?;
    builder.lisa("\"All right, we're live.  What've we got, ESRA?\"");
    builder.esra("\
        Searching...\n\n\
        $(500)O$(30)dd.  There don't seem to be any navsats in range.");
    builder.lisa("\
        \"Say what?  What, did we just overshoot the frontier entirely?  That \
        can't be right.\"");
    builder.esra("\
        I'm not recognizing these star patterns at all.  I will need some \
        time to try to determine our position.");
    builder.lisa("\
        \"Ugh.  All right.  In the meantime...ESRA, now that main sensors are \
        working, we should be able to optmize the heliostat more accurately, \
        right?  Wouldn't that give our solar power generation a bit of an \
        efficiency boost?\"");
    builder.esra("Yes.  Some, but not much.");
    builder.lisa("\
        \"It'll have to be enough.  ESRA, I need you to get Chief Walker \
        awake.  Ration heat and oxygen for myself and the Commander if you \
        have to, just make it work.  We $/need$/ that reactor back online.\"");
    builder.esra("Acknowledged, Captain.  I will see what I can do.");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn an_idea(profile: &Profile, builder: &mut ConversationBuilder)
                      -> Result<(), ()> {
    builder.lisa("TODO");
    builder.puzzle(profile, Puzzle::AutomateGrapple)?;
    builder.lisa("TODO");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn more_power(profile: &Profile, builder: &mut ConversationBuilder)
                         -> Result<(), ()> {
    builder.lisa("TODO");
    builder.puzzle(profile, Puzzle::AutomateReactor)?;
    builder.lisa("TODO");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn sensor_results(_profile: &Profile,
                             builder: &mut ConversationBuilder)
                             -> Result<(), ()> {
    builder.esra("TODO");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn descent(profile: &Profile, builder: &mut ConversationBuilder)
                      -> Result<(), ()> {
    builder.esra("TODO");
    builder.puzzle(profile, Puzzle::CommandLander)?;
    builder.esra("TODO");
    Ok(())
}

//===========================================================================//
