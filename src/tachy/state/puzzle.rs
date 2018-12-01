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

use super::eval::{CircuitState, NullPuzzleEval, PuzzleEval};
use tachy::save::Puzzle;

//===========================================================================//

pub fn new_puzzle_eval(puzzle: Puzzle, slots: Vec<Vec<usize>>)
                       -> Box<PuzzleEval> {
    match puzzle {
        Puzzle::SandboxEvent => Box::new(SandboxEventEval::new(slots)),
        _ => Box::new(NullPuzzleEval()), // TODO other puzzles
    }
}

//===========================================================================//

struct SandboxEventEval {
    metronome_esend: usize,
    timer_bsend: usize,
}

impl SandboxEventEval {
    fn new(slots: Vec<Vec<usize>>) -> SandboxEventEval {
        debug_assert_eq!(slots.len(), 1);
        debug_assert_eq!(slots[0].len(), 2);
        SandboxEventEval {
            metronome_esend: slots[0][0],
            timer_bsend: slots[0][1],
        }
    }
}

impl PuzzleEval for SandboxEventEval {
    fn begin_time_step(&mut self, time_step: u32, state: &mut CircuitState)
                       -> bool {
        state.values[self.metronome_esend] = (0, true);
        state.values[self.timer_bsend] = (time_step % 0xff, true);
        true
    }

    fn end_time_step(&mut self, _state: &CircuitState) -> Option<i32> { None }
}

//===========================================================================//
