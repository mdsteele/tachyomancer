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
use serde::de::Error;
use std::collections::{BTreeMap, HashMap};
use std::i64;
use std::str::FromStr;

//===========================================================================//

// TODO: Once const fn Vec::new is stabilized, make a public constant for an
//   empty ScoreCurve so we can return &ScoreCurve in various places instead of
//   Option<&ScoreCurve>, and not require ScoreCurveMap to store empty curves.
#[derive(Clone)]
pub struct ScoreCurve {
    scores: Vec<(i32, u32)>,
}

impl ScoreCurve {
    pub fn new(mut scores: Vec<(i32, u32)>) -> ScoreCurve {
        ScoreCurve::fix(&mut scores);
        ScoreCurve { scores }
    }

    pub fn is_empty(&self) -> bool {
        self.scores.is_empty()
    }

    pub fn scores(&self) -> &[(i32, u32)] {
        &self.scores
    }

    pub fn insert(&mut self, score: (i32, u32)) {
        self.scores.push(score);
        ScoreCurve::fix(&mut self.scores);
    }

    fn fix(points: &mut Vec<(i32, u32)>) {
        points.sort();
        let mut best_score = i64::MAX;
        points.retain(|&(_, score)| {
            let score = score as i64;
            if score < best_score {
                best_score = score;
                true
            } else {
                false
            }
        });
    }
}

impl<'d> serde::Deserialize<'d> for ScoreCurve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let scores = Vec::<(i32, u32)>::deserialize(deserializer)?;
        Ok(ScoreCurve::new(scores))
    }
}

impl serde::Serialize for ScoreCurve {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.scores.serialize(serializer)
    }
}

//===========================================================================//

pub struct ScoreCurveMap {
    map: HashMap<Puzzle, ScoreCurve>,
}

impl ScoreCurveMap {
    pub fn new() -> ScoreCurveMap {
        ScoreCurveMap {
            map: Puzzle::all()
                .map(|puzzle| (puzzle, ScoreCurve::new(Vec::new())))
                .collect(),
        }
    }

    pub fn get(&self, puzzle: Puzzle) -> &ScoreCurve {
        self.map.get(&puzzle).unwrap()
    }

    pub fn set(&mut self, puzzle: Puzzle, scores: ScoreCurve) {
        self.map.insert(puzzle, scores);
    }

    pub fn insert(&mut self, puzzle: Puzzle, area: i32, score: u32) {
        self.map.get_mut(&puzzle).unwrap().insert((area, score));
    }

    pub fn deserialize_from_string(
        string: &str,
    ) -> Result<ScoreCurveMap, String> {
        toml::from_str(string)
            .map_err(|err| format!("Could not deserialize scores: {}", err))
    }

    pub fn serialize_to_string(&self) -> Result<String, String> {
        toml::to_string(self)
            .map_err(|err| format!("Could not serialize scores: {}", err))
    }
}

impl<'d> serde::Deserialize<'d> for ScoreCurveMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let string_map =
            BTreeMap::<&str, ScoreCurve>::deserialize(deserializer)?;
        let mut score_map = ScoreCurveMap::new();
        for (key, scores) in string_map.into_iter() {
            match Puzzle::from_str(key) {
                Ok(puzzle) => {
                    score_map.set(puzzle, scores);
                }
                Err(_) => {
                    return Err(D::Error::custom(format!(
                        "Invalid puzzle key: {:?}",
                        key
                    )))
                }
            }
        }
        return Ok(score_map);
    }
}

