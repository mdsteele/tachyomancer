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

use sha::sha256::Sha256;
use sha::utils::DigestExt;
use std::hash::Hasher;
use tachy::save::{CircuitData, Puzzle, ScoreCurveMap, SolutionData};

//===========================================================================//

pub trait ScoreDatabase: Send + Sync {
    /// Returns the number of distinct verified solutions in the database.
    fn load_num_verified_solutions(&self) -> Result<u64, String>;

    /// Returns the global scores.
    fn load_scores(&self) -> Result<ScoreCurveMap, String>;

    /// Returns a key to be passed to `store_verified_solution` if the solution
    /// is new and needs to be verified, otherwise returns `None`.
    fn store_new_solution(
        &self,
        solution: &SolutionData,
    ) -> Result<Option<(Puzzle, String)>, String>;

    /// Record that the specified solution was valid (and resulted in the given
    /// score).
    fn store_verified_solution(
        &self,
        key: (Puzzle, String),
        area: i32,
        score: u32,
    ) -> Result<(), String>;
}

//===========================================================================//

pub fn hash_circuit_data(circuit: &CircuitData) -> Result<String, String> {
    let mut hash = Sha256::default();
    let serialized = circuit.serialize_to_string()?;
    hash.write(serialized.as_bytes());
    Ok(hash.to_hex())
}

//===========================================================================//
