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

mod fabricate;
mod heliostat;
mod iface;
mod reactor;
mod rng;
mod robotarm;
mod sandbox;
mod sensors;
mod shared;
mod tutorial;

pub use self::fabricate::{FabricateHalveEval, FabricateIncEval,
                          FabricateMulEval, FabricateXorEval};
pub use self::heliostat::HeliostatEval;
pub use self::iface::Interface;
pub use self::robotarm::RobotArmEval;
pub use self::sensors::SensorsEval;
pub use self::tutorial::{TutorialAddEval, TutorialBubblePosition,
                         TutorialDemuxEval, TutorialMuxEval, TutorialOrEval};
use super::chip::{ChipAvailability, ChipExt};
use super::eval::PuzzleEval;
use tachy::geom::{Coords, Direction};
use tachy::save::{CHIP_CATEGORIES, ChipSet, ChipType, Conversation, Profile,
                  Puzzle, PuzzleKind};

//===========================================================================//

pub trait PuzzleExt {
    fn origin_conversations(&self) -> &'static [Conversation];
    fn allowed_chips(&self, profile: &Profile) -> ChipSet;
    fn interfaces(&self) -> &'static [Interface];
    fn tutorial_bubbles(
        &self)
        -> &'static [(TutorialBubblePosition, &'static str)];
}

//===========================================================================//

impl PuzzleExt for Puzzle {
    fn origin_conversations(&self) -> &'static [Conversation] {
        match self {
            Puzzle::AutomateHeliostat => &[Conversation::RestorePower],
            Puzzle::AutomateReactor => &[Conversation::StepTwo],
            Puzzle::AutomateSensors => &[Conversation::CaptainsCall],
            Puzzle::TutorialAdd => &[Conversation::Basics],
            Puzzle::TutorialMux => &[Conversation::Basics],
            Puzzle::TutorialOr => &[Conversation::Basics],
            _ => &[], // TODO: origin_conversations for other puzzles
        }
    }

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
            Puzzle::AutomateHeliostat => self::heliostat::INTERFACES,
            Puzzle::AutomateReactor => self::reactor::INTERFACES,
            Puzzle::AutomateRobotArm => self::robotarm::INTERFACES,
            Puzzle::AutomateSensors => self::sensors::INTERFACES,
            Puzzle::FabricateHalve => self::fabricate::HALVE_INTERFACES,
            Puzzle::FabricateInc => self::fabricate::INC_INTERFACES,
            Puzzle::FabricateMul => self::fabricate::MUL_INTERFACES,
            Puzzle::FabricateXor => self::fabricate::XOR_INTERFACES,
            Puzzle::SandboxBehavior => self::sandbox::BEHAVIOR_INTERFACES,
            Puzzle::SandboxEvent => self::sandbox::EVENT_INTERFACES,
            Puzzle::TutorialAdd => self::tutorial::ADD_INTERFACES,
            Puzzle::TutorialDemux => self::tutorial::DEMUX_INTERFACES,
            Puzzle::TutorialMux => self::tutorial::MUX_INTERFACES,
            Puzzle::TutorialOr => self::tutorial::OR_INTERFACES,
        }
    }

    fn tutorial_bubbles(
        &self)
        -> &'static [(TutorialBubblePosition, &'static str)] {
        match self {
            Puzzle::TutorialAdd => self::tutorial::ADD_BUBBLES,
            Puzzle::TutorialDemux => self::tutorial::DEMUX_BUBBLES,
            Puzzle::TutorialMux => self::tutorial::MUX_BUBBLES,
            Puzzle::TutorialOr => self::tutorial::OR_BUBBLES,
            _ => &[],
        }
    }
}

fn is_chip_allowed_in(ctype: ChipType, puzzle: Puzzle, profile: &Profile)
                      -> bool {
    if !puzzle.allows_events() && ctype.uses_events() {
        return false;
    }
    match ctype.availibility() {
        ChipAvailability::Always => true,
        ChipAvailability::InteractiveOnly => {
            match puzzle.kind() {
                PuzzleKind::Tutorial | PuzzleKind::Fabricate |
                PuzzleKind::Automate => false,
                PuzzleKind::Command | PuzzleKind::Sandbox => true,
            }
        }
        ChipAvailability::OnlyIn(puzzles) => puzzles.contains(&puzzle),
        ChipAvailability::StartingWith(other_puzzle) => puzzle >= other_puzzle,
        ChipAvailability::UnlockedBy(other_puzzle) => {
            other_puzzle < puzzle && profile.is_puzzle_solved(other_puzzle)
        }
    }
}


//===========================================================================//

pub(super) fn new_puzzle_eval(puzzle: Puzzle,
                              slots: Vec<Vec<((Coords, Direction), usize)>>)
                              -> Box<PuzzleEval> {
    match puzzle {
        Puzzle::AutomateHeliostat => Box::new(HeliostatEval::new(slots)),
        Puzzle::AutomateReactor => {
            Box::new(self::reactor::AutomateReactorEval::new(slots))
        }
        Puzzle::AutomateRobotArm => Box::new(RobotArmEval::new(slots)),
        Puzzle::AutomateSensors => {
            Box::new(self::sensors::SensorsEval::new(slots))
        }
        Puzzle::FabricateHalve => Box::new(FabricateHalveEval::new(slots)),
        Puzzle::FabricateInc => Box::new(FabricateIncEval::new(slots)),
        Puzzle::FabricateMul => Box::new(FabricateMulEval::new(slots)),
        Puzzle::FabricateXor => Box::new(FabricateXorEval::new(slots)),
        Puzzle::SandboxBehavior => {
            Box::new(self::sandbox::SandboxBehaviorEval::new(slots))
        }
        Puzzle::SandboxEvent => {
            Box::new(self::sandbox::SandboxEventEval::new(slots))
        }
        Puzzle::TutorialAdd => Box::new(TutorialAddEval::new(slots)),
        Puzzle::TutorialDemux => Box::new(TutorialDemuxEval::new(slots)),
        Puzzle::TutorialMux => Box::new(TutorialMuxEval::new(slots)),
        Puzzle::TutorialOr => Box::new(TutorialOrEval::new(slots)),
    }
}

//===========================================================================//
