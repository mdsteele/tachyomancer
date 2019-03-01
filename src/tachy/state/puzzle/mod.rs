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
mod sandbox;
mod tutorial;

pub use self::iface::Interface;
use super::eval::PuzzleEval;
use tachy::geom::{Coords, Direction};
use tachy::save::Puzzle;

//===========================================================================//

pub fn puzzle_interfaces(puzzle: Puzzle) -> &'static [Interface] {
    match puzzle {
        Puzzle::TutorialOr => self::tutorial::OR_INTERFACES,
        Puzzle::AutomateHeliostat => self::heliostat::INTERFACES,
        Puzzle::AutomateReactor => self::reactor::INTERFACES,
        Puzzle::SandboxBehavior => self::sandbox::BEHAVIOR_INTERFACES,
        Puzzle::SandboxEvent => self::sandbox::EVENT_INTERFACES,
    }
}

//===========================================================================//

pub fn new_puzzle_eval(puzzle: Puzzle,
                       slots: Vec<Vec<((Coords, Direction), usize)>>)
                       -> Box<PuzzleEval> {
    match puzzle {
        Puzzle::TutorialOr => {
            Box::new(self::tutorial::TutorialOrEval::new(slots))
        }
        Puzzle::AutomateHeliostat => {
            Box::new(self::heliostat::AutomateHeliostatEval::new(slots))
        }
        Puzzle::AutomateReactor => {
            Box::new(self::reactor::AutomateReactorEval::new(slots))
        }
        Puzzle::SandboxBehavior => {
            Box::new(self::sandbox::SandboxBehaviorEval::new(slots))
        }
        Puzzle::SandboxEvent => {
            Box::new(self::sandbox::SandboxEventEval::new(slots))
        }
    }
}

//===========================================================================//
