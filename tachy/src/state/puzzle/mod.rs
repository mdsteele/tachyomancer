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
mod collector;
mod drill;
mod fab_arith;
mod fab_clock;
mod fab_event;
mod fab_memory;
mod fuel;
mod geiger;
mod grapple;
mod heliostat;
mod incubator;
mod injector;
mod lander;
mod mining;
mod reactor;
mod rng;
mod robotarm;
mod sandbox;
mod sensors;
mod shared;
mod shields;
mod sonar;
mod storage;
mod translator;
mod turret;
mod tutor_bvr;
mod tutor_clock;
mod tutor_evt;
mod tutor_ram;
mod xunit;

pub use self::beacon::BeaconEval;
pub use self::collector::CollectorEval;
pub use self::drill::DrillingRigEval;
pub use self::fab_arith::{
    FABRICATE_HALVE_DATA, FABRICATE_MUL_DATA, FABRICATE_XOR_DATA,
};
pub use self::fab_clock::{
    FABRICATE_EGG_TIMER_DATA, FABRICATE_STOPWATCH_DATA,
};
pub use self::fab_event::{FABRICATE_COUNTER_DATA, FABRICATE_INC_DATA};
pub use self::fab_memory::{FABRICATE_QUEUE_DATA, FABRICATE_STACK_DATA};
pub use self::fuel::FuelEval;
pub use self::geiger::GeigerEval;
pub use self::grapple::GrappleEval;
pub use self::heliostat::HeliostatEval;
pub use self::incubator::IncubatorEval;
pub use self::injector::InjectorEval;
pub use self::lander::LanderEval;
pub use self::mining::MiningRobotEval;
pub use self::reactor::ReactorEval;
pub use self::robotarm::RobotArmEval;
pub use self::sensors::SensorsEval;
pub use self::shared::{
    FabricationData, FabricationEval, TutorialBubblePosition,
};
pub use self::shields::ShieldsEval;
pub use self::sonar::SonarEval;
pub use self::storage::StorageDepotEval;
pub use self::translator::TranslatorEval;
pub use self::turret::TurretEval;
pub use self::tutor_bvr::{
    TUTORIAL_ADD_DATA, TUTORIAL_MUX_DATA, TUTORIAL_OR_DATA,
};
pub use self::tutor_clock::TUTORIAL_CLOCK_DATA;
pub use self::tutor_evt::{
    TUTORIAL_AMP_DATA, TUTORIAL_DEMUX_DATA, TUTORIAL_SUM_DATA,
};
pub use self::tutor_ram::TUTORIAL_RAM_DATA;
pub use self::xunit::XUnitEval;
use super::chip::{ChipAvailability, ChipExt};
use super::eval::PuzzleEval;
use super::interface::Interface;
use crate::geom::{Coords, Direction};
use crate::save::{
    ChipSet, ChipType, Conversation, Puzzle, PuzzleKind, PuzzleSet,
    CHIP_CATEGORIES,
};

//===========================================================================//

pub trait PuzzleExt {
    fn origin_conversations(&self) -> &'static [Conversation];
    fn allowed_chips(&self, solved: &PuzzleSet) -> ChipSet;
    fn interfaces(&self) -> &'static [Interface];
    fn tutorial_bubbles(
        &self,
    ) -> &'static [(TutorialBubblePosition, &'static str)];
}

//===========================================================================//

