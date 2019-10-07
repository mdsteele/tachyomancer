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

use crate::mancer::save::GlobalScoresDir;
use std::collections::VecDeque;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tachy::save::{ScoreCurveMap, SolutionData};
use toml;
use ureq;

//===========================================================================//

const CONNECT_TIMEOUT_MS: u64 = 5000;
const WRITE_TIMEOUT_MS: u64 = 1000;
const READ_TIMEOUT_MS: u64 = 1000;
const USER_AGENT: &str = concat!("tachyomancer/", env!("CARGO_PKG_VERSION"));

//===========================================================================//

/// Manages the background thread that communicates with the score server.
pub struct ScoreClient {
    global_scores: Arc<Mutex<ScoreCurveMap>>,
    solution_queue: Arc<Mutex<VecDeque<SolutionData>>>,
}

impl ScoreClient {
    pub(super) fn start(
        server_addr: &str,
        scores_dir: GlobalScoresDir,
    ) -> ScoreClient {
        let score_map =
            scores_dir.load_global_score_cache().unwrap_or_else(|err| {
                debug_warn!("Failed to load global score cache: {}", err);
                ScoreCurveMap::new()
            });
        let global_scores = Arc::new(Mutex::new(score_map));
        let solution_queue = Arc::new(Mutex::new(VecDeque::new()));
        let client = ScoreClient {
            global_scores: global_scores.clone(),
            solution_queue: solution_queue.clone(),
        };
        let server_addr_string = server_addr.to_string();
        thread::spawn(move || {
            score_client_thread_main(
                &server_addr_string,
                global_scores,
                solution_queue,
            );
        });
        client
    }

    pub fn global_scores(&self) -> &Arc<Mutex<ScoreCurveMap>> {
        &self.global_scores
    }

    pub fn submit_solution(&self, solution: SolutionData) {
        self.global_scores.lock().unwrap().insert(
            solution.puzzle,
            solution.circuit.size.area(),
            solution.score,
        );
        // TODO: save global scores cache sometimes
        self.solution_queue.lock().unwrap().push_back(solution);
    }
}

//===========================================================================//

fn score_client_thread_main(
    server_addr: &str,
    global_scores: Arc<Mutex<ScoreCurveMap>>,
    solution_queue: Arc<Mutex<VecDeque<SolutionData>>>,
) {
    match fetch_global_scores(server_addr) {
        Ok(scores) => {
            *global_scores.lock().unwrap() = scores;
        }
        Err(err) => {
            debug_log!("Failed to fetch global scores: {}", err);
            return;
        }
    }

    loop {
        // TODO: Use a Condvar rather than sleeping
        match submit_a_queued_solution(server_addr, &solution_queue) {
            Ok(true) => {}
            Ok(false) => thread::sleep(Duration::from_secs(1)),
            Err(err) => {
                debug_log!("Failed to submit queued solution: {}", err);
                // TODO: Save queued solutions to scores dir, so we can submit
                //   them later.
                return;
            }
        }
    }
}

fn fetch_global_scores(server_addr: &str) -> Result<ScoreCurveMap, String> {
    let mut request = ureq::get(&format!("{}/scores", server_addr));
    set_up_request(&mut request);
    let response = request.call();
    require_200(&response)?;
    let mut bytes = Vec::<u8>::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|err| format!("Failed to read response body: {}", err))?;
    let scores: ScoreCurveMap = toml::from_slice(&bytes)
        .map_err(|err| format!("Could not deserialize scores: {}", err))?;
    return Ok(scores);
}

fn submit_a_queued_solution(
    server_addr: &str,
    solution_queue: &Arc<Mutex<VecDeque<SolutionData>>>,
) -> Result<bool, String> {
    if let Some(solution) = solution_queue.lock().unwrap().pop_front() {
        submit_solution_to_server(server_addr, &solution)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn submit_solution_to_server(
    server_addr: &str,
    solution: &SolutionData,
) -> Result<(), String> {
    let serialized = solution.serialize_to_string()?;
    let mut request = ureq::post(&format!("{}/submit_solution", server_addr));
    set_up_request(&mut request);
    let response = request
        .set("Content-Type", "application/toml; charset=utf-8")
        .send_bytes(serialized.as_bytes());
    require_200(&response)?;
    Ok(())
}

fn set_up_request(request: &mut ureq::Request) {
    request
        .set("User-Agent", USER_AGENT)
        .timeout_connect(CONNECT_TIMEOUT_MS)
        .timeout_write(WRITE_TIMEOUT_MS)
        .timeout_read(READ_TIMEOUT_MS);
}

fn require_200(response: &ureq::Response) -> Result<(), String> {
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
    return Ok(());
}

//===========================================================================//
