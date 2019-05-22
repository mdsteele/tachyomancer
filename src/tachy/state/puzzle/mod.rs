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
mod tutorial;

pub use self::heliostat::HeliostatEval;
pub use self::iface::Interface;
pub use self::robotarm::RobotArmEval;
pub use self::tutorial::{TutorialBubblePosition, TutorialMuxEval,
                         TutorialOrEval, TutorialXorEval};
use super::eval::PuzzleEval;
use tachy::geom::{Coords, Direction};
use tachy::save::Puzzle;

//===========================================================================//

pub trait PuzzleExt {
    fn interfaces(&self) -> &'static [Interface];
    fn tutorial_bubbles(
        &self)
        -> &'static [(TutorialBubblePosition, &'static str)];
}

//===========================================================================//

impl PuzzleExt for Puzzle {
    fn interfaces(&self) -> &'static [Interface] {
        match self {
            Puzzle::TutorialOr => self::tutorial::OR_INTERFACES,
            Puzzle::TutorialXor => self::tutorial::XOR_INTERFACES,
            Puzzle::TutorialMux => self::tutorial::MUX_INTERFACES,
            Puzzle::AutomateHeliostat => self::heliostat::INTERFACES,
            Puzzle::AutomateReactor => self::reactor::INTERFACES,
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
