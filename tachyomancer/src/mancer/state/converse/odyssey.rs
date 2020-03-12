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
    builder.esra("I am glad you are awake, Commander.  Your help is \
                  required.");
    builder.you("\"Huh?  Who am I talking to?\"");
    builder.esra("This is the Emergency Situation Response AI built into the \
                  Odyssey's main computer.");
    builder.you("\"What!?  The ESRA's been activated?  What's going on?\"");
    builder.esra("\
        I will summarize the situation.  The Odyssey has been severely \
        damaged.  Much of the crew is dead.  We seem to be in a stable orbit, \
        but our location is unknown and our engines are inoperable.  There is \
        no sign of the other convoy ships.");
    builder.you("\"Where is the captain?\"");
    builder.esra("\
        Captain Jackson is alive and well in cryosleep, as are the other \
        surviving crew members.  You are the first one that I woke up.");
    let should = builder
        .choice(profile, "should")
        .option("captain", "\"You should have woken the captain first.\"")
        .option("what", "\"So what is it that you need me to do?\"")
        .done()?;
    if should == "captain" {
        builder.esra("\
            I understand your concerns, Commander, but as I will explain in \
            a moment, it was important that I start with you.  In an \
            emergency situation when all crewmembers are in cryosleep or \
            otherwise incapacitated, I do have the authority to make that \
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
        but one of the solar panels are missing or destroyed, and the last \
        one is stuck at the wrong angle because its actuator control board is \
        damaged.  We are collecting barely enough power to maintain minimal \
        life support.  I had to conserve power for nine months to save up \
        enough to safely thaw you.");
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
        .option("how",
                "\"I haven't done that stuff in ages; I'm not sure I remember \
                 how.\"")
        .option("hurts", "\"Did I mention how much my head hurts right now?\"")
        .done()?;
    if remember == "how" {
        builder.esra("\
            I am confident that you will manage.  There are tutorial programs \
            in my databanks that will help refresh your memeory.  I will send \
            them over to your terminal.");
    } else {
        builder.esra("\
            There should be some painkillers in the medical supply cabinet.  \
            Unfortunately, that cabinet was blown into space when the LTF \
            core exploded.  Hopefully, your headache will subside on its \
            own.\n\n\
            In the meantime, there are tutorial programs in my databanks that \
            will help get you back up to speed.  I will send them over to \
            your terminal.");
    }
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn basics(profile: &Profile, builder: &mut ConversationBuilder)
                     -> Result<(), ()> {
    builder.esra("\
        Before we begin repairs, it is worth taking a few minutes to \
        refamiliarize yourself with the Nanofabricator.  In addition, \
        with our cargo bays ruptured, we have lost our supplies of certain \
        key circuit components that we will need for later repairs.  \
        Therefore, I will walk you through re-fabricating some of these \
        components from scratch.  Let us begin with a basic $*OR$* gate.");
    builder.esra("\
        Fortunately, we still have large stocks of AND and NOT gates, and we \
        can mass-fabricate OR gates from those.  Follow the datalink below, \
        and I will walk you through it.");
    builder.puzzle(profile, Puzzle::TutorialOr)?;
    builder.esra("\
        Excellent.  I will start the Nanofabricator running on that design, \
        and soon we will have all the OR gates we could need.");
    builder.you("\"That wasn't so bad.  What's next?\"");
    builder.esra("\
        There are still other components we are missing, so let us do a \
        little more practice.  Follow the datalink below, and I will walk you \
        though building a simple $*Mux$*.");
    builder.puzzle(profile, Puzzle::TutorialMux)?;
    builder.esra("\
        Great.  There is one last exercise I want you to do before we start \
        the real work, which is to create an $*Add$* chip.  And there is an \
        important concept you will need to use in order to do so.");
    builder.esra("\
        So far, you have been working with 1-bit wires, which can carry two \
        different values: 0 or 1.  However, by a using a $*Pack$* chip, you \
        can join two 1-bit wires into a single 2-bit wire, which can carry \
        2x2=4 different values, from 0 to 3.\n\n\
        $=$#size = [7, 3]\n\
        [chips]\n\
        p0p0 = \"f0-DocBv(1, '1-bit')\"\n\
        p0p1 = \"f0-DocBv(1, '1-bit')\"\n\
        p2p1 = 'f0-Pack'\n\
        p3p2 = \"f0-DocBv(2, '2-bit')\"\n\
        p4p1 = 't0-Pack'\n\
        p6p1 = \"f0-DocBv(4, '4-bit')\"\n\
        [wires]\n\
        p0p0e = 'Stub'\n\
        p0p1e = 'Stub'\n\
        p1p0e = 'Straight'\n\
        p1p0w = 'Straight'\n\
        p1p1e = 'Straight'\n\
        p1p1w = 'Straight'\n\
        p2p0s = 'TurnLeft'\n\
        p2p0w = 'TurnRight'\n\
        p2p1e = 'Stub'\n\
        p2p1n = 'Stub'\n\
        p2p1w = 'Stub'\n\
        p3p1e = 'Straight'\n\
        p3p1w = 'Straight'\n\
        p3p2e = 'Stub'\n\
        p4p1e = 'Stub'\n\
        p4p1s = 'Stub'\n\
        p4p1w = 'Stub'\n\
        p4p2n = 'TurnRight'\n\
        p4p2w = 'TurnLeft'\n\
        p5p1e = 'Straight'\n\
        p5p1w = 'Straight'\n\
        p6p1w = 'Stub'\n\
        #$<\n\n\
        Similarly, can further join two 2-bit wires into a 4-bit wire, which \
        can carry 2x2x2x2=16 different values, from 0 to 15.  And so on.");
    builder.you("\"And I can also split those wires back up, yes?\"");
    builder.esra("\
        Correct.  You can use an $*Unpack$* chip to split a 4-bit wire back \
        into two 2-bit wires, and so on:\n\n\
        $=$#size = [5, 2]\n\
        [chips]\n\
        p0p1 = \"f0-DocBv(4, '4-bit')\"\n\
        p2p1 = 'f0-Unpack'\n\
        p4p0 = \"f0-DocBv(2, '2-bit')\"\n\
        p4p1 = \"f0-DocBv(2, '2-bit')\"\n\
        [wires]\n\
        p0p1e = 'Stub'\n\
        p1p1e = 'Straight'\n\
        p1p1w = 'Straight'\n\
        p2p0e = 'TurnLeft'\n\
        p2p0s = 'TurnRight'\n\
        p2p1e = 'Stub'\n\
        p2p1n = 'Stub'\n\
        p2p1w = 'Stub'\n\
        p3p0e = 'Straight'\n\
        p3p0w = 'Straight'\n\
        p3p1e = 'Straight'\n\
        p3p1w = 'Straight'\n\
        p4p0w = 'Stub'\n\
        p4p1w = 'Stub'\n\
        #$<");
    builder.esra("\
        Most chips you'll use can work with any size of wires.  For example, \
        a $*NOT$* chip will invert each bit on the wire separately, \
        regardless of how many bits the wire has.  A generic $*Add$* chip, \
        similarly, can add two values for any size of wire.");
    builder.esra("\
        Unfortunately, we lost our stocks of generic $*Add$* chips, but we do \
        have some specialized ones that only work on 2-bit wires.  In this \
        exercise, you will use these 2-bit adders to build a 4-bit adder.  \
        Generalizing from there will allow us to fabricate as many generic \
        $*Add$* chips as we need.");
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
        Now that you are back up to speed on circuit fabrication, the first \
        task we need to accomplish is restoring additional ship power.  That \
        will allow us to safely rouse additional crew members from cryosleep, \
        and to start bringing other ship systems back online.");
    builder.esra("\
        The LTF core is badly damaged, and we simply do not have the raw \
        materials available to repair it.  The backup reactor is probably \
        repairable, but not by a single person.  Therefore, my recommendation \
        is that you begin by repairing the heliostat controller so that we \
        can get a better power output from the remaining solar panel.");
    let start = builder
        .choice(profile, "start")
        .option("sgtm", "\"Sounds like a good plan.\"")
        .option("how-much", "\"How much power will we get?\"")
        .done()?;
    if start == "how-much" {
        builder.esra("\
            It will probably provide enough power to rouse one additional \
            crew member from cryosleep, and provide life support for the \
            both of you, but not much more than that.");
    } else {
        builder.esra("\
            It is a start, at least.  It should allow us to rouse another \
            crew member from cryosleep, and with the help of a second person, \
            more repairs should become feasible.");
    }
    builder.esra("\
        The heliostat positioning sensors are still working, and can \
        automatically calculate the optimal mirror position at any given \
        time.  However, the motor control board that actually moves the \
        mirror into that position is damaged, and we don't have a schematic \
        for it.  We need you to design a new one.  It should not be too \
        difficult, but as I have previously explained, you are the only \
        surviving member of the crew who can do this.");
    builder.esra("\
        I will upload the relevant specifications to your terminal.  Let me \
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
        but he is the best-qualified person to help you repair the backup \
        reactor.");
    builder.esra("\
        Practically speaking, my advice would be to start with Chief Walker, \
        so that we can get enough power restored to wake more crew members.  \
        However, you are the one in command here, and I know that you may \
        feel it more appropriate to start with the captain.\n\n\
        What is your decision, Commander?");
    let who = builder
        .choice(profile, "who")
        .option("lisa", "\"We should wake Captain Jackson first.\"")
        .option("henry", "\"We should wake Chief Walker first.\"")
        .done()?;
    if who == "henry" {
        builder.esra("\
            Acknowledged.  I will re-enable life support in his section of \
            the ship and start the process of thawing him out of cryo.  Once \
            he is awake, I will send you both a comm with my recommendations \
            for next repair steps.");
    } else {
        builder.esra("\
            Acknowledged, Commander.  In that case, I will re-enable life \
            support in the captain's quarters and start the process of \
            thawing her out of cryo.  Once she is awake, I will send you a \
            comm with my updated report on the situation.");
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
        fabricate that could help you with your other designs.");
    builder.esra("\
        I have taken the liberty of selecting a few possibilities that should \
        be within your current capabilities, although they may be a bit more \
        challenging than what you have already done.  You should consider \
        these tasks optional, but again, any that you can complete will \
        provide new components that you can use for future tasks.");
    builder.puzzles(profile, &[
        Puzzle::FabricateXor,
        Puzzle::FabricateHalve,
        Puzzle::FabricateMul,
    ])?;
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
        Hello to both of you.  I have just finished filling in Chief Walker \
        on the situation.  Thanks to the Commander's work, we have enough \
        solar power to sustain life support for the both of you, but no more \
        than that.  I have asked the Chief to look into repairing the backup \
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
    builder.you("\"Yes, make it so, Chief.\"");
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
    builder.you("\"One reactor control board, coming up.\"");
    builder.henry("\
        \"Thank you, Commander!  I'm having your design fabricated right \
        now.  I've already got the other repairs on the reactor done, so I'll \
        get the new board slotted in and then get 'er warmed up.\"");
    builder.esra("\
        Congratulations to both of you.  Once the backup reactor comes \
        online, I will begin thawing out all the rest of the surviving crew, \
        starting with Captain Jackson.  I will brief her on our situation, \
        and then set up comm with the three of you once she is ready.");
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
        $(500)O$()dd.  There do not seem to be any navsats in range.");
    builder.henry("\
        \"$/That$/ can't be right.\"");
    builder.you("\"No navsats at all?  Are you sure, ESRA?\"");
    builder.esra("\
        The sensors are working perfectly, but there is no carrier wave at \
        all.  That implies that the ship is well outside Joint Federation \
        space.");
    builder.henry("\
        \"Waaaiit a minute...we couldn't've travelled $/that$/ far off \
        course, could we?  I mean, I'm a pretty good judge of normal engine \
        wear and tear, and from looking at it I'd have guessed we've gone \
        less than half the distance of our planned route.  That's nowhere \
        near enough to take us outside the navsat net.\"");
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
    builder.lisa("\
        \"What a nightmare.  That cryo thaw hurt like a bugger, and now I \
        find out that my ship is in pieces, half of my crew is dead, and we \
        are $/all out of coffee$/.  Somebody, $/please$/ tell me you \
        have good news for me.\"");
    builder.you("\"Captain!  Glad to see you're awake.\"");
    builder.lisa("\
        \"It's good to see you too, $'YOURNAME'.  Sorry all this had to \
        happen on your very first mission with us.  But the ESRA tells me \
        that it's all thanks to you and Chief Walker that we've got power \
        restored.  Good work.  Somehow, I had a feeling you'd come in \
        handy.\"");
    builder.you("\"Glad I could help.  Wish it hadn't been necessary.\"");
    builder.lisa("\
        \"You and me both, Commander.  But we've got a lot of work ahead of \
        us, and a lot of good men and women still alive on this boat that \
        need us, so I hope you're ready to help more.\"");
    builder.esra("\
        Hello, Captain, Commander.  I am glad to see that you are both well.  \
        May I give my report?");
    builder.lisa("\"Please do.\"");
    builder.esra("\
        All surviving crew have now been roused from cryosleep, and we have \
        enough power to sustain basic ship operations.  The current crew \
        complement is yourself, Commander $'YOURNAME', Chief Walker, and \
        about two thirds of the crewmen from Mechanical.");
    builder.lisa("\
        \"...Okay.  I'm afraid we'll have to mourn the others later.  Our top \
        priority is to find the other convoy ships and make sure they're \
        safe.  Chief, how're the engines?\"");
    builder.henry("\
        \"The engines are fixed now, Captain, but all the fuel tanks are \
        ruptured.  We can fix 'em too, of course, but the hyperfuel is long \
        gone.  We could synthesize more, but only if we could mine the raw \
        materials for it.\"");
    builder.lisa("\
        \"Which means we need to get down to the planet surface.  ESRA, \
        that's an M-class planet down there, yes?\"");
    builder.esra("\
        Correct, Captain.  Unfortunately, the ship's lander craft was torn \
        off and lost at some point during the disaster.");
    builder.lisa("\
        \"Nope, not lost, just adrift.   I saw it fly by my window five \
        minutes ago.\"");
    builder.you("\"Come again?\"");
    builder.lisa("\
        \"It looks like it's orbiting the planet, just like the $/Odyssey$/.  \
        If we can grapple it as it goes by, we could use it to get to the \
        surface.\"");
    builder.henry("\
        \"I did check on the grapple launcher along with everything else, \
        Captain.  A lot of the coils are busted up, but I think we could make \
        do if we could rework the control circuit to compensate.\"");
    builder.lisa("\"Commander $'YOURNAME'?\"");
    builder.you("\"I'll see to it, Captain.\"");
    builder.puzzle(profile, Puzzle::AutomateGrapple)?;
    builder.henry("\"C'mon...c'mon...aaaaaand...got 'er!\"");
    builder.lisa("\
        \"Great work, both of you.  Let's get that lander reeled in and \
        patched up.\"");
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
        \"It's good to see you too, $'YOURNAME'.  Sorry all this had to \
        happen on your very first mission with us.  But the ESRA tells me \
        that it's all thanks to you that we've got power hardly at all.  Good \
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
        Hello, Captain, Commander.  I am glad to see that you are both well.  \
        May I give my report?");
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
        Unfortunately, Captain, I do not know.  Main sensors are offline, so \
        we are not able to get any locator signal from any navsats that might \
        be in range.  Moreover, the planet below does not appear to match \
        anything in my databanks.");
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
    builder.esra("\
        Here are the specifications for new sensor signal amplifiers, \
        Commander.  The captain reports that she is ready to install the \
        replacements as soon as you have them ready.");
    builder.puzzle(profile, Puzzle::AutomateSensors)?;
    builder.lisa("\"All right, we're live.  What've we got, ESRA?\"");
    builder.esra("\
        Searching...\n\n\
        $(500)O$()dd.  There do not seem to be any navsats in range.");
    builder.lisa("\
        \"Say what?  What, did we just overshoot the frontier entirely?  That \
        can't be right.\"");
    builder.esra("\
        I am not recognizing these star patterns at all.  I will need some \
        time to try to determine our position.");
    builder.lisa("\
        \"Ugh.  All right.  Keep working on it, and report as soon as you \
        have something.  We $/need$/ to know where we are, and what happened \
        to the rest of the convoy.\"");
    builder.esra("Acknowledged, Captain.  I will see what I can do.");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn an_idea(profile: &Profile, builder: &mut ConversationBuilder)
                      -> Result<(), ()> {
    builder.lisa("\"Commander.  I know where we can get more power.\"");
    builder.you("\"Yes, Captain?\"");
    builder.lisa("\
        \"The lander.  It has an independant power source.  Not nearly enough \
        to power the $/Odyssey$/, but enough to boost life support and thaw \
        out Chief Walker, so he can help us fix the backup reactor.\"");
    builder.esra("\
        Unfortunately, Captain, the lander was torn off the ship when the \
        disaster occurred.  Otherwise I would already have suggested using it \
        for auxiliary power.");
    builder.lisa("\
        \"Thank you, ESRA, but I already know that, because I saw it drift by \
        my window five minutes ago.\"");
    builder.you("\"Come again?\"");
    builder.lisa("\
        \"The lander is adrift, but it's not lost; it looks like it's \
        orbiting the planet, just like the $/Odyssey$/.  If we can grapple \
        it as it goes by, we could use its power source.\"");
    builder.esra_interrupted("\
        Unfortunately, Captain, the grapple launch coils are badly damaged.  \
        In addition, it would by inadvisable to attempt to grapple something \
        that large until the ship's engines have been properly-");
    builder.lisa("\
        \"$/Thank you$/, ESRA, but I don't care if it's inadvisable, because \
        it's the only chance we have right now.  And I already checked on the \
        grapple launcher before I messaged the Commander.  Enough of the \
        coils are still working that I think we can make do, if we redesign \
        the control circuit to compensate.  Commander $'YOURNAME'?\"");
    builder.you("\"I'll see to it, Captain.\"");
    builder.puzzle(profile, Puzzle::AutomateGrapple)?;
    builder.lisa("\"Steady...steady...got it!\"");
    builder.esra("Our orbit appears to still be stable.");
    builder.lisa("\
        \"Perfect.  ESRA, as soon as we get it reeled in and the power \
        hooked up, get Chief Walker thawed out so we can get that backup \
        reactor fixed.\"");
    builder.esra("Affirmative, Captain.");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn more_power(profile: &Profile, builder: &mut ConversationBuilder)
                         -> Result<(), ()> {
    builder.henry("\
        \"Oi, that still smarts.  You know, I've been in cryo dozens of \
        times, and I'm almost positive that coming out of it isn't supposed \
        to hurt this much.\"");
    builder.lisa("\
        \"Yes, well, unfortunately we've all been frozen for nine months \
        straight.  There's a reason that IFC regulations say we're not \
        supposed to go for more than six.\"");
    builder.henry("\
        \"Oh!  Good morning, Captain!  I'm not quite sure what's been going \
        on while I was asleep, but I do seem to have missed out on some kind \
        of delightfully horrifying catastrophe.\"");
    builder.lisa("\
        \"That you have, Chief, and we're badly in need of your talents right \
        now.  Has the ESRA filled you in already?\"");
    builder.henry("\
        \"Yes, ma'am, and I've already been in to take a look.  The LTF core \
        got smashed up pretty good, but the backup reactor seems salvagable.  \
        I've even still got parts here to fix most of it.  Problem is, the \
        regulator board got proper fried, and I don't know the first thing \
        about electronics.  Without a replacement, we're out of luck.\"");
    builder.lisa("\
        \"Commander $'YOURNAME' here should be able to create a new regulator \
        board, if you can handle the mechanical repairs.\"");
    builder.henry("\
        \"Ah, glad to hear it!  I'll send the relevant specs to you, \
        Commander.  It shouldn't be too bad; the reactor knows how much power \
        it's currently generating, and the ship knows how much power it \
        needs.  The circuit just needs to adjust the reactor control rods to \
        make those match.\"");
    builder.puzzle(profile, Puzzle::AutomateReactor)?;
    builder.esra("\
        Backup power has been restored.  I have roused the rest of the \
        surviving crew from cryosleep.");
    builder.lisa("\"Good work, everyone.  Who do we have left?\"");
    builder.esra("\
        The current crew complement is yourself, Commander $'YOURNAME', Chief \
        Walker, and about two thirds of the crewmen from Mechanical.");
    builder.lisa("\
        \"...Okay.  We'll have to mourn the others later.  Our top priority \
        is to find the other convoy ships and make sure they're safe.  Chief, \
        how're the engines?\"");
    builder.henry("\
        \"The engines are fixable, Captain, but all the fuel tanks are \
        ruptured, and the hyperfuel is long gone.  But we could probably \
        synthesize more, if we could take take the lander down to the planet \
        surface.\"");
    builder.lisa("\
        \"Then that's our plan.  Everyone, start making preparations.\"");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn sensor_results(profile: &Profile,
                             builder: &mut ConversationBuilder)
                             -> Result<(), ()> {
    builder.esra("\
        Captain.  Commander.  I have been able to determine our location.");
    builder.lisa("\
        \"Excellent.  Chief Walker--I want you here for this too.\"\n\n\
        \"Go ahead, ESRA.\"");
    builder.esra("\
        The reason why there are no navsats in sensor range is that the ship \
        is not simply in the wrong star system.\n\n\
        The ship is in the wrong galaxy.");
    builder.lisa("\"I'm sorry, what?\"");
    builder.esra("\
        We are no longer in the Milky Way galaxy.  It took some lengthy \
        analysis of the surrounding star patterns to confirm this.");
    builder.henry("\
        \"We're in another $/galaxy?$/  Is that...even possible?\"");
    builder.lisa("\
        \"No, it isn't.  ESRA, this is ridiculous.  The nearest thing you \
        could technically call another galaxy is tens of thousands of \
        lightyears from where we started, which is a couple orders of \
        magnitude farther than we could possibly have traveled in the time \
        we were underway.  And we weren't even heading in that direction.\"");
    builder.henry("\
        \"Could we have gone through a wormhole or something?\"");
    builder.lisa("\
        \"$/No.$/  Wormholes only exist in fiction, Chief.  They're \
        mathematically inconsistent with modern metarelativity.  I was an \
        astrophysicist before I became a captain, you know.\"");
    builder.esra("\
        While it it unclear how the ship came to be here, the sensor data is \
        unambiguous.");
    builder.you("\"Where do you think we are, then?\"");
    builder.esra("\
        Based on sensor analysis, we are currently in Messier 51, also known \
        as the Whirlpool galaxy.  Distance to Earth: approximately 23 million \
        lightyears.");
    builder.lisa("\"I'm sorry, $/what!?$/\"");
    builder.you("\
        \"And...how long would it take to get home if we started back now?\"");
    builder.esra("\
        If the $/Odyssey$/ could maintain cruising speed indefinitely, it \
        would reach Earth in approximately 6300 years.");
    builder.henry("\
        \"That's probably a $/bit$/ longer than the engines're good for.  And \
        seems like kind of a long time in cryo, yeah?\"\n\n\
        \"Er, wait, can humans even survive in cryo for that long?\"");
    builder.esra("They cannot.");
    builder.henry("\
        \"Well then.  I don't suppose we could speed things up a bit?\"");
    builder.lisa("\
        \"$/Sigh.$/  Again, no.  Metarelativity is what makes it possible to \
        travel faster than light, but it still places fundamental limits on \
        how fast objects of a given mass can travel.  If we actually are \
        where ESRA thinks we are--and I remain $/politely skeptical$/ of \
        that--then it's physically impossible to travel fast enough to reach \
        home within our lifetimes.");
    let then = builder
        .choice(profile, "then")
        .option("how", "\"So how did we get here?\"")
        .option("wizard", "\"I guess we'll have to take up wizardry.\"")
        .done()?;
    if then == "how" {
        builder.lisa("\
            \"I don't know, but we're not going to find out by sitting \
            around.  ESRA, do long-range sensors show any sign of the other \
            convoy ships?\"");
    } else {
        builder.lisa("\
            \"Well, I certainly don't intend to just sit around here.  ESRA, \
            do long-range sensors show any sign of the other convoy ships?\"");
    }
    builder.esra("Negative.");
    builder.lisa("\
        \"All right, then.  We need to look for them, which means we need to \
        synthesize more hyperfuel, which means we need to get down to the \
        planet surface and start mining.  Commander, you and the Chief are \
        going to head down there on the lander while I oversee repairs up \
        here.  Chief, do whatever it takes to get the lander prepped.\"");
    builder.henry("\"Aye, Captain!\"");
    Ok(())
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn descent(profile: &Profile, builder: &mut ConversationBuilder)
                      -> Result<(), ()> {
    builder.henry("\
        \"I've been patching up the lander, Captain.  It's pretty banged up, \
        but life support's fine and I've got the thrusters working again.\"");
    builder.lisa("\
        \"Good work, Chief.  How soon can you have it ready for descent?\"");
    builder.henry("\
        \"Well, there's one problem.  The instruments on it got all fouled \
        up, and we don't have replacements.  I don't think we can trust the \
        current autopilot at all.  So we'd be taking it down on manual.\"");
    builder.esra("That would be extremely inadvisable.");
    builder.lisa("\
        \"Yeah, this time I think the ESRA's right.  Manual descent on an \
        unfamiliar planet is very unlikely to go well, and we don't have any \
        spare landers if we crash this one.\"\n\n\
        \"Also, everyone aboard would probably die.\"");
    builder.you("\
        \"Maybe I could make a new circuit to automate the landing.  Or at \
        least, semi-automate it, to make a manual landing more feasible.\"");
    builder.lisa("\
        \"That might be our best chance.  See what you can do, and then have \
        the Chief install it once you're done.\"");
    builder.puzzle(profile, Puzzle::CommandLander)?;
    builder.henry("\"The lander's prepped and ready to go, Captain.\"");
    builder.lisa("\
        \"All right, here's the plan.  Commander, you'll be leading an away \
        team down to the surface to extract materials and start synthesizing \
        hyperfuel.  Take Chief Walker with you.  Chief, I want you to pick \
        two of your best from Mechanical to go with you and the Commander.\"");
    builder.henry("\
        \"Yes, ma'am.  Uh, for this job that'd probably be Crewman Jiménez \
        and Crewman Patel.\"");
    builder.lisa("\
        \"Good.  I'll stay with the ship and supervise the remaining repairs \
        up here.  We'll maintain radio contact as much as possible, but \
        you'll be on point while you're down there, Commander.  No pressure \
        or anything, but our survival and our chances of finding the rest of \
        the convoy are absolutely dependent on your success.\"");
    builder.you("\"Aye, Captain.\"");
    Ok(())
}

//===========================================================================//
