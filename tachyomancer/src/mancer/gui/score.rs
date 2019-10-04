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

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use tachy::save::{Puzzle, ScoreCurve, ScoreCurveMap, SolutionData};
use toml;
use ureq;

//===========================================================================//

const CONNECT_TIMEOUT_MS: u64 = 5000;
const WRITE_TIMEOUT_MS: u64 = 1000;
const READ_TIMEOUT_MS: u64 = 1000;
const USER_AGENT: &str = concat!("tachyomancer/", env!("CARGO_PKG_VERSION"));

//===========================================================================//

#[derive(Clone)]
pub struct GlobalScores {
    scores: Arc<Mutex<ScoreCurveMap>>,
}

impl GlobalScores {
    pub fn new() -> GlobalScores {
        GlobalScores { scores: Arc::new(Mutex::new(ScoreCurveMap::new())) }
    }

    pub fn scores_for(&self, puzzle: Puzzle) -> ScoreCurve {
        self.scores.lock().unwrap().get(puzzle).clone()
    }

    fn replace_with(&self, scores: ScoreCurveMap) {
        *self.scores.lock().unwrap() = scores;
    }
}

//===========================================================================//

pub struct ScoreClient {
    global_scores: GlobalScores,
}

impl ScoreClient {
    pub(super) fn start(server_addr: &str) -> ScoreClient {
        let global_scores = GlobalScores::new();
        let client = ScoreClient { global_scores: global_scores.clone() };
        let server_addr_string = server_addr.to_string();
        thread::spawn(move || {
            // TODO: First, load saved cache of global scores, if present.
            //   That way we can work offline.  Replace them with real scores
            //   if the fetch succeeds.
            let scores_map = match fetch_global_scores(&server_addr_string) {
                Ok(scores) => scores,
                Err(error) => {
                    debug_log!("Failed to fetch global scores: {}", error);
                    return;
                }
            };
            global_scores.replace_with(scores_map);
        });
        client
    }

    pub fn global_scores(&self) -> &GlobalScores {
        &self.global_scores
    }

    pub fn submit_solution(&self, solution: SolutionData) {
        debug_log!(
            "Submitting solution for {:?} with score {}",
            solution.puzzle,
            solution.score
        );
        // TODO: Insert score into global_scores cache
        // TODO: Queue solution to be sent to score server.
    }
}

//===========================================================================//

fn fetch_global_scores(server_addr: &str) -> Result<ScoreCurveMap, String> {
    let response = ureq::get(&format!("{}/scores", server_addr))
        .set("User-Agent", USER_AGENT)
        .timeout_connect(CONNECT_TIMEOUT_MS)
        .timeout_write(WRITE_TIMEOUT_MS)
        .timeout_read(READ_TIMEOUT_MS)
        .call();
    if let Some(err) = response.synthetic_error() {
        return Err(format!("Network error: {}", err));
    }
    let status = response.status();
    if status != 200 {
        return Err(format!(
            "Got HTTP {}: {}",
            status,
            response.status_text()
        ));
    }
    let mut bytes = Vec::<u8>::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|err| format!("Failed to read response body: {}", err))?;
    let scores: ScoreCurveMap = toml::from_slice(&bytes)
        .map_err(|err| format!("Could not deserialize scores: {}", err))?;
    return Ok(scores);
}

//===========================================================================//
