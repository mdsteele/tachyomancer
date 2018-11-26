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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Puzzle {
    TutorialOr,
    AutomateHeliostat,
    SandboxBehavior,
    SandboxEvent,
}

impl Puzzle {
    #[allow(dead_code)]
    pub fn title(self) -> &'static str { self.data().title }

    pub fn kind(self) -> PuzzleKind { self.data().kind }

    #[allow(dead_code)]
    pub fn instructions(self) -> &'static str { self.data().instructions }

    pub fn allows_events(self) -> bool { self.data().allow_events }

    fn data(self) -> &'static PuzzleData {
        match self {
            Puzzle::TutorialOr => {
                &PuzzleData {
                    title: "1-Bit Or Gate",
                    kind: PuzzleKind::Tutorial,
                    instructions: "\
                        Create a circuit that outputs 1 if at least one of \
                        the two inputs is 1, or 0 if both inputs are 0.",
                    allow_events: false,
                }
            }
            Puzzle::AutomateHeliostat => {
                &PuzzleData {
                    title: "Heliostat",
                    kind: PuzzleKind::Automate,
                    instructions: "\
                        Move the heliostat towards the optimal position.",
                    allow_events: false,
                }
            }
            Puzzle::SandboxBehavior => {
                &PuzzleData {
                    title: "Behavior Sandbox",
                    kind: PuzzleKind::Sandbox,
                    instructions: "",
                    allow_events: false,
                }
            }
            Puzzle::SandboxEvent => {
                &PuzzleData {
                    title: "Event Sandbox",
                    kind: PuzzleKind::Sandbox,
                    instructions: "",
                    allow_events: true,
                }
            }
        }
    }
}

//===========================================================================//

struct PuzzleData {
    title: &'static str,
    kind: PuzzleKind,
    instructions: &'static str,
    allow_events: bool,
}

//===========================================================================//
