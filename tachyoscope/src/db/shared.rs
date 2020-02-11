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
use sha::utils::{Digest, DigestExt};
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
    let serialized = circuit.serialize_to_string()?;
    Ok(Sha256::default().digest(serialized.as_bytes()).to_hex())
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::hash_circuit_data;
    use tachy::save::SolutionData;

    #[test]
    fn circuit_hash_1() {
        let solution =
            SolutionData::load("tests/solutions/tutorial_or_1.toml").unwrap();
        let hash = hash_circuit_data(&solution.circuit).unwrap();
        assert_eq!(
            hash,
            "f38b0da4b7a01eeb75cfc2698efb7dbd\
             895b03bd971cc5170aecb0220a0724d8"
        );
    }

    #[test]
    fn circuit_hash_2() {
        let solution =
            SolutionData::load("tests/solutions/tutorial_or_2.toml").unwrap();
        let hash = hash_circuit_data(&solution.circuit).unwrap();
        assert_eq!(
            hash,
            "9af25154810a935af30a2107019b4d5b\
             c2a62b99e95c82607bbb8c574c22a2d2"
        );
    }
}

//===========================================================================//