impl PuzzleExt for Puzzle {
    fn origin_conversations(&self) -> &'static [Conversation] {
        match self {
            Puzzle::AutomateBeacon => &[Conversation::OneMoreThing],
            Puzzle::AutomateFuelSynthesis => &[Conversation::MakingFuel],
            Puzzle::AutomateGrapple => {
                &[Conversation::CaptainAwake, Conversation::AnIdea]
            }
            Puzzle::AutomateHeliostat => &[Conversation::RestorePower],
            Puzzle::AutomateMiningRobot => &[Conversation::ScoutReport],
            Puzzle::AutomateReactor => {
                &[Conversation::ReactorSpecs, Conversation::MorePower]
            }
            Puzzle::AutomateRobotArm => &[Conversation::ANewProblem],
            Puzzle::AutomateSensors => {
                &[Conversation::WhereAreWe, Conversation::LowVisibility]
            }
            Puzzle::CommandLander => &[Conversation::Descent],
            Puzzle::CommandTurret => &[Conversation::UnexpectedCompany],
            Puzzle::FabricateHalve
            | Puzzle::FabricateMul
            | Puzzle::FabricateXor => &[Conversation::MoreComponents],
            Puzzle::SandboxBehavior => &[Conversation::Prototyping],
            Puzzle::SandboxEvent => &[Conversation::MorePrototypes],
            Puzzle::TutorialAdd | Puzzle::TutorialMux | Puzzle::TutorialOr => {
                &[Conversation::Basics]
            }
            Puzzle::TutorialAmp
            | Puzzle::TutorialDemux
            | Puzzle::TutorialSum => &[Conversation::AdvancedCircuits],
            Puzzle::FabricateInc | Puzzle::FabricateCounter => {
                &[Conversation::AdditionalChips]
            }
            Puzzle::TutorialClock => &[Conversation::KeepingTime],
            Puzzle::TutorialRam => &[Conversation::Memory],
            _ => &[], // TODO: origin_conversations for other puzzles
        }
    }

    fn allowed_chips(&self, solved: &PuzzleSet) -> ChipSet {
        let mut allowed = ChipSet::new();
        for &(_, ctypes) in CHIP_CATEGORIES.iter() {
            for &ctype in ctypes.iter() {
                if is_chip_allowed_in(ctype, *self, solved) {
                    allowed.insert(ctype);
                }
            }
        }
        allowed
    }

    fn interfaces(&self) -> &'static [Interface] {
        match self {
            Puzzle::AutomateBeacon => self::beacon::INTERFACES,
            Puzzle::AutomateCollector => self::collector::INTERFACES,
            Puzzle::AutomateDrillingRig => self::drill::INTERFACES,
            Puzzle::AutomateFuelSynthesis => self::fuel::INTERFACES,
            Puzzle::AutomateGeigerCounter => self::geiger::INTERFACES,
            Puzzle::AutomateGrapple => self::grapple::INTERFACES,
            Puzzle::AutomateHeliostat => self::heliostat::INTERFACES,
            Puzzle::AutomateIncubator => self::incubator::INTERFACES,
            Puzzle::AutomateInjector => self::injector::INTERFACES,
            Puzzle::AutomateMiningRobot => self::mining::INTERFACES,
            Puzzle::AutomateReactor => self::reactor::INTERFACES,
            Puzzle::AutomateRobotArm => self::robotarm::INTERFACES,
            Puzzle::AutomateSensors => self::sensors::INTERFACES,
            Puzzle::AutomateSonar => self::sonar::INTERFACES,
            Puzzle::AutomateStorageDepot => self::storage::INTERFACES,
            Puzzle::AutomateTranslator => self::translator::INTERFACES,
            Puzzle::AutomateXUnit => self::xunit::INTERFACES,
            Puzzle::CommandLander => self::lander::INTERFACES,
            Puzzle::CommandShields => self::shields::INTERFACES,
            Puzzle::CommandTurret => self::turret::INTERFACES,
            Puzzle::FabricateCounter => self::fab_event::COUNTER_INTERFACES,
            Puzzle::FabricateEggTimer => self::fab_clock::EGG_TIMER_INTERFACES,
            Puzzle::FabricateHalve => self::fab_arith::HALVE_INTERFACES,
            Puzzle::FabricateInc => self::fab_event::INC_INTERFACES,
            Puzzle::FabricateMul => self::fab_arith::MUL_INTERFACES,
            Puzzle::FabricateQueue => self::fab_memory::QUEUE_INTERFACES,
            Puzzle::FabricateStack => self::fab_memory::STACK_INTERFACES,
            Puzzle::FabricateStopwatch => {
                self::fab_clock::STOPWATCH_INTERFACES
            }
            Puzzle::FabricateXor => self::fab_arith::XOR_INTERFACES,
            Puzzle::SandboxBehavior => self::sandbox::BEHAVIOR_INTERFACES,
            Puzzle::SandboxEvent => self::sandbox::EVENT_INTERFACES,
            Puzzle::TutorialAdd => self::tutor_bvr::ADD_INTERFACES,
            Puzzle::TutorialAmp => self::tutor_evt::AMP_INTERFACES,
            Puzzle::TutorialClock => self::tutor_clock::CLOCK_INTERFACES,
            Puzzle::TutorialDemux => self::tutor_evt::DEMUX_INTERFACES,
            Puzzle::TutorialMux => self::tutor_bvr::MUX_INTERFACES,
            Puzzle::TutorialOr => self::tutor_bvr::OR_INTERFACES,
            Puzzle::TutorialRam => self::tutor_ram::RAM_INTERFACES,
            Puzzle::TutorialSum => self::tutor_evt::SUM_INTERFACES,
        }
    }

    fn tutorial_bubbles(
        &self,
    ) -> &'static [(TutorialBubblePosition, &'static str)] {
        match self {
            Puzzle::TutorialAdd => self::tutor_bvr::ADD_BUBBLES,
            Puzzle::TutorialAmp => self::tutor_evt::AMP_BUBBLES,
            Puzzle::TutorialClock => self::tutor_clock::CLOCK_BUBBLES,
            Puzzle::TutorialDemux => self::tutor_evt::DEMUX_BUBBLES,
            Puzzle::TutorialMux => self::tutor_bvr::MUX_BUBBLES,
            Puzzle::TutorialOr => self::tutor_bvr::OR_BUBBLES,
            Puzzle::TutorialRam => self::tutor_ram::RAM_BUBBLES,
            Puzzle::TutorialSum => self::tutor_evt::SUM_BUBBLES,
            _ => &[],
        }
    }
}

