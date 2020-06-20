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

use super::types::ConversationBuilder;
use crate::mancer::save::Profile;
use tachy::save::Puzzle;

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn advanced_circuits(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.esra("\
        Congratulations on your safe arrival to the planet surface, \
        Commander.  As the other members of the away team are still setting \
        up the temporary surface base, this seems like an ideal time for you \
        to review how to use $Cevent wires$D in your circuits, since you will \
        most likely be needing them soon.");
    builder.esra("\
        Until now, you have only been using $Obehavior wires$D, which always \
        carry some value, even if that value is zero.  By contrast, $Cevent \
        wires$D carry occasional, momentary pulses.  When no event pulse is \
        occuring, there is no value on the wire at all.");
    builder.esra("\
        There are several new chips you'll need to master in order to use \
        events.  Let's start with the $*Demux$* chip, which receives an \
        event, and sends it to one of two places, depending on the value of \
        its 1-bit control wire, like so:\n\
        $=$#size = [5, 3]\n\
        [chips]\n\
        p0p1 = \"t0-Comment('In')\"\n\
        p2p0 = \"t1-Comment('Ctrl')\"\n\
        p2p1 = 'f0-Demux'\n\
        p4p1 = \"t2-Comment('Out0')\"\n\
        p4p2 = \"t2-Comment('Out1')\"\n\
        [wires]\n\
        p0p1e = 'Stub'\n\
        p1p1e = 'Straight'\n\
        p1p1w = 'Straight'\n\
        p2p0s = 'Stub'\n\
        p2p1e = 'Stub'\n\
        p2p1n = 'Stub'\n\
        p2p1s = 'Stub'\n\
        p2p1w = 'Stub'\n\
        p2p2e = 'TurnRight'\n\
        p2p2n = 'TurnLeft'\n\
        p3p1e = 'Straight'\n\
        p3p1w = 'Straight'\n\
        p3p2e = 'Straight'\n\
        p3p2w = 'Straight'\n\
        p4p1w = 'Stub'\n\
        p4p2w = 'Stub'\n\
        #");
    builder.esra("\
        Follow the datalink below to find a simple exercise in using these \
        new chips.");
    builder.puzzle(profile, Puzzle::TutorialDemux)?;
    builder.esra("\
        Good job.  You may have noticed that the event wires in that exercise \
        were 0-bit wires, so they do not really have a value; either there is \
        an event on the wire, or there is not.  However, events can also \
        carry any size of value that behavior wires can: 1-bit, 2-bit, 4-bit, \
        or more.  The next few event chips I will introduce allow you to work \
        with those values.");
    builder.esra("\
        $=$#size = [6, 2]\n\
        [chips]\n\
        p1p0 = 'f0-Sample'\n\
        p1p1 = 'f3-Const(12)'\n\
        p3p0 = 'f0-Latest'\n\
        p4p1 = 'f0-Discard'\n\
        [wires]\n\
        m1p0e = 'Stub'\n\
        p0p0e = 'Straight'\n\
        p0p0w = 'Straight'\n\
        p1p0e = 'Stub'\n\
        p1p0s = 'Stub'\n\
        p1p0w = 'Stub'\n\
        p1p1n = 'Stub'\n\
        p2p0e = 'SplitLeft'\n\
        p2p0s = 'SplitTee'\n\
        p2p0w = 'SplitRight'\n\
        p2p1e = 'TurnRight'\n\
        p2p1n = 'TurnLeft'\n\
        p3p0e = 'Stub'\n\
        p3p0w = 'Stub'\n\
        p3p1e = 'Straight'\n\
        p3p1w = 'Straight'\n\
        p4p0e = 'Straight'\n\
        p4p0w = 'Straight'\n\
        p4p1e = 'Stub'\n\
        p4p1w = 'Stub'\n\
        p5p0e = 'Straight'\n\
        p5p0w = 'Straight'\n\
        p5p1e = 'Straight'\n\
        p5p1w = 'Straight'\n\
        p6p0w = 'Stub'\n\
        p6p1w = 'Stub'\n\
        #$<\n\
        First, a $*Sample$* chip attaches a value to a 0-bit event by \
        sampling the value of the behavior wire at the moment that the event \
        passes through.  Second, a $*Latest$* chip extracts the value from an \
        event; its output is equal to the value of the most recent event to \
        arrive, or zero if none have arrived yet.  Finally, a $*Discard$* \
        chip removes an event's value, turning it back into a 0-bit event.");
    builder.esra("\
        Using these chips, you can can extract the value from an event, \
        perform calculations on it using behaviors, and then reattatch the \
        new value to the event pulse.  Follow the datalink below and we will \
        work through another exercise.");
    builder.puzzle(profile, Puzzle::TutorialAmp)?;
    builder.esra("\
        Perfect.  There are two final event chips to discuss, which will \
        allow you to use events to their full potential in your circuits.");
    builder.esra("\
        Until now, you have been unable to put $Rloops$D in your circuits; \
        for example, you cannot use the output of an adder chip as one of \
        its inputs, because the result would not be well-defined:\n\
        $=$#size = [5, 2]\n\
        [chips]\n\
        p0p0 = 'f0-Const(5)'\n\
        p2p0 = 'f0-Add'\n\
        p4p0 = \"f0-DocBv(4, '???')\"\n\
        [wires]\n\
        p0p0e = 'Stub'\n\
        p1p0e = 'Straight'\n\
        p1p0w = 'Straight'\n\
        p2p0e = 'Stub'\n\
        p2p0s = 'Stub'\n\
        p2p0w = 'Stub'\n\
        p2p1e = 'TurnRight'\n\
        p2p1n = 'TurnLeft'\n\
        p3p0e = 'SplitLeft'\n\
        p3p0s = 'SplitTee'\n\
        p3p0w = 'SplitRight'\n\
        p3p1n = 'TurnRight'\n\
        p3p1w = 'TurnLeft'\n\
        p4p0w = 'Stub'\n\
        #");
    builder.esra("\
        However, in event-based circuits, time steps can be divided into \
        multiple $Ccycles$D, and the $*Delay$* chip allows an event to be \
        delayed for one cycle.  This controlled delay allows your circuit to \
        contain well-defined loops:\n\
        $=$#size = [3, 2]\n\
        [chips]\n\
        p1p0 = 'f0-Delay'\n\
        [wires]\n\
        p0p0e = 'TurnLeft'\n\
        p0p0s = 'TurnRight'\n\
        p0p1e = 'TurnRight'\n\
        p0p1n = 'TurnLeft'\n\
        p1p0e = 'Stub'\n\
        p1p0w = 'Stub'\n\
        p1p1e = 'Straight'\n\
        p1p1w = 'Straight'\n\
        p2p0s = 'TurnLeft'\n\
        p2p0w = 'TurnRight'\n\
        p2p1n = 'TurnRight'\n\
        p2p1w = 'TurnLeft'\n\
        #");
    builder.esra("\
        Of course, a loop like the one above would have no way to get \
        started.  A $*Join$* chip, which can merge two event streams into \
        one, is the final piece needed for looping circuits:\n\
        $=$#size = [3, 2]\n\
        [chips]\n\
        p0p0 = 'f0-Join'\n\
        p1p1 = 'f2-Delay'\n\
        p2p0 = 'f0-Demux'\n\
        [wires]\n\
        m1p0e = 'Stub'\n\
        p0p0e = 'Stub'\n\
        p0p0s = 'Stub'\n\
        p0p0w = 'Stub'\n\
        p0p1e = 'TurnRight'\n\
        p0p1n = 'TurnLeft'\n\
        p1p0e = 'Straight'\n\
        p1p0w = 'Straight'\n\
        p1p1e = 'Stub'\n\
        p1p1w = 'Stub'\n\
        p2m1s = 'Stub'\n\
        p2p0e = 'Stub'\n\
        p2p0n = 'Stub'\n\
        p2p0s = 'Stub'\n\
        p2p0w = 'Stub'\n\
        p2p1n = 'TurnRight'\n\
        p2p1w = 'TurnLeft'\n\
        p3p0w = 'Stub'\n\
        #");
    builder.esra("\
        That was a lot to absorb, so I suggest you try one final exercise to \
        practice using these concepts.");
    builder.puzzle(profile, Puzzle::TutorialSum)?;
    builder.henry("\
        \"Commander!  The base is all set up, and I think we're ready to get \
        started on mining, on your orders.\"");
    builder.you("\"Thank you, Chief.  Please proceed.\"");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn scout_report(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.henry("\
        \"Well, Commander, I'm happy to report that the first batch of raw \
        materials is coming in.  It's slow going though, and it'll take a \
        while to collect as much as we need.\"");
    builder.you("\"Anything we can do to make things more efficient?\"");
    builder.henry("\
        \"Actually, Commander, there might be.  See, we've got a handful of \
        basic scout robots, and Crewman Jiménez has been setting them up to \
        travel outwards from the base and radio back to us when they find \
        mineral deposits.  Then we go and and dig 'em up.\"");
    builder.henry("\
        \"Anyway, I was thinking we should get the robots to do the digging \
        for us.  She and I could get them fitted with the right attachments, \
        but it's not really what they're programmed for.  But we could swap \
        in new logic boards easy, if you were to design one.\"");
    builder.you("\"I'll take a look and let you know.\"");
    builder.puzzle(profile, Puzzle::AutomateMiningRobot)?;
    builder.henry("\
        \"Thanks, Commander, these'll be perfect.  We'll get those robots \
        retrofitted right away.\"");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn additional_chips(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.esra("\
        Now that you've mastered event-based circuits, Commander, there are \
        some additional chips you could fabricate that might be useful in \
        your future designs.  I will send the relevant specifications, in \
        case you choose to do that.");
    builder.puzzles(profile, &[
        Puzzle::FabricateInc,
        Puzzle::FabricateLatch,
        Puzzle::FabricateCounter,
    ])?;
    builder.esra("\
        Well done, Commander.  Those are all the chip specifications I have \
        available for now, but I will let you know if any others become \
        available in the future.");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn more_prototypes(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.esra("\
        Event-based circuits can quickly become complex, so you may find \
        yourself wanting to do additional prototyping and testing.  The \
        datalink below will provide an area for experimenting with these \
        circuits.");
    builder.puzzle(profile, Puzzle::SandboxEvent)?;
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn making_fuel(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.you("\"How is progress on fuel-making, Chief?\"");
    builder.henry("\
        \"We've got a good start.  Crewman Patel's got the catalysis \
        equipment set up, and he'll be managing the synthesis process as we \
        bring in materials.  Reagents get pumped in as we refine them, and \
        then when we've got enough for a batch, we pump them into the mixing \
        chamber to be catalyzed.  Simple enough.\"");
    builder.you("\"Huh.  I didn't realize hyperfuel was so easy to make.\"");
    builder.henry("\
        \"Oh sure, easy to make.  Easy to blow yourself up trying, too.\"");
    builder.you("\"Wait.  Really?\"");
    builder.henry("\
        \"Haha, yep, if he ever gets the valves messed up, the whole base'll \
        be up in smoke.  Or maybe in pieces.\"\n\n\
        \"...Oh, uh, he'll be fine though, Commander.  He's done this sort of \
        thing before.\"");
    let valves = builder
        .choice(profile, "valves")
        .option("auto",
                "\"No no no no no.  Sorry, Chief, but this is not good plan.  \
                 I'd feel a lot better if I could get the valves automated.\"")
        .option("fine",
                "\"This seems totally safe and fine, and I have no \
                 objections.\"")
        .done()?;
    if valves == "fine" {
        builder.henry("\
            \"Well...now that I think about it, Commander, it'd probably help \
            our throughput if we could get the valves automated.  Plus, that \
            would let me assign Crewman Patel some other tasks.\"");
    } else {
        builder.henry("\
            \"Oh.  Well, I guess that would let me assign him some other \
            tasks.  That would be great, actually.\"");
    }
    builder.puzzle(profile, Puzzle::AutomateFuelSynthesis)?;
    if valves == "fine" {
        builder.henry("\
            \"Thank you, Commander, this is much better.   Now our throughput \
            is up, $/and$/ we probably won't explode the base!\"");
    } else {
        builder.henry("\
            \"You were right, Commander, this is much better.   Now our \
            throughput is up, $/and$/ we probably won't explode the base!\"");
    }
    builder.you("\"I'm glad we could get that sorted out, Chief.\"");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn one_more_thing(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.lisa("\
        \"Commander, do you read me?  How are things going down there?\"");
    builder.you("\
        \"Going smoothly so far, Captain.  We've started extracting \
        materials, and the Chief is overseeing fuel synthesis work.\"");
    builder.lisa("\
        \"Good.  Listen, there's something else you need to do while you're \
        down there.  I've been trying to raise the other convoy ships on \
        subspace, but I haven't picked up any signals yet.  So I need your \
        team to construct a wide-array subspace beacon on the planet \
        surface, to give us a better chance of finding them.\"");
    builder.you("\"How's that work?\"");
    builder.lisa("\
        \"It should be straightforward.  Have the Nanofabricator manufacture \
        a bunch of subspace reflector panels, and lay them out in an array in \
        a flat area, over the space or a square kilometer or so.  Obviously, \
        that's a much wider array then we could ever fit on the ship, so it \
        should give us a much better chance of establishing a signal, if any \
        of the missing ships are in a nearby star system.\"");
    builder.you("\"Aye, Captain.  We'll get to work on it.\"");
    builder.lisa("\
        \"The only trick is that you'll need to automate the array to track \
        any signals we find.  Have the Chief and the two Crewmen get started \
        on the panels, and meanwhile I'll send you the specs you'll need for \
        making the control circuit.  Jackson out.\"");
    builder.puzzle(profile, Puzzle::AutomateBeacon)?;
    builder.henry("\"We've got the array set up, Commander.\"");
    builder.you("\
        \"Good work.  Go ahead and install this controller board, and have it \
        start scanning.  Report to me as soon we find anything.\"");
    builder.henry("\"Aye, Commander, will do.\"");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn we_found_something(
    _profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.henry("\
        \"Sorry to bother you, Commander, but Crewman Jiménez just discovered \
        something, and I thought you'd want to hear it right away.\"");
    builder.you("\"What is it, Chief?\"");
    builder.henry("\
        \"Well, she's been managing all the scouting and mining robots we've \
        been sending out from the base, and one of them came back with some \
        unexpected readings, so she went out to check it out herself.\"");
    builder.henry("\
        \"Turns out, there are some $/ruins$/ about 40 klicks north of the \
        base.  Someone used to live around here; a lot of someones, in fact.  \
        And judging from her report, I'd hazard that whoever it was was at a \
        comparible level of technological development as ourselves.  This was \
        probably one of their colony planets.\"");
    builder.you("\"Do you think any of them are still around?\"");
    builder.henry("\
        \"I doubt it.  Active civilization at that level of development?  We \
        would've seen signs of it from orbit.  City lights at night, radio \
        transmissions, that sort of thing.\"");
    builder.henry("\
        \"The thing is, though, these ruins...Jiménez showed me the \
        phototypes she took of the site, and, well, I've served in campaigns \
        before, Commander.  From the look of it, that city was pretty clearly \
        bombed.  $/From orbit.$/  And not even all that long ago--maybe a few \
        months to a year before our convoy left spacedock?");
    builder.you("\
        \"So, we may have just stumbled into the middle of an interplanetary \
        war.\"");
    builder.henry("\
        \"Kinda puts a terrifying new spin on our situation, doesn't it?\"");
    builder.you("\
        \"I'll forward this information to Captain Jackson.  In the meantime, \
        let's get that hyperfuel finished, on the double.\"");
    builder.henry("\"Aye, Commander.\"");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn a_new_problem(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.henry("\
        \"Do you have a minute, Commander?  We've got a slightly embarrassing \
        problem with the fuel synthesis, and I was hoping you could help.\"");
    builder.you("\"What's the trouble?\"");
    builder.henry("\
        \"It's kind of silly, actually.  The fuel processing tanks have to be \
        kept below a certain pressure so they don't rupture.  Normally they \
        stay well below that threshold, but because of atomospheric \
        conditions on this planet...well, I'll spare you the details, but \
        anyway, the pressure keeps getting a little too high.\"");
    builder.you("\"Aren't there safety valves, to relieve the pressure?\"");
    builder.henry("\
        \"Of course!  And normally they'd do their job, and this would all be \
        a non-issue.  But the darn things keep getting jammed.  Something \
        about the composition of the dust on this planet, getting into the \
        bearings.  So anyway, I've been having Crewman Patel stand by the \
        tanks and manually unjam the valves every so often, so that we don't \
        have to keep pausing the synthesis process.  Pretty standard \
        percussive maintenance--just give 'em a good whack, basically.\"");
    builder.you("\"Okay...\"");
    builder.henry("\
        \"But we don't really have hands to spare right now, and I need him \
        working on other things!  So I want to detatch the external \
        manipulator arm from the lander and set it up next to the tanks to \
        unstick the valves automatically.  It's a ridiculous kludge, but I \
        think it might actually be the quickest solution at this point.\"");
    builder.you("\
        \"So we're going to use a highly-sophisticated spaceflight robotic \
        arm to...whack some valves periodically?\"");
    builder.henry("\
        \"That was my thinking, yes, Commander.  But we would need you to \
        make a new control circuit for it to work.  It's not my place to ask, \
        but...\"");
    builder.you("\"Don't worry about it Chief.  I'll see what I can do.\"");
    builder.puzzle(profile, Puzzle::AutomateRobotArm)?;
    builder.henry("\
        \"Thanks so much Commander, you're a life-saver.  We should have all \
        the fuel we need in no time.\"");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn incoming_signal(
    _profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.henry("\
        \"Commander, I think we're getting a signal on the subspace beacon!  \
        A bunch of them, actually.\"");
    builder.you("\"Is it the other convoy ships?\"");
    builder.henry("\
        \"I think so?  Two of the signals do look like they have IFC carrier \
        waves, but they're really faint.  Could take hours to establish a \
        proper lock.\"");
    builder.you("\"Are the identifier codes coming through?\"");
    builder.henry("\
        \"Let me see...yes.  This first one looks like the $/H.L.S. \
        Calliope$/.  That's the ship that Lt. Cara Powart had acting command \
        of for this mission.  Er, I know you just recently joined us, \
        Commander, so I guess you might not know the other officers well \
        yet.  Lt. Powart is our medical officier, but she's also a \
        xenolinguist.  It's a shame she's not here, because she might be \
        able to make more sense of those ruins we found.\"");
    builder.you("\"And the second one?\"");
    builder.henry("\
        \"Second one...looks like the $/H.L.S. Orpheus$/.  Lt. Cmdr. Andrei \
        Sholokhov had that one.  He's our communications officer, and if he \
        were here, he would be doing a much better job isolating these \
        signals than I can.\"\n\n\
        \"Unfortunately, I can't find any sign of the last convoy ship.\"");
    builder.you("\
        \"I'm sure you're doing fine, Chief.  What about the other subspace \
        signals you were seeing?\"");
    builder.henry("\
        \"They could be random noise, but they could also be alien vessels, \
        presumably native to this galaxy.  And, uh, there's definitely a \
        chance that they're going to notice our beacon transmissions.\"");
    builder.you("\"We'll have to take that risk.\"");
    builder.henry("\
        \"Understood, Commander.  I'll keep trying to get a better fix on the \
        convoy ships, see if I can determine their locations.\"");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn unexpected_company(
    profile: &Profile,
    builder: &mut ConversationBuilder,
) -> Result<(), ()> {
    builder.esra("\
        DANGER, DANGER.  Commander, please come in.  Orbital radar is \
        detecting a large number of ground objects moving towards your base \
        from all directions, and long range scans are detecting an unknown \
        vessel on course towards this planet.");
    builder.lisa("\"What's going on down there?\"");
    builder.henry("\
        \"Captain?  Commander?  We're getting an incoming subspace \
        transmission!\"");
    builder.lisa("\"From one of the convoy ships?\"");
    builder.henry("\"Er, I don't think so...\"");
    builder.purge("WE ARE THE PURGE.  YOU DO NOT BELONG.");
    builder.you("\"You speak our language?\"");
    builder.purge("\
        WE HAVE ALREADY ANALYZED YOUR LANGUAGE AND SPECIES.  YOU DO NOT \
        BELONG.");
    builder.you("\"Looks like our beacon got some unwanted attention.\"");
    builder.lisa("\
        \"Greetings, Purge.  My name is Captain Lisa Jackson of the Human \
        vessel $/H.L.S. Odyssey$/.  Please accept my regrets for our \
        intrusion.  We mean you no harm.\"");
    builder.purge("UNACCEPTABLE.  YOU DO NOT BELONG.  YOU MUST BE DESTROYED.");
    builder.henry("\"Well, they seem like nice fellas.\"");
    builder.you("\"Shouldn't we be preparting to fight back?\"");
    builder.lisa("\
        \"Only if we have to.  Look, we may not be here by choice, but if \
        we're trespassing in their space, I can't blame them for not wanting \
        us here.  And if we've stepped into the middle of their war, I \
        $/definitely$/ don't want to get involved.\"");
    builder.lisa("\
        \"Purge, there is no need for conflict here.  We do not wish to \
        trespass.  We were brought here by accident, and we intend to depart \
        your territory, peacefully, as soon as possible.\"");
    builder.purge("\
        TERRITORY IS IRRELEVANT.  YOU DO NOT BELONG TO US, AND THEREFORE YOU \
        MUST BE DESTROYED, JUST AS WE DESTROYED THE VERMIN THAT INHABITED \
        THIS STAR SYSTEM.");
    builder.lisa("\"Beg pardon?\"");
    builder.henry("\"Like I said: nice fellas.\"");
    builder.purge_interrupted("\
        $/ALL$/ THAT ARE NOT OF THE PURGE MUST BE ERADIC-");
    builder.lisa("\
        \"Yeah, no.  End transmission.  Okay, new plan: screw these losers.  \
        Chief, what've you got down there for guns?\"");
    builder.henry("\
        \"Uh, the lander's pulse cannon still works.  I could rig it up on a \
        rotating turret, but we'd need to improvise a fire control system.\"");
    builder.lisa("\
        \"Do it.  Commander, slap together a control board.  We don't have \
        any orbital weapons up here to speak of, so you're going to have to \
        hold off their ground forces on your own until the beacon can get a \
        lock on the other convoy ships.  Then get in the lander and get back \
        up here so we can all get the heck out of this system before that \
        Purge ship arrives.\"");
    builder.puzzle(profile, Puzzle::CommandTurret)?;
    builder.henry("\
        \"Commander!  The beacon's almost got a lock, but I don't think we \
        can hold out for long enough to get a fix on both ships.  Which one \
        should we scan for?\"");
    let chapter = builder
        .choice(profile, "chapter")
        .option("calliope",
                "\"Scan for the $/Calliope$/ first.  We'll need Lt. Powart's \
                 expertise for dealing with these aliens.\"")
        .option("orpheus",
                "\"Scan for the $/Orpheus$/ first.  We'll need Lt. Cmdr. \
                 Sholokhov's help finding the other convoy ships.\"")
        .done()?;
    if chapter == "orpheus" {
        builder.henry("\"Aye aye, Commander, scanning for the $/Orpheus$/.\"");
    } else {
        builder.henry("\
            \"Aye aye, Commander, scanning for the $/Calliope$/.\"");
    }
    builder.you("\
        \"Tell the Crewmen to finish packing, and then get to the lander as \
        soon as you've got a location fix.  I'll keep the Purge off your back \
        until you do.\"");
    Ok(())
}

//===========================================================================//
