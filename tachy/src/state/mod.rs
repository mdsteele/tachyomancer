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

mod change;
mod check;
mod chip;
mod edit;
mod eval;
mod interface;
mod port;
mod puzzle;
mod verify;

pub use self::change::GridChange;
pub use self::check::{
    detect_loops, determine_wire_sizes, group_wires, map_ports_to_wires,
    recolor_wires, WireColor, WireError, WireId, WireInfo,
};
pub use self::chip::ChipExt;
pub use self::edit::{ChipsIter, EditGrid, WireFragmentsIter};
pub use self::eval::{CircuitEval, EvalError, EvalResult};
pub use self::interface::{Interface, InterfacePort};
pub use self::port::{
    PortColor, PortConstraint, PortDependency, PortFlow, PortSpec,
};
pub use self::puzzle::*;
pub use self::verify::verify_solution;

//===========================================================================//
