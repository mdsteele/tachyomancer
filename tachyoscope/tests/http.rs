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

extern crate iron;
extern crate portpicker;
extern crate tachy;
extern crate tachyoscope;
extern crate ureq;

use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tachy::save::{ScoreCurveMap, SolutionData};

//===========================================================================//

const CONNECT_TIMEOUT_MS: u64 = 5000;
const WRITE_TIMEOUT_MS: u64 = 1000;
const READ_TIMEOUT_MS: u64 = 1000;

//===========================================================================//

#[test]
fn readiness_check() {
    let port = portpicker::pick_unused_port().unwrap();
    let flags = tachyoscope::StartupFlags {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
    };
    let _server = Server(tachyoscope::run_server(&flags).unwrap());
    let response =
        http_get(&format!("http://localhost:{}/readiness_check", port));
    assert_eq!(response.status(), 200);
    let mut payload = String::new();
    response.into_reader().read_to_string(&mut payload).unwrap();
    assert_eq!(payload, "ready\n");
}

#[test]
fn submit_solution() {
    // Start a score server with a fresh database:
    let port = portpicker::pick_unused_port().unwrap();
    let flags = tachyoscope::StartupFlags {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
    };
    let _server = Server(tachyoscope::run_server(&flags).unwrap());

    // Load a solution:
    let solution =
        SolutionData::load("tests/solutions/tutorial_or_1.toml").unwrap();

    // Get the initial set of scores:
    let response = http_get(&format!("http://localhost:{}/scores", port));
    assert_eq!(response.status(), 200);
    let mut payload = String::new();
    response.into_reader().read_to_string(&mut payload).unwrap();
    let scores = ScoreCurveMap::deserialize_from_string(&payload).unwrap();
    assert!(scores.get(solution.puzzle).is_empty());

    // Submit the solution:
    let response = http_post(
        &format!("http://localhost:{}/submit_solution", port),
        "application/toml",
        &solution.serialize_to_string().unwrap().as_bytes(),
    );
    assert_eq!(response.status(), 200);

    // Get the scores again and make sure the submitted solution is there:
    let response = http_get(&format!("http://localhost:{}/scores", port));
    assert_eq!(response.status(), 200);
    let mut payload = String::new();
    response.into_reader().read_to_string(&mut payload).unwrap();
    let scores = ScoreCurveMap::deserialize_from_string(&payload).unwrap();
    assert_eq!(
        scores.get(solution.puzzle).scores(),
        &[(solution.circuit.size.area(), solution.score)]
    );
}

#[test]
fn submit_invalid_utf8() {
    let port = portpicker::pick_unused_port().unwrap();
    let flags = tachyoscope::StartupFlags {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
    };
    let _server = Server(tachyoscope::run_server(&flags).unwrap());
    let response = http_post(
        &format!("http://localhost:{}/submit_solution", port),
        "application/toml",
        b"\x80\x81\x82\x83",
    );
    assert_eq!(response.status(), 400);
    let mut payload = String::new();
    response.into_reader().read_to_string(&mut payload).unwrap();
    assert_eq!(payload, "stream did not contain valid UTF-8\n");
}

#[test]
fn submit_invalid_solution_toml() {
    let port = portpicker::pick_unused_port().unwrap();
    let flags = tachyoscope::StartupFlags {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
    };
    let _server = Server(tachyoscope::run_server(&flags).unwrap());
    let response = http_post(
        &format!("http://localhost:{}/submit_solution", port),
        "application/toml",
        b"puzzle = \"Foobar\"\n",
    );
    assert_eq!(response.status(), 400);
    let mut payload = String::new();
    response.into_reader().read_to_string(&mut payload).unwrap();
    assert!(payload.starts_with("Could not deserialize solution"));
}

//===========================================================================//

struct Server(iron::Listening);

impl Drop for Server {
    fn drop(&mut self) {
        self.0.close().unwrap();
    }
}

//===========================================================================//

fn http_get(url: &str) -> ureq::Response {
    ureq::get(url)
        .timeout_connect(CONNECT_TIMEOUT_MS)
        .timeout_write(WRITE_TIMEOUT_MS)
        .timeout_read(READ_TIMEOUT_MS)
        .call()
}

fn http_post(url: &str, mime_type: &str, payload: &[u8]) -> ureq::Response {
    ureq::post(url)
        .timeout_connect(CONNECT_TIMEOUT_MS)
        .timeout_write(WRITE_TIMEOUT_MS)
        .timeout_read(READ_TIMEOUT_MS)
        .set("Content-Type", &format!("{}; charset=utf-8", mime_type))
        .send_bytes(payload)
}

//===========================================================================//
