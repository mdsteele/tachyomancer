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

mod beacon;
mod fab_arith;
mod fab_clock;
mod grapple;
mod heliostat;
mod iface;
mod lander;
mod mining;
mod reactor;
mod rng;
mod robotarm;
mod sandbox;
mod sensors;
mod shared;
mod storage;
mod tutor_bvr;
mod tutor_evt;
mod xunit;

pub use self::beacon::BeaconEval;
pub use self::fab_arith::{
    FabricateHalveEval, FabricateIncEval, FabricateMulEval, FabricateXorEval,
};
pub use self::fab_clock::{FabricateEggTimerEval, FabricateStopwatchEval};
pub use self::grapple::GrappleEval;
pub use self::heliostat::HeliostatEval;
pub use self::iface::Interface;
pub use self::lander::LanderEval;
pub use self::mining::MiningRobotEval;
pub use self::robotarm::RobotArmEval;
pub use self::sensors::SensorsEval;
pub use self::shared::TutorialBubblePosition;
pub use self::storage::StorageDepotEval;
pub use self::tutor_bvr::{TutorialAddEval, TutorialMuxEval, TutorialOrEval};
pub use self::tutor_evt::{TutorialDemuxEval, TutorialSumEval};
pub use self::xunit::XUnitEval;
use super::chip::{ChipAvailability, ChipExt};
use super::eval::PuzzleEval;
use crate::geom::{Coords, Direction};
use crate::save::{
    ChipSet, ChipType, Conversation, Puzzle, PuzzleKind, CHIP_CATEGORIES,
};
use std::collections::HashSet;

//===========================================================================//

pub trait PuzzleExt {
    fn origin_conversations(&self) -> &'static [Conversation];
    fn allowed_chips(&self, solved_puzzles: &HashSet<Puzzle>) -> ChipSet;
    fn interfaces(&self) -> &'static [Interface];
    fn tutorial_bubbles(
        &self,
    ) -> &'static [(TutorialBubblePosition, &'static str)];
}

//===========================================================================//

impl PuzzleExt for Puzzle {
    fn origin_conversations(&self) -> &'static [Conversation] {
        match self {
            Puzzle::AutomateGrapple => {
                &[Conversation::CaptainAwake, Conversation::AnIdea]
            }
            Puzzle::AutomateHeliostat => &[Conversation::RestorePower],
            Puzzle::AutomateReactor => {
                &[Conversation::ReactorSpecs, Conversation::MorePower]
            }
            Puzzle::AutomateSensors => {
                &[Conversation::WhereAreWe, Conversation::LowVisibility]
            }
            Puzzle::CommandLander => &[Conversation::Descent],
            Puzzle::FabricateHalve
            | Puzzle::FabricateMul
            | Puzzle::FabricateXor => &[Conversation::MoreComponents],
            Puzzle::TutorialAdd | Puzzle::TutorialMux | Puzzle::TutorialOr => {
                &[Conversation::Basics]
            }
            Puzzle::TutorialDemux => &[Conversation::AdvancedCircuits],
            _ => &[], // TODO: origin_conversations for other puzzles
        }
    }

    fn allowed_chips(&self, solved_puzzles: &HashSet<Puzzle>) -> ChipSet {
        let mut allowed = ChipSet::new();
        for &(_, ctypes) in CHIP_CATEGORIES.iter() {
            for &ctype in ctypes.iter() {
                if is_chip_allowed_in(ctype, *self, solved_puzzles) {
                    allowed.insert(ctype);
                }
            }
        }
        allowed
    }

    fn interfaces(&self) -> &'static [Interface] {
        match self {
            Puzzle::AutomateBeacon => self::beacon::INTERFACES,
            Puzzle::AutomateGrapple => self::grapple::INTERFACES,
            Puzzle::AutomateHeliostat => self::heliostat::INTERFACES,
            Puzzle::AutomateMiningRobot => self::mining::INTERFACES,
            Puzzle::AutomateReactor => self::reactor::INTERFACES,
            Puzzle::AutomateRobotArm => self::robotarm::INTERFACES,
            Puzzle::AutomateSensors => self::sensors::INTERFACES,
            Puzzle::AutomateStorageDepot => self::storage::INTERFACES,
            Puzzle::AutomateXUnit => self::xunit::INTERFACES,
            Puzzle::CommandLander => self::lander::INTERFACES,
            Puzzle::FabricateEggTimer => self::fab_clock::EGG_TIMER_INTERFACES,
            Puzzle::FabricateHalve => self::fab_arith::HALVE_INTERFACES,
            Puzzle::FabricateInc => self::fab_arith::INC_INTERFACES,
            Puzzle::FabricateMul => self::fab_arith::MUL_INTERFACES,
            Puzzle::FabricateStopwatch => {
                self::fab_clock::STOPWATCH_INTERFACES
            }
            Puzzle::FabricateXor => self::fab_arith::XOR_INTERFACES,
            Puzzle::SandboxBehavior => self::sandbox::BEHAVIOR_INTERFACES,
            Puzzle::SandboxEvent => self::sandbox::EVENT_INTERFACES,
            Puzzle::TutorialAdd => self::tutor_bvr::ADD_INTERFACES,
            Puzzle::TutorialDemux => self::tutor_evt::DEMUX_INTERFACES,
            Puzzle::TutorialMux => self::tutor_bvr::MUX_INTERFACES,
            Puzzle::TutorialOr => self::tutor_bvr::OR_INTERFACES,
            Puzzle::TutorialSum => self::tutor_evt::SUM_INTERFACES,
        }
    }

    fn tutorial_bubbles(
        &self,
    ) -> &'static [(TutorialBubblePosition, &'static str)] {
        match self {
            Puzzle::TutorialAdd => self::tutor_bvr::ADD_BUBBLES,
            Puzzle::TutorialDemux => self::tutor_evt::DEMUX_BUBBLES,
            Puzzle::TutorialMux => self::tutor_bvr::MUX_BUBBLES,
            Puzzle::TutorialOr => self::tutor_bvr::OR_BUBBLES,
            Puzzle::TutorialSum => self::tutor_evt::SUM_BUBBLES,
            _ => &[],
        }
    }
}

