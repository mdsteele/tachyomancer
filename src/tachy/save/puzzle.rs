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

use strum::IntoEnumIterator;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum PuzzleKind {
    Tutorial,
    Fabricate,
    Automate,
    Command,
    Sandbox,
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Deserialize, EnumIter, EnumString, Eq, Hash,
         Ord, PartialEq, PartialOrd, Serialize)]
pub enum Puzzle {
    TutorialOr,
    FabricateXor,
    TutorialMux,
    TutorialAdd,
    AutomateHeliostat,
    AutomateReactor,
    AutomateSensors,
    FabricateInc,
    AutomateRobotArm,
    SandboxBehavior,
    SandboxEvent,
}

impl Puzzle {
    /// Returns the first puzzle in the game, which is always unlocked.
    pub fn first() -> Puzzle { Puzzle::TutorialOr }

    /// Returns an iterator over all puzzles.
    pub fn all() -> PuzzleIter { Puzzle::iter() }

    pub fn title(self) -> &'static str { self.data().title }

    pub fn kind(self) -> PuzzleKind { self.data().kind }

    pub fn score_units(self) -> &'static str { self.data().score_units }

    pub fn graph_bounds(self) -> (i32, i32) { self.data().graph_bounds }

    pub fn description(self) -> &'static str { self.data().description }

    pub fn instructions(self) -> &'static str { self.data().instructions }

    pub fn allows_events(self) -> bool { self.data().allow_events }

