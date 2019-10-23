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

extern crate getopts;
extern crate iron;
extern crate router;
extern crate sha;
#[macro_use]
extern crate tachy;

mod db;

use self::db::{InMemoryScoreDatabase, ScoreDatabase};
use iron::status;
use iron::{Handler, Iron, IronError, IronResult, Request, Response};
use router::Router;
use std::io::{self, Read};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tachy::save::SolutionData;
use tachy::state::verify_solution;

//===========================================================================//

fn main() -> Result<(), String> {
    run_server(&parse_flags()?)
}

//===========================================================================//

#[derive(Debug)]
struct StartupFlags {
    addr: SocketAddr,
}

fn parse_flags() -> Result<StartupFlags, String> {
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("", "host", "the IP to listen on", "HOST");
    opts.optopt("", "port", "the port to listen on", "PORT");

    let args: Vec<String> = std::env::args().collect();
    let matches =
        opts.parse(&args[1..]).map_err(|err| format!("{:?}", err))?;
    if matches.opt_present("help") {
        eprint!("{}", opts.usage(&format!("Usage: {} [options]", &args[0])));
        std::process::exit(0);
    }

    let host: IpAddr = matches
        .opt_get_default("host", IpAddr::V4(Ipv4Addr::LOCALHOST))
        .map_err(|err| format!("{:?}", err))?;
    let port: u16 = matches
        .opt_get_default("port", 8080)
        .map_err(|err| format!("{:?}", err))?;
    Ok(StartupFlags { addr: SocketAddr::new(host, port) })
}

//===========================================================================//

fn run_server(flags: &StartupFlags) -> Result<(), String> {
    let server = Iron::new(make_router())
        .http(flags.addr)
        .map_err(|err| format!("{:?}", err))?;
    debug_log!("HTTP server now listening on {}", server.socket);
    Ok(())
}

fn make_router() -> Router {
    let db: Arc<Box<dyn ScoreDatabase>> =
        Arc::new(Box::new(InMemoryScoreDatabase::new()));
    let mut router = Router::new();
    router.get("/", LandingPageHandler { db: db.clone() }, "LandingPage");
    router.get("/scores", GetScoresHandler { db: db.clone() }, "GetScores");
    router.post(
        "/submit_solution",
        SubmitSolutionHandler { db },
        "SubmitSolution",
    );
    router
}

fn internal_error(err: String) -> IronError {
    debug_warn!("{}", err);
    let msg = format!("{}\n", err);
    let io_err = io::Error::new(io::ErrorKind::Other, err);
    IronError::new(io_err, (status::InternalServerError, msg))
}

//===========================================================================//

struct LandingPageHandler {
    db: Arc<Box<dyn ScoreDatabase>>,
}

impl Handler for LandingPageHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let count =
            self.db.load_num_verified_solutions().map_err(internal_error)?;
        let response = format!("There are {} verified solutions.\n", count);
        Ok(Response::with((status::Ok, response)))
    }
}

//===========================================================================//

struct GetScoresHandler {
    db: Arc<Box<dyn ScoreDatabase>>,
}

impl Handler for GetScoresHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let scores = self.db.load_scores().map_err(internal_error)?;
        let response = scores.serialize_to_string().map_err(internal_error)?;
        Ok(Response::with((status::Ok, response)))
    }
}

//===========================================================================//

struct SubmitSolutionHandler {
    db: Arc<Box<dyn ScoreDatabase>>,
}

impl Handler for SubmitSolutionHandler {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        debug_log!("Received submission from {:?}", request.remote_addr);

        // Read the serialized solution from the client:
        let mut body = String::new();
        request.body.read_to_string(&mut body).map_err(|err| {
            let msg = format!("{}\n", err);
            IronError::new(err, (status::BadRequest, msg))
        })?;

        // Deserialize the SolutionData:
        let data =
            SolutionData::deserialize_from_string(&body).map_err(|err| {
                let msg = format!("{}\n", err);
                let io_err = io::Error::new(io::ErrorKind::InvalidData, err);
                IronError::new(io_err, (status::BadRequest, msg))
            })?;
        debug_log!(
            "Got solution data from ID={:?} for {:?} with size: {:?}",
            data.install_id,
            data.puzzle,
            data.circuit.size
        );

        // Ignore solutions that fall outside graph bounds:
        let (max_area, max_score) = data.puzzle.graph_bounds();
        if data.score > max_score || data.circuit.size.area() > max_area {
            let response = "Solution is not within graph bounds.".to_string();
            return Ok(Response::with((status::Ok, response)));
        }

        // If we've never verified this solution before, verify it:
        if let Some(key) =
            self.db.store_new_solution(&data).map_err(internal_error)?
        {
            let errors = verify_solution(&data);
            if errors.is_empty() {
                self.db
                    .store_verified_solution(
                        key,
                        data.circuit.size.area(),
                        data.score,
                    )
                    .map_err(internal_error)?;
            } else {
                let mut response = "Circuit had errors:\n".to_string();
                for error in errors.iter() {
                    response.push_str(&format!("- {}\n", error));
                }
                return Ok(Response::with((status::Ok, response)));
            }
        };
        return Ok(Response::with((status::Ok, "ok\n")));
    }
}

//===========================================================================//
