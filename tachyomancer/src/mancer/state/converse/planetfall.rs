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
pub(super) fn advanced_circuits(profile: &Profile,
                                builder: &mut ConversationBuilder)
                                -> Result<(), ()> {
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
        $=$#size = [3, 2]\n\
        [chips]\n\
        p1p0 = 'f0-Add'\n\
        [wires]\n\
        m1p0e = 'Stub'\n\
        p0p0e = 'Straight'\n\
        p0p0w = 'Straight'\n\
        p1p0e = 'Stub'\n\
        p1p0s = 'Stub'\n\
        p1p0w = 'Stub'\n\
        p1p1e = 'TurnRight'\n\
        p1p1n = 'TurnLeft'\n\
        p2p0s = 'TurnLeft'\n\
        p2p0w = 'TurnRight'\n\
        p2p1n = 'TurnRight'\n\
        p2p1w = 'TurnLeft'\n\
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
    builder.esra("Well done.");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn unexpected_company(profile: &Profile,
                                 builder: &mut ConversationBuilder)
                                 -> Result<(), ()> {
    builder.henry("Enemies approaching!");
    builder.puzzle(profile, Puzzle::CommandTurret)?;
    builder.henry("Which ship should we scan for first?");
    let chapter = builder
        .choice(profile, "chapter")
        .option("calliope", "\"Scan for the Calliope.\"")
        .option("orpheus", "\"Scan for the Orpheus.\"")
        .done()?;
    if chapter == "orpheus" {
        builder.henry("Okay, scanning for the Orpheus.");
    } else {
        builder.henry("Okay, scanning for the Calliope.");
    }
    Ok(())
}

//===========================================================================//
