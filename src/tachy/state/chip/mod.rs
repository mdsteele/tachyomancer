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

use self::data::{ChipData, localize};
use super::eval::{ChipEval, CircuitInteraction};
use super::port::{PortConstraint, PortDependency, PortFlow, PortSpec};
use super::size::WireSize;
use std::cell::RefCell;
use std::rc::Rc;
use tachy::geom::{Coords, Orientation};
use tachy::save::ChipType;

//===========================================================================//

pub trait ChipExt {
    fn ports(&self, coords: Coords, orient: Orientation) -> Vec<PortSpec>;
    fn constraints(&self, coords: Coords, orient: Orientation)
                   -> Vec<PortConstraint>;
    fn dependencies(&self, coords: Coords, orient: Orientation)
                    -> Vec<PortDependency>;
}

impl ChipExt for ChipType {
    fn ports(&self, coords: Coords, orient: Orientation) -> Vec<PortSpec> {
        let size = self.size();
        chip_data(*self)
            .ports
            .iter()
            .map(|&(flow, color, delta, dir)| {
                PortSpec {
                    flow,
                    color,
                    coords: coords +
                        orient.transform_in_size(delta.into(), size),
                    dir: orient * dir,
                }
            })
            .collect()
    }

    fn constraints(&self, coords: Coords, orient: Orientation)
                   -> Vec<PortConstraint> {
        let size = self.size();
        let data = chip_data(*self);
        data.constraints
            .iter()
            .map(|constraint| {
                     constraint.reify(coords, orient, size, data.ports)
                 })
            .collect()
    }

    fn dependencies(&self, coords: Coords, orient: Orientation)
                    -> Vec<PortDependency> {
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
        ChipType::And => self::logic::AND_CHIP_DATA,
        ChipType::Break => self::special::BREAK_CHIP_DATA,
        ChipType::Button => self::special::BUTTON_CHIP_DATA,
        ChipType::Clock => self::event::CLOCK_CHIP_DATA,
        ChipType::Cmp => self::compare::CMP_CHIP_DATA,
        ChipType::CmpEq => self::compare::CMPEQ_CHIP_DATA,
        ChipType::Const(value) => self::value::const_chip_data(value),
        ChipType::Delay => self::event::DELAY_CHIP_DATA,
        ChipType::Discard => self::event::DISCARD_CHIP_DATA,
        ChipType::Display => self::special::DISPLAY_CHIP_DATA,
        ChipType::Eq => self::compare::EQ_CHIP_DATA,
        ChipType::Join => self::event::JOIN_CHIP_DATA,
        ChipType::Latest => self::event::LATEST_CHIP_DATA,
        ChipType::Mul => self::arith::MUL_CHIP_DATA,
        ChipType::Mux => self::logic::MUX_CHIP_DATA,
        ChipType::Not => self::logic::NOT_CHIP_DATA,
        ChipType::Or => self::logic::OR_CHIP_DATA,
        ChipType::Pack => self::value::PACK_CHIP_DATA,
        ChipType::Ram => self::special::RAM_CHIP_DATA,
        ChipType::Sample => self::event::SAMPLE_CHIP_DATA,
        ChipType::Sub => self::arith::SUB_CHIP_DATA,
        ChipType::Unpack => self::value::UNPACK_CHIP_DATA,
    }
}

//===========================================================================//

pub(super) fn new_chip_evals(ctype: ChipType, coords: Coords,
                             slots: &[(usize, WireSize)],
                             interact: &Rc<RefCell<CircuitInteraction>>)
                             -> Vec<(usize, Box<ChipEval>)> {
    debug_assert_eq!(slots.len(), chip_data(ctype).ports.len());
    match ctype {
        ChipType::Add => self::arith::AddChipEval::new_evals(slots),
        ChipType::And => self::logic::AndChipEval::new_evals(slots),
        ChipType::Break => {
            self::special::BreakChipEval::new_evals(slots, coords)
        }
        ChipType::Button => {
            self::special::ButtonChipEval::new_evals(slots,
                                                     coords,
                                                     interact.clone())
        }
        ChipType::Clock => self::event::ClockChipEval::new_evals(slots),
        ChipType::Cmp => self::compare::CmpChipEval::new_evals(slots),
        ChipType::CmpEq => self::compare::CmpEqChipEval::new_evals(slots),
        ChipType::Const(value) => {
            self::value::ConstChipEval::new_evals(value, slots)
        }
        ChipType::Delay => self::event::DelayChipEval::new_evals(slots),
        ChipType::Discard => self::event::DiscardChipEval::new_evals(slots),
        ChipType::Display => vec![],
        ChipType::Eq => self::compare::EqChipEval::new_evals(slots),
        ChipType::Join => self::event::JoinChipEval::new_evals(slots),
        ChipType::Latest => self::event::LatestChipEval::new_evals(slots),
        ChipType::Mul => self::arith::MulChipEval::new_evals(slots),
        ChipType::Mux => self::logic::MuxChipEval::new_evals(slots),
        ChipType::Not => self::logic::NotChipEval::new_evals(slots),
        ChipType::Or => self::logic::OrChipEval::new_evals(slots),
        ChipType::Pack => self::value::PackChipEval::new_evals(slots),
        ChipType::Ram => self::special::RamChipEval::new_evals(slots),
        ChipType::Sample => self::event::SampleChipEval::new_evals(slots),
        ChipType::Sub => self::arith::SubChipEval::new_evals(slots),
        ChipType::Unpack => self::value::UnpackChipEval::new_evals(slots),
    }
}

//===========================================================================//
