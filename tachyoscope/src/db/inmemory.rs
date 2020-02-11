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

use super::shared::{hash_circuit_data, ScoreDatabase};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tachy::save::{CircuitData, Puzzle, ScoreCurveMap, SolutionData};

//===========================================================================//

pub struct InMemoryScoreDatabase {
    storage: Mutex<Storage>,
}

struct Storage {
    circuits: HashMap<String, CircuitData>,
    submissions: HashMap<(u64, Puzzle, String), Instant>,
    verified: HashMap<(Puzzle, String), (i32, u32)>,
}

impl InMemoryScoreDatabase {
    pub fn new() -> InMemoryScoreDatabase {
        InMemoryScoreDatabase {
            storage: Mutex::new(Storage {
                circuits: HashMap::new(),
                submissions: HashMap::new(),
                verified: HashMap::new(),
            }),
        }
    }
}

impl ScoreDatabase for InMemoryScoreDatabase {
    fn load_num_verified_solutions(&self) -> Result<u64, String> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.verified.len() as u64)
    }

    fn load_scores(&self) -> Result<ScoreCurveMap, String> {
        let mut scores = ScoreCurveMap::new();
        let storage = self.storage.lock().unwrap();
        for (&(puzzle, _), &(area, score)) in storage.verified.iter() {
            scores.insert(puzzle, area, score);
        }
        Ok(scores)
    }

    fn store_new_solution(
        &self,
        solution: &SolutionData,
    ) -> Result<Option<(Puzzle, String)>, String> {
        let hash = hash_circuit_data(&solution.circuit)?;
        debug_log!("Solution hash is {}", hash);
        let mut storage = self.storage.lock().unwrap();
        if !storage.circuits.contains_key(&hash) {
            storage.circuits.insert(hash.clone(), solution.circuit.clone());
        }
        if let Some(id) = solution.install_id {
            storage
                .submissions
                .entry((id, solution.puzzle, hash.clone()))
                .or_insert_with(Instant::now);
        }
        let key = (solution.puzzle, hash);
        if storage.verified.contains_key(&key) {
            return Ok(None);
        }
        return Ok(Some(key));
    }

    fn store_verified_solution(
        &self,
        key: (Puzzle, String),
        area: i32,
        score: u32,
    ) -> Result<(), String> {
        let mut storage = self.storage.lock().unwrap();
        storage.verified.insert(key, (area, score));
        Ok(())
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::super::shared::ScoreDatabase;
    use super::InMemoryScoreDatabase;
    use tachy::save::{Puzzle, SolutionData};

    #[test]
    fn num_verified_solutions() {
        let db = InMemoryScoreDatabase::new();
        assert_eq!(db.load_num_verified_solutions().unwrap(), 0);

        // Store a solution.  It should not count as verified until we have
        // actually verified it.
        let solution =
            SolutionData::load("tests/solutions/tutorial_or_1.toml").unwrap();
        let key = db.store_new_solution(&solution).unwrap();
        assert!(key.is_some());
        assert_eq!(db.load_num_verified_solutions().unwrap(), 0);
        db.store_verified_solution(
            key.unwrap(),
            solution.circuit.size.area(),
            solution.score,
        )
        .unwrap();
        assert_eq!(db.load_num_verified_solutions().unwrap(), 1);

        // Store another solution.  Again, it should not count as verified
        // until we have actually verified it.
        let solution =
            SolutionData::load("tests/solutions/tutorial_or_2.toml").unwrap();
        let key = db.store_new_solution(&solution).unwrap();
        assert!(key.is_some());
        assert_eq!(db.load_num_verified_solutions().unwrap(), 1);
        db.store_verified_solution(
            key.unwrap(),
            solution.circuit.size.area(),
            solution.score,
        )
        .unwrap();
        assert_eq!(db.load_num_verified_solutions().unwrap(), 2);

        // Try storing the same solution again.  It should be rejected as a
        // duplicate.
        assert!(db.store_new_solution(&solution).unwrap().is_none());
        assert_eq!(db.load_num_verified_solutions().unwrap(), 2);

        // Load scores and check that we have both scores for TutorialOr.
        let score_map = db.load_scores().unwrap();
        let score_curve = score_map.get(Puzzle::TutorialOr);
        assert_eq!(score_curve.scores(), &[(8, 16), (9, 12)]);
    }
}

//===========================================================================//
