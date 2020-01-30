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

use super::puzzle::Puzzle;
use std::collections::HashMap;

//===========================================================================//

pub struct PuzzleSet {
    solved: HashMap<Puzzle, bool>,
}

impl PuzzleSet {
    pub fn new() -> PuzzleSet {
        let mut solved = HashMap::new();
        solved.insert(Puzzle::first(), false);
        PuzzleSet { solved }
    }

    pub fn with_everything_solved() -> PuzzleSet {
        PuzzleSet { solved: Puzzle::all().map(|p| (p, true)).collect() }
    }

    pub fn is_unlocked(&self, puzzle: Puzzle) -> bool {
        self.solved.contains_key(&puzzle)
    }

    pub fn is_solved(&self, puzzle: Puzzle) -> bool {
        self.solved.get(&puzzle) == Some(&true)
    }

    pub fn unlock(&mut self, puzzle: Puzzle) {
        self.solved.entry(puzzle).or_insert(false);
    }

    pub fn solve(&mut self, puzzle: Puzzle) {
        self.solved.insert(puzzle, true);
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::super::puzzle::Puzzle;
    use super::PuzzleSet;

    #[test]
    fn first_puzzle_is_always_unlocked() {
        let puzzles = PuzzleSet::new();
        assert!(puzzles.is_unlocked(Puzzle::first()));
    }

    #[test]
    fn new_puzzle_set_has_nothing_solved() {
        let puzzles = PuzzleSet::new();
        for puzzle in Puzzle::all() {
            assert!(!puzzles.is_solved(puzzle));
        }
    }

    #[test]
    fn puzzle_set_with_everything_solved() {
        let puzzles = PuzzleSet::with_everything_solved();
        for puzzle in Puzzle::all() {
            assert!(puzzles.is_unlocked(puzzle));
            assert!(puzzles.is_solved(puzzle));
        }
    }
}

//===========================================================================//
