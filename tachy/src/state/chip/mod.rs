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

mod arith;
mod compare;
mod data;
mod event;
mod logic;
mod special;
mod value;

use self::data::{localize, AbstractConstraint, ChipData};
use super::eval::{ChipEval, CircuitInteraction};
use super::port::{
    PortColor, PortConstraint, PortDependency, PortFlow, PortSpec,
};
use super::size::WireSize;
use crate::geom::{Coords, Orientation};
use crate::save::{ChipType, Puzzle};
use cgmath::Bounded;
use std::cell::RefCell;
use std::rc::Rc;

//===========================================================================//

pub enum ChipAvailability {
    Always,
    InteractiveOnly,
    OnlyIn(&'static [Puzzle]),
    StartingWith(Puzzle),
    UnlockedBy(Puzzle),
}

//===========================================================================//

pub trait ChipExt {
    fn uses_events(&self) -> bool;

    fn availibility(&self) -> ChipAvailability;

    fn ports(&self, coords: Coords, orient: Orientation) -> Vec<PortSpec>;

    fn constraints(
        &self,
        coords: Coords,
        orient: Orientation,
    ) -> Vec<PortConstraint>;

    fn dependencies(
        &self,
        coords: Coords,
        orient: Orientation,
    ) -> Vec<PortDependency>;
}

impl ChipExt for ChipType {
    fn uses_events(&self) -> bool {
        chip_data(*self)
            .ports
            .iter()
            .any(|&(_, color, _, _)| color == PortColor::Event)
    }

    fn availibility(&self) -> ChipAvailability {
        match *self {
            ChipType::Const(_)
            | ChipType::Display
            | ChipType::Not
            | ChipType::And => ChipAvailability::Always,
            ChipType::Or => ChipAvailability::UnlockedBy(Puzzle::TutorialOr),
            ChipType::Xor => {
                ChipAvailability::UnlockedBy(Puzzle::FabricateXor)
            }
            ChipType::Mux => ChipAvailability::UnlockedBy(Puzzle::TutorialMux),
            ChipType::Pack | ChipType::Unpack => {
                ChipAvailability::StartingWith(Puzzle::TutorialAdd)
            }
            ChipType::Add2Bit => {
                ChipAvailability::OnlyIn(&[Puzzle::TutorialAdd])
            }
            ChipType::Add | ChipType::Sub => {
                ChipAvailability::UnlockedBy(Puzzle::TutorialAdd)
            }
            ChipType::Halve => {
                ChipAvailability::UnlockedBy(Puzzle::FabricateHalve)
            }
            ChipType::Mul4Bit => {
                ChipAvailability::OnlyIn(&[Puzzle::FabricateMul])
            }
            ChipType::Mul => {
                ChipAvailability::UnlockedBy(Puzzle::FabricateMul)
            }
            ChipType::Cmp | ChipType::CmpEq | ChipType::Eq => {
                ChipAvailability::StartingWith(Puzzle::AutomateHeliostat)
            }
            ChipType::Filter => {
                ChipAvailability::OnlyIn(&[Puzzle::TutorialDemux])
            }
            ChipType::Break
            | ChipType::Clock
            | ChipType::Delay
            | ChipType::Demux
            | ChipType::Discard
            | ChipType::Join
            | ChipType::Latest
            | ChipType::Ram
            | ChipType::Sample => {
                ChipAvailability::UnlockedBy(Puzzle::TutorialDemux)
            }
            ChipType::Inc => {
                ChipAvailability::UnlockedBy(Puzzle::FabricateInc)
            }
            ChipType::Button | ChipType::Toggle(_) => {
                ChipAvailability::InteractiveOnly
            }
        }
    }

    fn ports(&self, coords: Coords, orient: Orientation) -> Vec<PortSpec> {
        let size = self.size();
        let data = chip_data(*self);
        data.ports
            .iter()
            .enumerate()
            .map(|(index, &(flow, color, delta, dir))| {
                let mut max_size = WireSize::max_value();
                for constraint in data.constraints.iter() {
                    match *constraint {
                        AbstractConstraint::Exact(i, s) if i == index => {
                            max_size = s;
                            break;
                        }
                        AbstractConstraint::AtMost(i, s) if i == index => {
                            max_size = max_size.min(s);
                        }
                        AbstractConstraint::Double(_, i) if i == index => {
                            max_size =
                                max_size.min(WireSize::max_value().half());
                        }
                        _ => {}
                    }
                }
                PortSpec {
                    flow,
                    color,
                    coords: coords
                        + orient.transform_in_size(delta.into(), size),
                    dir: orient * dir,
                    max_size,
                }
            })
            .collect()
    }

    fn constraints(
        &self,
        coords: Coords,
        orient: Orientation,
    ) -> Vec<PortConstraint> {
        let size = self.size();
        let data = chip_data(*self);
        data.constraints
            .iter()
            .map(|constraint| {
                constraint.reify(coords, orient, size, data.ports)
            })
            .collect()
    }

