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

mod check;
mod chip;
mod converse;
mod edit;
mod eval;
mod game;
mod port;
mod puzzle;
mod size;

pub use self::check::WireColor;
pub use self::chip::ChipExt;
pub use self::converse::{ConversationBubble, ConversationPortrait};
pub use self::edit::{ChipsIter, EditGrid, GridChange, WireFragmentsIter};
pub use self::eval::{CircuitEval, EvalError, EvalResult, EvalScore};
pub use self::game::GameState;
pub use self::port::{PortColor, PortFlow, PortSpec};
pub use self::puzzle::{HeliostatEval, Interface, RobotArmEval,
                       TutorialOrEval, TutorialXorEval};
pub use self::size::WireSize;

//===========================================================================//
