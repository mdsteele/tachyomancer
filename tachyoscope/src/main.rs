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
use tachy::save::{Puzzle, SolutionData};
use tachy::state::{EditGrid, EvalResult, EvalScore, WireError};

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
    router.post("/submit_solution", submit_solution, "submit_solution");
    router
}

//===========================================================================//

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello, world!\n")))
}

fn submit_solution(request: &mut Request) -> IronResult<Response> {
    debug_log!("Received submission from {:?}", request.remote_addr);

    let mut body = String::new();
    request.body.read_to_string(&mut body).map_err(|err| {
        let msg = format!("{}\n", err);
        IronError::new(err, (status::BadRequest, msg))
    })?;

    let data =
        SolutionData::deserialize_from_string(&body).map_err(|err| {
            let msg = format!("{}\n", err);
            let io_err = io::Error::new(io::ErrorKind::InvalidData, err);
            IronError::new(io_err, (status::BadRequest, msg))
        })?;
    debug_log!(
        "Got solution data for {:?} with size: {:?}",
        data.puzzle,
        data.circuit.size
    );

    let mut grid = EditGrid::from_circuit_data(
        data.puzzle,
        &Puzzle::all().collect(),
        &data.circuit,
    );
    let wire_length = grid.wire_fragments().len();
    let mut errors = Vec::<String>::new();
    if !grid.start_eval() {
        for error in grid.errors() {
            errors.push(match error {
                WireError::MultipleSenders(idx) => {
                    format!("Wire {} has multiple senders", idx)
                }
                WireError::PortColorMismatch(idx) => {
                    format!("Wire {} has a color mismatch", idx)
                }
                WireError::NoValidSize(idx) => {
                    format!("Wire {} has a size mismatch", idx)
                }
                WireError::UnbrokenLoop(idxs, _) => {
                    format!("Wires {:?} form a loop", idxs)
                }
            });
        }
        errors.push("Circuit had errors".to_string());
    } else {
        let eval = grid.eval_mut().unwrap();
        for time_step in 0..(data.time_steps + 1) {
            match eval.step_time() {
                EvalResult::Continue if time_step < data.time_steps => {}
                EvalResult::Continue => {
                    errors.push(format!(
                        "Evaluation did not end at time step {}",
                        time_step
                    ));
                    break;
                }
                EvalResult::Breakpoint(_) => {
                    // TODO: We should just ignore breakpoints.
                    errors.push(format!(
                        "Breakpoint at time step {}",
                        time_step
                    ));
                    break;
                }
                EvalResult::Failure => {
                    errors.extend(
                        eval.errors()
                            .iter()
                            .map(|error| error.message.clone()),
                    );
                    break;
                }
                EvalResult::Victory(score) if time_step < data.time_steps => {
                    errors.push(format!(
                        "Unexpected victory at time step {}: {:?}",
                        time_step, score
                    ));
                    break;
                }
                EvalResult::Victory(score) => {
                    let score = match score {
                        EvalScore::Value(value) => value as u32,
                        EvalScore::WireLength => wire_length as u32,
                    };
                    if score != data.score {
                        errors.push(format!(
                            "Actual score was {}, but expected {}",
                            score, data.score
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(Response::with((status::Ok, "ok\n")))
    } else {
        let mut response = "Circuit had errors:\n".to_string();
        for error in errors.iter() {
            response.push_str(&format!("- {}\n", error));
        }
        Ok(Response::with((status::Ok, response)))
    }
}

//===========================================================================//
