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

use super::super::eval::EvalError;
use crate::geom::{Coords, Direction};
use std::u32;
use std::u64;

//===========================================================================/

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TutorialBubblePosition {
    Bounds(Direction),
    ControlsTray,
    PartsTray,
}

//===========================================================================//

pub fn u64_to_opt_u32(value: u64) -> Option<u32> {
    if value <= (u32::MAX as u64) {
        Some(value as u32)
    } else {
        None
    }
}

pub fn opt_u32_to_u64(opt_value: Option<u32>) -> u64 {
    if let Some(value) = opt_value {
        value as u64
    } else {
        u64::MAX
    }
}

//===========================================================================//

pub fn end_cycle_check_event_output(
    opt_actual: Option<u32>,
    opt_expected: u64,
    has_received: &mut bool,
    port: (Coords, Direction),
    time_step: u32,
    errors_out: &mut Vec<EvalError>,
) {
    if let Some(actual) = opt_actual {
        if let Some(expected) = u64_to_opt_u32(opt_expected) {
            if *has_received {
                let error = EvalError {
                    time_step,
                    port: Some(port),
                    message: format!(
                        "Expected only one output event, \
                         but got more than one"
                    ),
                };
                errors_out.push(error);
            } else if actual != expected {
                let error = EvalError {
                    time_step,
                    port: Some(port),
                    message: format!(
                        "Expected output event value of {}, \
                         but got output event value of {}",
                        expected, actual
                    ),
                };
                errors_out.push(error);
            }
        } else {
            let error = EvalError {
                time_step,
                port: Some(port),
                message: format!(
                    "No output event expected, but got \
                     output event value of {}",
                    actual
                ),
            };
            errors_out.push(error);
        }
        *has_received = true;
    }
}

pub fn end_time_step_check_event_output(
    opt_expected: u64,
    has_received: bool,
    port: (Coords, Direction),
    time_step: u32,
    errors_out: &mut Vec<EvalError>,
) {
    if !has_received {
        if let Some(expected) = u64_to_opt_u32(opt_expected) {
            let error = EvalError {
                time_step,
                port: Some(port),
                message: format!(
                    "Expected output event value of {}, but \
                     got no output event",
                    expected
                ),
            };
            errors_out.push(error);
        }
    }
}

//===========================================================================//
