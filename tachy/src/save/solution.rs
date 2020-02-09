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

use super::circuit::{delta_key_string, key_string_delta, CircuitData};
use super::puzzle::Puzzle;
use crate::geom::CoordsDelta;
use serde::de::Error;
use std::collections::{btree_map, BTreeMap};
use std::fs;
use std::path::Path;
use toml;

//===========================================================================//

#[derive(Deserialize, Serialize)]
pub struct SolutionData {
    pub install_id: Option<u64>,
    pub puzzle: Puzzle,
    pub score: u32,
    pub time_steps: u32,
    pub circuit: CircuitData,
    pub inputs: Option<InputsData>,
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

#[derive(Clone)]
pub struct InputsData(BTreeMap<(u32, u32, i32, i32, u32), u32>);

impl InputsData {
    pub fn new() -> InputsData {
        InputsData(BTreeMap::new())
    }

    pub fn insert(
        &mut self,
        time_step: u32,
        cycle: u32,
        delta: CoordsDelta,
        subloc: u32,
        count: u32,
    ) {
        *self
            .0
            .entry((time_step, cycle, delta.x, delta.y, subloc))
            .or_insert(0) += count;
    }

    pub fn iter(&self) -> InputsDataIter {
        InputsDataIter { inner: self.0.iter() }
    }
}

impl<'d> serde::Deserialize<'d> for InputsData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let map = BTreeMap::<&str, u32>::deserialize(deserializer)?;
        let mut buttons = BTreeMap::new();
        for (key, count) in map.into_iter() {
            let tuple = key_string_button(key).ok_or_else(|| {
                D::Error::custom(format!("Invalid button key: {:?}", key))
            })?;
            buttons.insert(tuple, count);
        }
        Ok(InputsData(buttons))
    }
}

impl serde::Serialize for InputsData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0
            .iter()
            .map(|(&(time_step, cycle, x, y, subloc), &count)| {
                (button_key_string(time_step, cycle, x, y, subloc), count)
            })
            .collect::<BTreeMap<String, u32>>()
            .serialize(serializer)
    }
}

pub struct InputsDataIter<'a> {
    inner: btree_map::Iter<'a, (u32, u32, i32, i32, u32), u32>,
}

impl<'a> Iterator for InputsDataIter<'a> {
    type Item = (u32, u32, CoordsDelta, u32, u32);

    fn next(&mut self) -> Option<(u32, u32, CoordsDelta, u32, u32)> {
        if let Some((&(time_step, cycle, x, y, subloc), &count)) =
            self.inner.next()
        {
            Some((time_step, cycle, CoordsDelta::new(x, y), subloc, count))
        } else {
            None
        }
    }
}

//===========================================================================//

fn button_key_string(
    time_step: u32,
    cycle: u32,
    x: i32,
    y: i32,
    subloc: u32,
) -> String {
    format!(
        "t{:05}_{:03}_{}_{}",
        time_step,
        cycle,
        delta_key_string(CoordsDelta::new(x, y)),
        subloc
    )
}

fn key_string_button(key: &str) -> Option<(u32, u32, i32, i32, u32)> {
    let parts: Vec<&str> = key.splitn(4, "_").collect();
    if parts.len() != 4 {
        return None;
    }
    if !parts[0].starts_with("t") {
        return None;
    }
    let time_step = match parts[0][1..].parse::<u32>() {
        Ok(time_step) => time_step,
        Err(_) => return None,
    };
    let cycle = match parts[1].parse::<u32>() {
        Ok(cycle) => cycle,
        Err(_) => return None,
    };
    let delta = match key_string_delta(parts[2]) {
        Some(delta) => delta,
        None => return None,
    };
    let subloc = match parts[3].parse::<u32>() {
        Ok(subloc) => subloc,
        Err(_) => return None,
    };
    return Some((time_step, cycle, delta.x, delta.y, subloc));
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::super::circuit::CircuitData;
    use super::super::puzzle::Puzzle;
    use super::{InputsData, SolutionData};
    use crate::geom::{CoordsDelta, RectSize};

    #[test]
    fn serialize_solution_data() {
        let mut inputs = InputsData::new();
        inputs.insert(2, 1, CoordsDelta::new(3, 2), 0, 2);
        let solution = SolutionData {
            install_id: None,
            puzzle: Puzzle::TutorialOr,
            score: 14,
            time_steps: 4,
            circuit: CircuitData::new(4, 3),
            inputs: Some(inputs),
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
             [circuit.wires]\n\n\
             [inputs]\n\
             t00002_001_p3p2_0 = 2\n"
        );
    }

    #[test]
    fn deserialize_solution_data() {
        let toml = "puzzle = \"TutorialOr\"\n\
                    score = 14\n\
                    time_steps = 4\n\n\
                    [circuit]\n\
                    size = [4, 3]\n\n\
                    [circuit.chips]\n\n\
                    [circuit.wires]\n\n\
                    [inputs]\n\
                    t00002_001_p3p2_0 = 2\n";
        let solution = SolutionData::deserialize_from_string(toml).unwrap();
        assert_eq!(solution.install_id, None);
        assert_eq!(solution.puzzle, Puzzle::TutorialOr);
        assert_eq!(solution.score, 14);
        assert_eq!(solution.time_steps, 4);
        assert_eq!(solution.circuit.size, RectSize::new(4, 3));
        assert!(solution.inputs.is_some());
        let inputs: Vec<(u32, u32, CoordsDelta, u32, u32)> =
            solution.inputs.unwrap().iter().collect();
        assert_eq!(inputs, vec![(2, 1, CoordsDelta::new(3, 2), 0, 2)]);
    }
}

//===========================================================================//