impl serde::Serialize for ScoreCurveMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.map
            .iter()
            .filter(|(_, curve)| !curve.is_empty())
            .map(|(puzzle, curve)| (format!("{:?}", puzzle), curve))
            .collect::<BTreeMap<String, &ScoreCurve>>()
            .serialize(serializer)
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{ScoreCurve, ScoreCurveMap};
    use crate::save::Puzzle;
    use std::collections::HashMap;

    #[test]
    fn fix_empty_scores() {
        let mut scores = vec![];
        ScoreCurve::fix(&mut scores);
        assert_eq!(scores, vec![]);
    }

    #[test]
    fn fix_scores_with_one_score() {
        let mut scores = vec![(20, 30)];
        ScoreCurve::fix(&mut scores);
        assert_eq!(scores, vec![(20, 30)]);
    }

    #[test]
    fn fix_unsorted_scores() {
        let mut scores = vec![(16, 35), (9, 50), (20, 30), (12, 40)];
        ScoreCurve::fix(&mut scores);
        assert_eq!(scores, vec![(9, 50), (12, 40), (16, 35), (20, 30)]);
    }

    #[test]
    fn fix_repeated_scores() {
        let mut scores = vec![(9, 50), (16, 35), (16, 35), (9, 50)];
        ScoreCurve::fix(&mut scores);
        assert_eq!(scores, vec![(9, 50), (16, 35)]);
    }

    #[test]
    fn fix_dominated_scores_with_same_area() {
        let mut scores = vec![(9, 60), (9, 50), (16, 35), (16, 40)];
        ScoreCurve::fix(&mut scores);
        assert_eq!(scores, vec![(9, 50), (16, 35)]);
    }

    #[test]
    fn fix_dominated_scores_with_same_score() {
        let mut scores = vec![(9, 60), (16, 60), (20, 30)];
        ScoreCurve::fix(&mut scores);
        assert_eq!(scores, vec![(9, 60), (20, 30)]);
    }

    #[test]
    fn fix_fully_dominated_scores() {
        let mut scores = vec![(9, 60), (20, 70), (16, 75)];
        ScoreCurve::fix(&mut scores);
        assert_eq!(scores, vec![(9, 60)]);
    }

    #[test]
    fn serialize_score_curve() {
        let scores = ScoreCurve::new(vec![(16, 85), (20, 43)]);
        let mut map = HashMap::<String, ScoreCurve>::new();
        map.insert("foo".to_string(), scores);
        let bytes = toml::to_vec(&map).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap().as_str(),
            "foo = [[16, 85], [20, 43]]\n"
        );
    }

    #[test]
    fn deserialize_score_curve() {
        let toml = "foo = [[20, 43], [16, 85], [30, 100]]\n";
        let map: HashMap<String, ScoreCurve> =
            toml::from_slice(toml.as_bytes()).unwrap();
        assert_eq!(map.get("foo").unwrap().scores(), &[(16, 85), (20, 43)]);
    }

    #[test]
    fn serialize_score_curve_map() {
        let mut scores = ScoreCurveMap::new();
        scores
            .set(Puzzle::TutorialOr, ScoreCurve::new(vec![(8, 16), (9, 12)]));
        scores.set(Puzzle::TutorialMux, ScoreCurve::new(vec![(12, 30)]));
        scores.set(Puzzle::TutorialAdd, ScoreCurve::new(vec![]));
        let serialized = scores.serialize_to_string().unwrap();
        assert_eq!(
            serialized.as_str(),
            "TutorialMux = [[12, 30]]\n\
             TutorialOr = [[8, 16], [9, 12]]\n"
        );
    }

    #[test]
    fn deserialize_score_curve_map() {
        let serialized = "TutorialMux = [[12, 30]]\n\
                          TutorialOr = [[8, 16], [9, 12]]\n";
        let scores =
            ScoreCurveMap::deserialize_from_string(serialized).unwrap();
        assert_eq!(
            scores.get(Puzzle::TutorialOr).scores(),
            &[(8, 16), (9, 12)]
        );
        assert_eq!(scores.get(Puzzle::TutorialMux).scores(), &[(12, 30)]);
        assert_eq!(scores.get(Puzzle::TutorialAdd).scores(), &[]);
    }
}

//===========================================================================//