fn is_chip_allowed_in(
    ctype: ChipType,
    puzzle: Puzzle,
    solved: &PuzzleSet,
) -> bool {
    if !puzzle.allows_events() && ctype.uses_events() {
        return false;
    }
    match ctype.availibility() {
        ChipAvailability::Always => true,
        ChipAvailability::DiagramOnly => false,
        ChipAvailability::InteractiveOnly => match puzzle.kind() {
            PuzzleKind::Tutorial
            | PuzzleKind::Fabricate
            | PuzzleKind::Automate => false,
            PuzzleKind::Command | PuzzleKind::Sandbox => true,
        },
        ChipAvailability::OnlyIn(puzzles) => puzzles.contains(&puzzle),
        ChipAvailability::StartingWith(other) => {
            puzzle >= other && solved.is_unlocked(other)
        }
        ChipAvailability::StartingWithButNotIn(other, puzzles) => {
            puzzle >= other
                && solved.is_unlocked(other)
                && !puzzles.contains(&puzzle)
        }
        ChipAvailability::UnlockedBy(other) => {
            puzzle > other && solved.is_solved(other)
        }
        ChipAvailability::UnlockedByButNotIn(other, puzzles) => {
            puzzle > other
                && solved.is_solved(other)
                && !puzzles.contains(&puzzle)
        }
        ChipAvailability::UnlockedByButOnlyIn(other, puzzles) => {
            debug_assert!(puzzles.iter().all(|&p| p > other));
            solved.is_solved(other) && puzzles.contains(&puzzle)
        }
    }
}

//===========================================================================//