fn is_chip_allowed_in(
    ctype: ChipType,
    puzzle: Puzzle,
    solved_puzzles: &HashSet<Puzzle>,
) -> bool {
    if !puzzle.allows_events() && ctype.uses_events() {
        return false;
    }
    match ctype.availibility() {
        ChipAvailability::Always => true,
        ChipAvailability::InteractiveOnly => match puzzle.kind() {
            PuzzleKind::Tutorial
            | PuzzleKind::Fabricate
            | PuzzleKind::Automate => false,
            PuzzleKind::Command | PuzzleKind::Sandbox => true,
        },
        ChipAvailability::OnlyIn(puzzles) => puzzles.contains(&puzzle),
        ChipAvailability::StartingWith(other_puzzle) => puzzle >= other_puzzle,
        ChipAvailability::UnlockedBy(other_puzzle) => {
            puzzle > other_puzzle && solved_puzzles.contains(&other_puzzle)
        }
    }
}

//===========================================================================//

pub(super) fn new_puzzle_eval(
    puzzle: Puzzle,
    slots: Vec<Vec<((Coords, Direction), usize)>>,
) -> Box<dyn PuzzleEval> {
    match puzzle {
        Puzzle::AutomateBeacon => {
            Box::new(self::beacon::BeaconEval::new(slots))
        }
        Puzzle::AutomateGrapple => Box::new(GrappleEval::new(slots)),
        Puzzle::AutomateHeliostat => Box::new(HeliostatEval::new(slots)),
        Puzzle::AutomateMiningRobot => Box::new(MiningRobotEval::new(slots)),
        Puzzle::AutomateReactor => {
            Box::new(self::reactor::AutomateReactorEval::new(slots))
        }
        Puzzle::AutomateRobotArm => Box::new(RobotArmEval::new(slots)),
        Puzzle::AutomateSensors => {
            Box::new(self::sensors::SensorsEval::new(slots))
        }
        Puzzle::AutomateStorageDepot => Box::new(StorageDepotEval::new(slots)),
        Puzzle::AutomateXUnit => Box::new(XUnitEval::new(slots)),
        Puzzle::CommandLander => Box::new(LanderEval::new(slots)),
        Puzzle::FabricateEggTimer => {
            Box::new(FabricateEggTimerEval::new(slots))
        }
        Puzzle::FabricateHalve => Box::new(FabricateHalveEval::new(slots)),
        Puzzle::FabricateInc => Box::new(FabricateIncEval::new(slots)),
        Puzzle::FabricateMul => Box::new(FabricateMulEval::new(slots)),
        Puzzle::FabricateStopwatch => {
            Box::new(FabricateStopwatchEval::new(slots))
        }
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
        Puzzle::TutorialSum => Box::new(TutorialSumEval::new(slots)),
    }
}

//===========================================================================//
