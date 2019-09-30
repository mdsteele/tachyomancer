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

use super::circuit::CircuitData;
use super::puzzle::Puzzle;
use std::fs;
use std::path::Path;
use toml;

//===========================================================================//

#[derive(Deserialize, Serialize)]
pub struct SolutionData {
    pub puzzle: Puzzle,
    pub score: u32,
    pub time_steps: u32,
    pub circuit: CircuitData,
}

impl SolutionData {
    pub fn load(path: &Path) -> Result<SolutionData, String> {
        let bytes = fs::read(path).map_err(|err| {
            format!("Could not read solution file from {:?}: {}", path, err)
        })?;
        toml::from_slice(&bytes)
            .map_err(|err| format!("Could not deserialize solution: {}", err))
    }

    pub fn deserialize_from_string(
        string: &str,
    ) -> Result<SolutionData, String> {
        toml::from_str(string)
            .map_err(|err| format!("Could not deserialize solution: {}", err))
    }

    pub fn serialize_to_string(&self) -> Result<String, String> {
        toml::to_string(self)
            .map_err(|err| format!("Could not serialize solution: {}", err))
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{CircuitData, Puzzle, SolutionData};

    #[test]
    fn serialize_solution_data() {
        let solution = SolutionData {
            puzzle: Puzzle::TutorialOr,
            score: 14,
            time_steps: 4,
            circuit: CircuitData::new(4, 3),
        };
        let string = solution.serialize_to_string().unwrap();
        assert_eq!(
            string.as_str(),
            "puzzle = \"TutorialOr\"\n\
             score = 14\n\
             time_steps = 4\n\n\
             [circuit]\n\
             size = [4, 3]\n\n\
             [circuit.chips]\n\n\
             [circuit.wires]\n"
        );
    }
}

//===========================================================================//
