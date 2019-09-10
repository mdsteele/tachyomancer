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
use tachy::geom::CoordsSize;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    FabricateHalve,
    FabricateMul,
    AutomateHeliostat,
    AutomateReactor,
    AutomateSensors,
    CommandLander,
    TutorialDemux,
    TutorialSum,
    FabricateInc,
    AutomateBeacon,
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

    pub fn initial_board_size(self) -> CoordsSize {
        let (width, height) = self.data().init_size;
        CoordsSize::new(width, height)
    }

    fn data(self) -> &'static PuzzleData {
        match self {
            Puzzle::AutomateBeacon => {
                &PuzzleData {
                    title: "Subspace Beacon",
                    kind: PuzzleKind::Automate,
                    allow_events: false,
                    init_size: (8, 6),
                    score_units: "Time",
                    graph_bounds: (150, 150),
                    description: "TODO",
                    instructions: "\
                        * $!Your goal is TODO.\n\
                        * $!The optimal position is given by the sensor \
                          interface on the left side of the board.  This \
                          optimal position will change over time.\n\
                        * $!The current position is given by the motor \
                          interface on the right side of the board.\n\
                        * $!The closer the current position is to optimal, \
                          TODO.",
                }
            }
            Puzzle::AutomateHeliostat => {
                &PuzzleData {
                    title: "Heliostat",
                    kind: PuzzleKind::Automate,
                    allow_events: false,
                    init_size: (6, 5),
                    score_units: "Time",
                    graph_bounds: (50, 150),
                    description: "\
                        Automate the ship's heliostat to reflect sunlight \
                        onto the solar panels at the optimal angle.",
                    instructions: "\
                        * $!Your goal is to fill the energy meter by always \
                          moving the heliostat towards the optimal position.\n\
                        * $!The optimal position is given by the sensor \
                          interface on the left side of the board.  This \
                          optimal position will change over time.\n\
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
                    init_size: (6, 8),
                    score_units: "Time",
                    graph_bounds: (150, 150),
                    description: "\
                        Manipulate the reactor's control rods to regulate \
                        the power output to the desired level.",
                    instructions: "TODO",
                }
            }
            Puzzle::AutomateRobotArm => {
                &PuzzleData {
                    title: "Manipulator Arm",
                    kind: PuzzleKind::Automate,
                    allow_events: true,
                    init_size: (8, 6),
                    score_units: "Time",
                    graph_bounds: (150, 150),
                    description: "\
                        Operate a robotic arm in response to radio commands.",
                    instructions: "",
                }
            }
            Puzzle::AutomateSensors => {
                &PuzzleData {
                    title: "Main Sensors",
                    kind: PuzzleKind::Automate,
                    allow_events: false,
                    init_size: (6, 7),
                    score_units: "Time",
                    graph_bounds: (100, 150),
                    description: "\
                        Design a replacement signal amplifier for the main \
                        sensor array.",
                    instructions: "\
                        * $!The two inputs indicate the current upper and \
                          lower bounds (inclusive) of the scan range.\n\
                        * $!Your goal is to subdivide the scan range at each \
                          step, until the final value is found.  The final \
                          value will always be odd.\n\
                        * $!When the bounds are 2 or more apart, output \
                          any value strictly between the two.\n\
                        * $!When the bounds are exactly 1 apart, output \
                          whichever of those two values is odd.\n\
                        * $!When the bounds are equal, output that final \
                          value to terminate the scan.\n\
                        * $!Note that ($/x$/ AND 1) is 0 when $/x$/ is even, \
                          and 1 when $/x$/ is odd.",
                }
            }
            Puzzle::CommandLander => {
                &PuzzleData {
                    title: "Orbital Lander",
                    kind: PuzzleKind::Command,
                    allow_events: false,
                    init_size: (6, 8),
                    score_units: "Time",
                    graph_bounds: (150, 150),
                    description: "TODO",
                    instructions: "TODO",
                }
            }
            Puzzle::FabricateHalve => {
                &PuzzleData {
                    title: "4-Bit Halver",
                    kind: PuzzleKind::Fabricate,
                    allow_events: false,
                    init_size: (7, 5),
                    score_units: "Wire Length",
                    graph_bounds: (50, 50),
                    description: "\
                        Build a 4-bit halver using packers and unpackers.\n\n\
                        Once this task is completed, you will be able to use \
                        generic $*Halve$* chips in future tasks.",
                    instructions: "\
                        * $!The output should be half the value of the input, \
                          rounded down.\n\
                        * $!This can be achieved by unpacking the input into \
                          four 1-bit wires, then packing the highest three \
                          wires of the input into the lowest three wires of \
                          the output.",
                }
            }
            Puzzle::FabricateInc => {
                &PuzzleData {
                    title: "4-Bit Incrementor",
                    kind: PuzzleKind::Fabricate,
                    allow_events: true,
                    init_size: (5, 5),
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
            Puzzle::FabricateMul => {
                &PuzzleData {
                    title: "8-Bit Multiplier",
                    kind: PuzzleKind::Fabricate,
                    allow_events: false,
                    init_size: (7, 7),
                    score_units: "Wire Length",
                    graph_bounds: (50, 100),
                    description: "\
                        Build an 8-bit multiplier using 4-bit multipliers.\n\n\
                        Once this task is completed, you will be able to use \
                        generic $*Mul$* chips in future tasks.",
                    instructions: "\
                        * $!Your goal is to construct an 8-bit multiplier.\n\
                        * $!The output should be the product of $*In1$* and \
                          $*In2$*.  This product will never be more than \
                          255.",
                }
            }
            Puzzle::FabricateXor => {
                &PuzzleData {
                    title: "1-Bit XOR Gate",
                    kind: PuzzleKind::Fabricate,
                    allow_events: false,
                    init_size: (5, 5),
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
            Puzzle::SandboxBehavior => {
                &PuzzleData {
                    title: "Behavior Sandbox",
                    kind: PuzzleKind::Sandbox,
                    allow_events: false,
                    init_size: (8, 6),
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
                    init_size: (8, 6),
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
            Puzzle::TutorialAdd => {
                &PuzzleData {
                    title: "4-Bit Adder",
                    kind: PuzzleKind::Tutorial,
                    allow_events: false,
                    init_size: (5, 5),
                    score_units: "Wire Length",
                    graph_bounds: (50, 50),
                    description: "\
                        Tutorial: Build a 4-bit adder using 2-bit adders, \
                        packers and unpackers.\n\n\
                        Once this task is completed, you will be able to use \
                        generic $*Add$* chips in future tasks.",
                    instructions: "\
                        * $!Your goal is to construct a 4-bit adder.\n\
                        * $!The output should be the sum of $*In1$* and \
                          $*In2$*.  This sum will never be more than 15.\n\
                        * $!You can use $*Unpack$* chips to separate the \
                          4-bit inputs into hi and lo 2-bit values that the \
                          2-bit adders can accept.  Remember to handle carry \
                          bits appropriately.",
                }
            }
            Puzzle::TutorialDemux => {
                &PuzzleData {
                    title: "1-Bit DEMUX",
                    kind: PuzzleKind::Tutorial,
                    allow_events: true,
                    init_size: (5, 5),
                    score_units: "Wire Length",
                    graph_bounds: (50, 50),
                    description: "\
                        Tutorial: Build a 1-bit $*DEMUX$* using event filter \
                        chips.\n\n\
                        Once this task is completed, you will be able to use \
                        $*DEMUX$* chips in future tasks.",
                    instructions: "\
                        * $!Your goal is to construct a 1-bit DEMUX.\n\
                        * $!When an event arrives on $*In$*, it should be \
                          sent to $*Out0$* if $*Ctrl$* is 0, or to $*Out1$* \
                          if $*Ctrl$* is 1.",
                }
            }
            Puzzle::TutorialMux => {
                &PuzzleData {
                    title: "1-Bit MUX",
                    kind: PuzzleKind::Tutorial,
                    allow_events: false,
                    init_size: (5, 5),
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
            Puzzle::TutorialOr => {
                &PuzzleData {
                    title: "1-Bit OR Gate",
                    kind: PuzzleKind::Tutorial,
                    allow_events: false,
                    init_size: (5, 5),
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
            Puzzle::TutorialSum => {
                &PuzzleData {
                    title: "Resettable Sum",
                    kind: PuzzleKind::Tutorial,
                    allow_events: true,
                    init_size: (7, 7),
                    score_units: "Wire Length",
                    graph_bounds: (50, 100),
                    description: "\
                        Tutorial: Build a circuit for tracking a running \
                        total, and that can also be reset back to zero.\n\n\
                        Once this task is completed, you will be able to use \
                        $*Sum$* chips in future tasks.",
                    instructions: "\
                        * $!The output should start at zero.  When an input \
                          event arrives, its value should be added to the \
                          running total output.\n\
                        * $!When a reset event arrives, the output should be \
                          reset back to zero.\n\
                        * $!If a reset and input event arrive simultaneously, \
                          the reset should occur just $/before$/ the input is \
                          added in, so that the output for that time step is \
                          the value of the input event.",
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
    init_size: (i32, i32),
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

    #[test]
    fn puzzle_kinds() {
        for puzzle in Puzzle::all() {
            let name = format!("{:?}", puzzle);
            let kind = format!("{:?}", puzzle.kind());
            assert!(name.starts_with(kind.as_str()));
        }
    }
}

//===========================================================================//