    fn data(self) -> &'static PuzzleData {
        match self {
            Puzzle::TutorialOr => {
                &PuzzleData {
                    title: "1-Bit OR Gate",
                    kind: PuzzleKind::Tutorial,
                    allow_events: false,
                    score_units: "Wire Length",
                    graph_bounds: (50, 50),
                    description: "\
                        Tutorial: Build a 1-bit $*OR$* gate out of $*AND$* \
                        and $*NOT$* gates.\n\n\
                        Once this task is completed, you will be able to use \
                        $*OR$* gates in future tasks.",
                    instructions: "\
                        * $!Your goal is to construct an $*OR$* gate.\n\
                        * $!The output on the right side of the board should \
                          be 1 if either input is 1, or 0 if both inputs are \
                          0.\n\
                        * $!Note that ($/a$/ OR $/b$/) is equivalent to \
                          NOT ((NOT $/a$/) AND (NOT $/b$/)).",
                }
            }
            Puzzle::FabricateXor => {
                &PuzzleData {
                    title: "1-Bit XOR Gate",
                    kind: PuzzleKind::Fabricate,
                    allow_events: false,
                    score_units: "Wire Length",
                    graph_bounds: (50, 50),
                    description: "\
                        Build a 1-bit $*XOR$* gate out of $*AND$*, \
                        $*OR$*, and $*NOT$* gates.\n\n\
                        Once this task is completed, you will be able to use \
                        $*XOR$* gates in future tasks.",
                    instructions: "\
                        * $!Your goal is to construct a $*XOR$* gate.\n\
                        * $!The output on the right side of the board should \
                          be 1 if exactly one input is 1, but not both.\n\
                        * $!Note that ($/a$/ XOR $/b$/) is equivalent to \
                          ($/a$/ OR $/b$/) AND NOT ($/a$/ AND $/b$/).",
                }
            }
            Puzzle::TutorialMux => {
                &PuzzleData {
                    title: "1-Bit MUX",
                    kind: PuzzleKind::Tutorial,
                    allow_events: false,
                    score_units: "Wire Length",
                    graph_bounds: (50, 50),
                    description: "\
                        Tutorial: Build a 1-bit $*MUX$* using other logic \
                        gates.\n\n\
                        Once this task is completed, you will be able to use \
                        $*MUX$* chips in future tasks.",
                    instructions: "\
                        * $!Your goal is to construct a 1-bit MUX.\n\
                        * $!The output should be the value of $*in0$* if \
                          $*ctrl$* is 0, or of $*in1$* if $*ctrl$* is 1.\n\
                        * $!If $/a$/ and $/b$/ are the inputs and $/c$/ is \
                          the control, then a MUX is    \
                          ($/a$/ AND NOT $/c$/) OR ($/b$/ AND $/c$/).",
                }
            }
            Puzzle::TutorialAdd => {
                &PuzzleData {
                    title: "4-Bit Adder",
                    kind: PuzzleKind::Tutorial,
                    allow_events: false,
                    score_units: "Wire Length",
                    graph_bounds: (50, 50),
                    description: "\
                        Tutorial: Build a 4-bit adder using 2-bit adders, \
                        packers and unpackers.\n\n\
                        Once this task is completed, you will be able to use \
                        generic $*Add$* chips in future tasks.",
                    instructions: "\
                        * $!Your goal is to construct a 4-bit adder.\n\
                        * $!The output should be the sum of $*in1$* and \
                          $*in2$*.  This sum will never be more than 15.\n\
                        * $!You can use $*Unpack$* chips to separate the \
                          4-bit inputs into hi and lo 2-bit values that the \
                          2-bit adders can accept.  Remember to handle carry \
                          bits appropriately.",
                }
            }
            Puzzle::AutomateHeliostat => {
                &PuzzleData {
                    title: "Heliostat",
                    kind: PuzzleKind::Automate,
                    allow_events: false,
                    score_units: "Time",
                    graph_bounds: (150, 150),
                    description: "\
                        Automate the ship's heliostat to reflect sunlight \
                        onto the solar panels at the optimal angle.",
                    instructions: "\
                        * $!Your goal is fill the energy meter by always \
                          moving the heliostat towards the optimal position.\n\
                        * $!The optimal position is given by the sensor \
                          interface on the left side of the board.  This \
                          optimal position will change from time to time.\n\
                        * $!The current position is given by the motor \
                          interface on the right side of the board.\n\
                        * $!The closer the current position is to optimal, \
                          the more energy will be produced.",
                }
            }
            Puzzle::AutomateReactor => {
                &PuzzleData {
                    title: "Backup Reactor",
                    kind: PuzzleKind::Automate,
                    allow_events: false,
                    score_units: "Time",
                    graph_bounds: (150, 150),
                    description: "\
                        Manipulate the reactor's control rods to regulate \
                        the power output to the desired level.",
                    instructions: "TODO",
                }
            }
            Puzzle::AutomateSensors => {
                &PuzzleData {
                    title: "Main Sensors",
                    kind: PuzzleKind::Automate,
                    allow_events: false,
                    score_units: "Wire Length",
                    graph_bounds: (150, 150),
                    description: "\
                        Design a replacement signal amplifier for the main \
                        sensor array.",
                    instructions: "TODO",
                }
            }
            Puzzle::FabricateInc => {
                &PuzzleData {
                    title: "4-Bit Incrementor",
                    kind: PuzzleKind::Fabricate,
                    allow_events: true,
                    score_units: "Wire Length",
                    graph_bounds: (50, 50),
                    description: "\
                        Build a 4-bit incrementor using more basic event and \
                        behavior chips.\n\n\
                        Once this task is completed, you will be able to use \
                        generic $*Inc$* chips in future tasks.",
                    instructions: "\
                        * $!Your goal is to construct an incrementor.\n\
                        * $!When an input event arrives, the circuit should \
                          add the input behavior value to the event value, \
                          and emit an output event with the sum.",
                }
            }
            Puzzle::AutomateRobotArm => {
                &PuzzleData {
                    title: "Manipulator Arm",
                    kind: PuzzleKind::Automate,
                    allow_events: true,
                    score_units: "Time",
                    graph_bounds: (150, 150),
                    description: "\
                        Operate a robotic arm in response to radio commands.",
                    instructions: "",
                }
            }
            Puzzle::SandboxBehavior => {
                &PuzzleData {
                    title: "Behavior Sandbox",
                    kind: PuzzleKind::Sandbox,
                    allow_events: false,
                    score_units: "",
                    graph_bounds: (100, 100),
                    description: "\
                        Build any circuits you want using all behavior chips \
                        that are currently available.  You can use this area \
                        for prototyping, experimentation, or freeform design.",
                    instructions: "",
                }
            }
            Puzzle::SandboxEvent => {
                &PuzzleData {
                    title: "Event Sandbox",
                    kind: PuzzleKind::Sandbox,
                    allow_events: true,
                    score_units: "",
                    graph_bounds: (100, 100),
                    description: "\
                        Build any circuits you want using all behavior and \
                        event chips that are currently available.  You can \
                        use this area for prototyping, experimentation, or \
                        freeform design.",
                    instructions: "",
                }
            }
        }
    }
}

//===========================================================================//

struct PuzzleData {
    title: &'static str,
    kind: PuzzleKind,
    allow_events: bool,
    score_units: &'static str,
    graph_bounds: (i32, i32),
    description: &'static str,
    instructions: &'static str,
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::Puzzle;
    use std::str::FromStr;

    #[test]
    fn puzzle_to_and_from_string() {
        for puzzle in Puzzle::all() {
            let string = format!("{:?}", puzzle);
            assert_eq!(Puzzle::from_str(&string), Ok(puzzle));
        }
    }
}

//===========================================================================//