pub(super) fn new_puzzle_eval(
    puzzle: Puzzle,
    slots: Vec<Vec<((Coords, Direction), usize)>>,
) -> Box<dyn PuzzleEval> {
    match puzzle {
        Puzzle::AutomateBeacon => Box::new(BeaconEval::new(slots)),
        Puzzle::AutomateCollector => Box::new(CollectorEval::new(slots)),
        Puzzle::AutomateDrillingRig => Box::new(DrillingRigEval::new(slots)),
        Puzzle::AutomateFuelSynthesis => Box::new(FuelEval::new(slots)),
        Puzzle::AutomateGeigerCounter => Box::new(GeigerEval::new(slots)),
        Puzzle::AutomateGrapple => Box::new(GrappleEval::new(slots)),
        Puzzle::AutomateHeliostat => Box::new(HeliostatEval::new(slots)),
        Puzzle::AutomateIncubator => Box::new(IncubatorEval::new(slots)),
        Puzzle::AutomateInjector => Box::new(InjectorEval::new(slots)),
        Puzzle::AutomateMiningRobot => Box::new(MiningRobotEval::new(slots)),
        Puzzle::AutomateReactor => {
            Box::new(self::reactor::ReactorEval::new(slots))
        }
        Puzzle::AutomateRobotArm => Box::new(RobotArmEval::new(slots)),
        Puzzle::AutomateSensors => {
            Box::new(self::sensors::SensorsEval::new(slots))
        }
        Puzzle::AutomateSonar => Box::new(SonarEval::new(slots)),
        Puzzle::AutomateStorageDepot => Box::new(StorageDepotEval::new(slots)),
        Puzzle::AutomateTranslator => Box::new(TranslatorEval::new(slots)),
        Puzzle::AutomateXUnit => Box::new(XUnitEval::new(slots)),
        Puzzle::CommandLander => Box::new(LanderEval::new(slots)),
        Puzzle::CommandShields => Box::new(ShieldsEval::new(slots)),
        Puzzle::CommandTurret => Box::new(TurretEval::new(slots)),
        Puzzle::FabricateCounter => {
            Box::new(FabricationEval::new(FABRICATE_COUNTER_DATA, slots))
        }
        Puzzle::FabricateEggTimer => {
            Box::new(FabricationEval::new(FABRICATE_EGG_TIMER_DATA, slots))
        }
        Puzzle::FabricateHalve => {
            Box::new(FabricationEval::new(FABRICATE_HALVE_DATA, slots))
        }
        Puzzle::FabricateInc => {
            Box::new(FabricationEval::new(FABRICATE_INC_DATA, slots))
        }
        Puzzle::FabricateMul => {
            Box::new(FabricationEval::new(FABRICATE_MUL_DATA, slots))
        }
        Puzzle::FabricateQueue => {
            Box::new(FabricationEval::new(FABRICATE_QUEUE_DATA, slots))
        }
        Puzzle::FabricateStack => {
            Box::new(FabricationEval::new(FABRICATE_STACK_DATA, slots))
        }
        Puzzle::FabricateStopwatch => {
            Box::new(FabricationEval::new(FABRICATE_STOPWATCH_DATA, slots))
        }
        Puzzle::FabricateXor => {
            Box::new(FabricationEval::new(FABRICATE_XOR_DATA, slots))
        }
        Puzzle::SandboxBehavior => {
            Box::new(self::sandbox::SandboxBehaviorEval::new(slots))
        }
        Puzzle::SandboxEvent => {
            Box::new(self::sandbox::SandboxEventEval::new(slots))
        }
        Puzzle::TutorialAdd => {
            Box::new(FabricationEval::new(TUTORIAL_ADD_DATA, slots))
        }
        Puzzle::TutorialAmp => {
            Box::new(FabricationEval::new(TUTORIAL_AMP_DATA, slots))
        }
        Puzzle::TutorialClock => {
            Box::new(FabricationEval::new(TUTORIAL_CLOCK_DATA, slots))
        }
        Puzzle::TutorialDemux => {
            Box::new(FabricationEval::new(TUTORIAL_DEMUX_DATA, slots))
        }
        Puzzle::TutorialMux => {
            Box::new(FabricationEval::new(TUTORIAL_MUX_DATA, slots))
        }
        Puzzle::TutorialOr => {
            Box::new(FabricationEval::new(TUTORIAL_OR_DATA, slots))
        }
        Puzzle::TutorialRam => {
            Box::new(FabricationEval::new(TUTORIAL_RAM_DATA, slots))
        }
        Puzzle::TutorialSum => {
            Box::new(FabricationEval::new(TUTORIAL_SUM_DATA, slots))
        }
    }
}

//===========================================================================//
