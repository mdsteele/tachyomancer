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
#[macro_use]
extern crate tachy;

use iron::status;
use iron::{Iron, IronError, IronResult, Request, Response};
use router::Router;
use std::io::{self, Read};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tachy::save::{Puzzle, ScoreCurve, ScoreCurveMap, SolutionData};
use tachy::state::verify_solution;

// ========================================================================= //

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

//===========================================================================//

fn make_router() -> Router {
    let mut router = Router::new();
    router.get("/", hello_world, "hello_world");
    router.get("/scores", get_scores, "scores");
    router.post("/submit_solution", submit_solution, "submit_solution");
    router
}

//===========================================================================//

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello, world!\n")))
}

fn get_scores(_: &mut Request) -> IronResult<Response> {
    let mut scores = ScoreCurveMap::new();
    // TODO: Populate with real scores.
    scores.set(Puzzle::TutorialOr, ScoreCurve::new(vec![(8, 16), (9, 12)]));
    scores.set(Puzzle::TutorialMux, ScoreCurve::new(vec![(12, 30)]));
    let response = scores.serialize_to_string().map_err(|err| {
        debug_warn!("{}", err);
        let msg = format!("{}\n", err);
        let io_err = io::Error::new(io::ErrorKind::Other, err);
        IronError::new(io_err, (status::InternalServerError, msg))
    })?;
    Ok(Response::with((status::Ok, response)))
}

fn submit_solution(request: &mut Request) -> IronResult<Response> {
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

    // TODO: Hash solution; ignore if we already have a score in the database
    // for that hash.

    // Verify that the solution is valid:
    let errors = verify_solution(&data);
    if errors.is_empty() {
        // TODO: Solution is valid; store in database.
        return Ok(Response::with((status::Ok, "ok\n")));
    } else {
        let mut response = "Circuit had errors:\n".to_string();
        for error in errors.iter() {
            response.push_str(&format!("- {}\n", error));
        }
        return Ok(Response::with((status::Ok, response)));
    }
}

//===========================================================================//
