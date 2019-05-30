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

mod heliostat;
mod iface;
mod reactor;
mod rng;
mod robotarm;
mod sandbox;
mod sensors;
mod tutorial;

pub use self::heliostat::HeliostatEval;
pub use self::iface::Interface;
pub use self::robotarm::RobotArmEval;
pub use self::sensors::SensorsEval;
pub use self::tutorial::{TutorialBubblePosition, TutorialMuxEval,
                         TutorialOrEval, TutorialXorEval};
use super::chip::ChipExt;
use super::eval::PuzzleEval;
use tachy::geom::{Coords, Direction};
use tachy::save::{CHIP_CATEGORIES, ChipSet, ChipType, Profile, Puzzle,
                  PuzzleKind};

//===========================================================================//

pub trait PuzzleExt {
    fn allowed_chips(&self, profile: &Profile) -> ChipSet;
    fn interfaces(&self) -> &'static [Interface];
    fn tutorial_bubbles(
        &self)
        -> &'static [(TutorialBubblePosition, &'static str)];
}

//===========================================================================//

impl PuzzleExt for Puzzle {
    fn allowed_chips(&self, profile: &Profile) -> ChipSet {
        let mut allowed = ChipSet::new();
        for &(_, ctypes) in CHIP_CATEGORIES.iter() {
            for &ctype in ctypes.iter() {
                if is_chip_allowed_in(ctype, *self, profile) {
                    allowed.insert(ctype);
                }
            }
        }
        allowed
    }

    fn interfaces(&self) -> &'static [Interface] {
        match self {
            Puzzle::TutorialOr => self::tutorial::OR_INTERFACES,
            Puzzle::TutorialXor => self::tutorial::XOR_INTERFACES,
            Puzzle::TutorialMux => self::tutorial::MUX_INTERFACES,
            Puzzle::AutomateHeliostat => self::heliostat::INTERFACES,
            Puzzle::AutomateReactor => self::reactor::INTERFACES,
            Puzzle::AutomateSensors => self::sensors::INTERFACES,
            Puzzle::AutomateRobotArm => self::robotarm::INTERFACES,
            Puzzle::SandboxBehavior => self::sandbox::BEHAVIOR_INTERFACES,
            Puzzle::SandboxEvent => self::sandbox::EVENT_INTERFACES,
        }
    }

    fn tutorial_bubbles(
        &self)
        -> &'static [(TutorialBubblePosition, &'static str)] {
        match self {
            Puzzle::TutorialOr => self::tutorial::OR_BUBBLES,
            Puzzle::TutorialXor => self::tutorial::XOR_BUBBLES,
            Puzzle::TutorialMux => self::tutorial::MUX_BUBBLES,
            _ => &[],
        }
    }
}

pub fn is_chip_allowed_in(ctype: ChipType, puzzle: Puzzle, profile: &Profile)
                          -> bool {
    if !puzzle.allows_events() && ctype.uses_events() {
        return false;
    }
    match puzzle.kind() {
        PuzzleKind::Tutorial | PuzzleKind::Fabricate |
        PuzzleKind::Automate => {
            if ctype.is_interactive() {
                return false;
            }
        }
        PuzzleKind::Command | PuzzleKind::Sandbox => {}
    }
    if let Some(other_puzzle) = ctype.unlocked_by() {
        if other_puzzle >= puzzle || !profile.is_puzzle_solved(other_puzzle) {
            return false;
        }
    }
    return true;
}


//===========================================================================//

pub fn new_puzzle_eval(puzzle: Puzzle,
                       slots: Vec<Vec<((Coords, Direction), usize)>>)
                       -> Box<PuzzleEval> {
    match puzzle {
        Puzzle::TutorialOr => Box::new(TutorialOrEval::new(slots)),
        Puzzle::TutorialXor => Box::new(TutorialXorEval::new(slots)),
        Puzzle::TutorialMux => Box::new(TutorialMuxEval::new(slots)),
        Puzzle::AutomateHeliostat => Box::new(HeliostatEval::new(slots)),
        Puzzle::AutomateReactor => {
            Box::new(self::reactor::AutomateReactorEval::new(slots))
        }
        Puzzle::AutomateSensors => {
            Box::new(self::sensors::SensorsEval::new(slots))
        }
        Puzzle::AutomateRobotArm => Box::new(RobotArmEval::new(slots)),
        Puzzle::SandboxBehavior => {
            Box::new(self::sandbox::SandboxBehaviorEval::new(slots))
        }
        Puzzle::SandboxEvent => {
            Box::new(self::sandbox::SandboxEventEval::new(slots))
        }
    }
}

//===========================================================================//
