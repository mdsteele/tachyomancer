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
    builder.esra("Time to learn about event wires.\n\n\
                  You can use them in a circuit like this:\n\
                  $=$#size=[3,1]\n\
                  [chips]\n\
                  p1p0='t0-Filter'\n\
                  [wires]\n\
                  m1p0e='Stub'\n\
                  p0p0w='Straight'\n\
                  p0p0e='Straight'\n\
                  p1p0w='Stub'\n\
                  p1p0e='Stub'\n\
                  p2p0w='Straight'\n\
                  p2p0e='Straight'\n\
                  p3p0w='Stub'\n\
                  #");
    builder.puzzle(profile, Puzzle::TutorialDemux)?;
    builder.esra("Good job.");
    Ok(())
}

//===========================================================================//

#[cfg_attr(rustfmt, rustfmt_skip)]
pub(super) fn unexpected_company(profile: &Profile,
                                 builder: &mut ConversationBuilder)
                                 -> Result<(), ()> {
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