    fn dependencies(
        &self,
        coords: Coords,
        orient: Orientation,
    ) -> Vec<PortDependency> {
        let size = self.size();
        let data = chip_data(*self);
        data.dependencies
            .iter()
            .map(|&(recv_index, send_index)| {
                let recv_port = &data.ports[recv_index];
                let send_port = &data.ports[send_index];
                debug_assert_eq!(recv_port.0, PortFlow::Recv);
                debug_assert_eq!(send_port.0, PortFlow::Send);
                PortDependency {
                    recv: localize(coords, orient, size, recv_port),
                    send: localize(coords, orient, size, send_port),
                }
            })
            .collect()
    }
}

fn chip_data(ctype: ChipType) -> &'static ChipData {
    match ctype {
        ChipType::Add => self::arith::ADD_CHIP_DATA,
        ChipType::Add2Bit => self::arith::ADD_2BIT_CHIP_DATA,
        ChipType::And => self::logic::AND_CHIP_DATA,
        ChipType::Break => self::special::BREAK_CHIP_DATA,
        ChipType::Button => self::special::BUTTON_CHIP_DATA,
        ChipType::Clock => self::event::CLOCK_CHIP_DATA,
        ChipType::Cmp => self::compare::CMP_CHIP_DATA,
        ChipType::CmpEq => self::compare::CMPEQ_CHIP_DATA,
        ChipType::Const(value) => self::value::const_chip_data(value),
        ChipType::Delay => self::event::DELAY_CHIP_DATA,
        ChipType::Demux => self::event::DEMUX_CHIP_DATA,
        ChipType::Discard => self::event::DISCARD_CHIP_DATA,
        ChipType::Display => self::special::DISPLAY_CHIP_DATA,
        ChipType::Eq => self::compare::EQ_CHIP_DATA,
        ChipType::Filter => self::event::FILTER_CHIP_DATA,
        ChipType::Halve => self::arith::HALVE_CHIP_DATA,
        ChipType::Inc => self::event::INC_CHIP_DATA,
        ChipType::Join => self::event::JOIN_CHIP_DATA,
        ChipType::Latest => self::event::LATEST_CHIP_DATA,
        ChipType::Mul => self::arith::MUL_CHIP_DATA,
        ChipType::Mul4Bit => self::arith::MUL_4BIT_CHIP_DATA,
        ChipType::Mux => self::logic::MUX_CHIP_DATA,
        ChipType::Not => self::logic::NOT_CHIP_DATA,
        ChipType::Or => self::logic::OR_CHIP_DATA,
        ChipType::Pack => self::value::PACK_CHIP_DATA,
        ChipType::Ram => self::special::RAM_CHIP_DATA,
        ChipType::Sample => self::event::SAMPLE_CHIP_DATA,
        ChipType::Sub => self::arith::SUB_CHIP_DATA,
        ChipType::Toggle(_) => self::special::TOGGLE_CHIP_DATA,
        ChipType::Unpack => self::value::UNPACK_CHIP_DATA,
        ChipType::Xor => self::logic::XOR_CHIP_DATA,
    }
}

//===========================================================================//

pub(super) fn new_chip_evals(
    ctype: ChipType,
    coords: Coords,
    slots: &[(usize, WireSize)],
    interact: &Rc<RefCell<CircuitInteraction>>,
) -> Vec<(usize, Box<dyn ChipEval>)> {
    debug_assert_eq!(slots.len(), chip_data(ctype).ports.len());
    match ctype {
        ChipType::Add => self::arith::AddChipEval::new_evals(slots),
        ChipType::Add2Bit => self::arith::Add2BitChipEval::new_evals(slots),
        ChipType::And => self::logic::AndChipEval::new_evals(slots),
        ChipType::Break => {
            self::special::BreakChipEval::new_evals(slots, coords)
        }
        ChipType::Button => self::special::ButtonChipEval::new_evals(
            slots,
            coords,
            interact.clone(),
        ),
        ChipType::Clock => self::event::ClockChipEval::new_evals(slots),
        ChipType::Cmp => self::compare::CmpChipEval::new_evals(slots),
        ChipType::CmpEq => self::compare::CmpEqChipEval::new_evals(slots),
        ChipType::Const(value) => {
            self::value::ConstChipEval::new_evals(value, slots)
        }
        ChipType::Delay => self::event::DelayChipEval::new_evals(slots),
        ChipType::Demux => self::event::DemuxChipEval::new_evals(slots),
        ChipType::Discard => self::event::DiscardChipEval::new_evals(slots),
        ChipType::Display => vec![],
        ChipType::Eq => self::compare::EqChipEval::new_evals(slots),
        ChipType::Filter => self::event::FilterChipEval::new_evals(slots),
        ChipType::Halve => self::arith::HalveChipEval::new_evals(slots),
        ChipType::Inc => self::event::IncChipEval::new_evals(slots),
        ChipType::Join => self::event::JoinChipEval::new_evals(slots),
        ChipType::Latest => self::event::LatestChipEval::new_evals(slots),
        ChipType::Mul => self::arith::MulChipEval::new_evals(slots),
        ChipType::Mul4Bit => self::arith::Mul4BitChipEval::new_evals(slots),
        ChipType::Mux => self::logic::MuxChipEval::new_evals(slots),
        ChipType::Not => self::logic::NotChipEval::new_evals(slots),
        ChipType::Or => self::logic::OrChipEval::new_evals(slots),
        ChipType::Pack => self::value::PackChipEval::new_evals(slots),
        ChipType::Ram => self::special::RamChipEval::new_evals(slots),
        ChipType::Sample => self::event::SampleChipEval::new_evals(slots),
        ChipType::Sub => self::arith::SubChipEval::new_evals(slots),
        ChipType::Toggle(value) => self::special::ToggleChipEval::new_evals(
            value,
            slots,
            coords,
            interact.clone(),
        ),
        ChipType::Unpack => self::value::UnpackChipEval::new_evals(slots),
        ChipType::Xor => self::logic::XorChipEval::new_evals(slots),
    }
}

//===========================================================================//