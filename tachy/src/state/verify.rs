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

use crate::geom::Coords;
use crate::save::{PuzzleSet, SolutionData};
use crate::state::{EditGrid, EvalResult, WireError};
use std::collections::HashMap;

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
                WireError::MultipleSenders(id) => {
                    format!("Wire {} has multiple senders", id.0)
                }
                WireError::PortColorMismatch(id) => {
                    format!("Wire {} has a color mismatch", id.0)
                }
                WireError::NoValidSize(id) => {
                    format!("Wire {} has a size mismatch", id.0)
                }
                WireError::UnbrokenLoop(ids, _) => {
                    format!("Wires {:?} form a loop", ids)
                }
            });
        }
        errors.push("Circuit had errors".to_string());
        return errors;
    }

    let mut all_inputs = HashMap::<(u32, u32), Vec<(Coords, u32, u32)>>::new();
    if let Some(ref inputs) = data.inputs {
        let origin = grid.bounds().top_left();
        for (time_step, cycle, delta, subloc, count) in inputs.iter() {
            all_inputs
                .entry((time_step, cycle))
                .or_insert_with(Vec::new)
                .push((origin + delta, subloc, count));
        }
    }

    let eval = grid.eval_mut().unwrap();
    loop {
        let time_step = eval.time_step();
        if let Some(inputs) = all_inputs.remove(&(time_step, eval.cycle())) {
            for (coords, subloc, count) in inputs {
                eval.press_button(coords, subloc, count);
            }
        }
        match eval.step_cycle() {
            EvalResult::Continue => {
                if time_step >= data.time_steps {
                    errors.push(format!(
                        "Evaluation did not end at time step {}",
                        data.time_steps
                    ));
                    break;
                }
            }
            EvalResult::Breakpoint(_) => {}
            EvalResult::Failure => {
                errors.extend(eval.errors().iter().map(|error| {
                    format!("Time step {}: {}", error.time_step, error.message)
                }));
                break;
            }
            EvalResult::Victory(score) => {
                if time_step < data.time_steps {
                    errors.push(format!(
                        "Unexpected victory at time step {} with score of {}",
                        time_step, score
                    ));
                } else if score != data.score {
                    errors.push(format!(
                        "Actual score was {}, but expected {}",
                        score, data.score
                    ));
                }
                break;
            }
        }
    }
    errors
}

//===========================================================================//
