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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Puzzle {
    TutorialOr,
    AutomateHeliostat,
    SandboxBehavior,
    SandboxEvent,
}

const ALL_PUZZLES: &[Puzzle] = &[
    Puzzle::TutorialOr,
    Puzzle::AutomateHeliostat,
    Puzzle::SandboxBehavior,
    Puzzle::SandboxEvent,
];

impl Puzzle {
    /// Returns the first puzzle in the game, which is always unlocked.
    pub fn first() -> Puzzle { Puzzle::TutorialOr }

    /// Returns an iterator over all puzzles.
    pub fn all() -> AllPuzzlesIter { AllPuzzlesIter { index: 0 } }

    pub fn title(self) -> &'static str { self.data().title }

    pub fn kind(self) -> PuzzleKind { self.data().kind }

    #[allow(dead_code)]
    pub fn instructions(self) -> &'static str { self.data().instructions }

    pub fn allows_events(self) -> bool { self.data().allow_events }

    pub fn static_verification_data(self) -> &'static [u64] {
        self.data().verification
    }

    fn data(self) -> &'static PuzzleData {
        match self {
            Puzzle::TutorialOr => {
                &PuzzleData {
                    title: "1-Bit Or Gate",
                    kind: PuzzleKind::Tutorial,
                    allow_events: false,
                    instructions: "\
                        Create a circuit that outputs 1 if at least one of \
                        the two inputs is 1, or 0 if both inputs are 0.",
                    verification: &[0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1],
                }
            }
            Puzzle::AutomateHeliostat => {
                &PuzzleData {
                    title: "Heliostat",
                    kind: PuzzleKind::Automate,
                    allow_events: false,
                    instructions: "\
                        Move the heliostat towards the optimal position.",
                    verification: &[],
                }
            }
            Puzzle::SandboxBehavior => {
                &PuzzleData {
                    title: "Behavior Sandbox",
                    kind: PuzzleKind::Sandbox,
                    allow_events: false,
                    instructions: "",
                    verification: &[],
                }
            }
            Puzzle::SandboxEvent => {
                &PuzzleData {
                    title: "Event Sandbox",
                    kind: PuzzleKind::Sandbox,
                    allow_events: true,
                    instructions: "",
                    verification: &[],
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
    instructions: &'static str,
    verification: &'static [u64],
}

//===========================================================================//

pub struct AllPuzzlesIter {
    index: usize,
}

impl<'a> Iterator for AllPuzzlesIter {
    type Item = Puzzle;

    fn next(&mut self) -> Option<Puzzle> {
        if self.index < ALL_PUZZLES.len() {
            let puzzle = ALL_PUZZLES[self.index];
            self.index += 1;
            Some(puzzle)
        } else {
            None
        }
    }
}

//===========================================================================//
