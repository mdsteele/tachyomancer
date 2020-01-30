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

use crate::save::{PuzzleSet, SolutionData};
use crate::state::{EditGrid, EvalResult, WireError};

//===========================================================================//

pub fn verify_solution(data: &SolutionData) -> Vec<String> {
    let mut grid = EditGrid::from_circuit_data(
        data.puzzle,
        &PuzzleSet::with_everything_solved(),
        &data.circuit,
    );
    let mut errors = Vec::<String>::new();
    if !grid.start_eval() {
        for error in grid.errors() {
            errors.push(match error {
                WireError::MultipleSenders(idx) => {
                    format!("Wire {} has multiple senders", idx)
                }
                WireError::PortColorMismatch(idx) => {
                    format!("Wire {} has a color mismatch", idx)
                }
                WireError::NoValidSize(idx) => {
                    format!("Wire {} has a size mismatch", idx)
                }
                WireError::UnbrokenLoop(idxs, _) => {
                    format!("Wires {:?} form a loop", idxs)
                }
            });
        }
        errors.push("Circuit had errors".to_string());
    } else {
        let eval = grid.eval_mut().unwrap();
        eval.set_breakpoints_enabled(false);
        for time_step in 0..(data.time_steps + 1) {
            match eval.step_time() {
                EvalResult::Continue if time_step < data.time_steps => {}
                EvalResult::Continue => {
                    errors.push(format!(
                        "Evaluation did not end at time step {}",
                        time_step
                    ));
                    break;
                }
                EvalResult::Breakpoint(_) => {
                    unreachable!("Breakpoints were disabled.");
                }
                EvalResult::Failure => {
                    errors.extend(eval.errors().iter().map(|error| {
                        format!(
                            "Time step {}: {}",
                            error.time_step, error.message
                        )
                    }));
                    break;
                }
                EvalResult::Victory(score) if time_step < data.time_steps => {
                    errors.push(format!(
                        "Unexpected victory at time step {} with score of {}",
                        time_step, score
                    ));
                    break;
                }
                EvalResult::Victory(score) => {
                    if score != data.score {
                        errors.push(format!(
                            "Actual score was {}, but expected {}",
                            score, data.score
                        ));
                    }
                }
            }
        }
    }
    errors
}

//===========================================================================//
