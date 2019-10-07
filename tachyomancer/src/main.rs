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

extern crate app_dirs;
extern crate cgmath;
extern crate claxon;
extern crate getopts;
extern crate gl;
extern crate jpeg_decoder;
extern crate lewton;
extern crate num_integer;
extern crate png;
extern crate sdl2;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate strum;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate tachy;
extern crate toml;
extern crate unicase;
extern crate ureq;

mod mancer;

use self::mancer::gui::{GuiContext, Window, WindowOptions};
use self::mancer::mode::{self, ModeChange};
use self::mancer::save::{Prefs, SaveDir};
use self::mancer::state::GameState;
use std::path::PathBuf;
use tachy::geom::RectSize;

//===========================================================================//

// TODO: Change this once we have a real score server.
const DEFAULT_SCORE_SERVER_ADDR: &str = "http://127.0.0.1:8080";

//===========================================================================//

fn main() {
    match run_game(&parse_flags()) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("ERROR: {}", error);
            let message = format!(
                "Please file a bug with the below information at\n\
                 https://github.com/mdsteele/tachyomancer/issues\n\
                 \n{}\n\n\
                 OS={}, ARCH={}",
                error,
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            let result = sdl2::messagebox::show_simple_message_box(
                sdl2::messagebox::MessageBoxFlag::ERROR,
                "Tachyomancer Error",
                &message.replace('\0', ""),
                None,
            );
            if let Err(message_box_error) = result {
                eprintln!(
                    "ERROR: Failed to show message box: {:?}",
                    message_box_error
                );
            }
            std::process::exit(1);
        }
    }
}

//===========================================================================//

#[derive(Debug)]
struct StartupFlags {
    antialiasing: Option<bool>,
    fullscreen: Option<bool>,
    resolution: Option<RectSize<i32>>,
    save_dir: Option<PathBuf>,
    score_server_addr: Option<String>,
}

fn parse_flags() -> StartupFlags {
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflagopt(
        "",
        "antialiasing",
        "override antialiasing setting",
        "BOOL",
    );
    opts.optflagopt("", "fullscreen", "override fullscreen setting", "BOOL");
    opts.optopt("", "resolution", "override window/screen resolution", "WxH");
    opts.optopt("", "save_dir", "override save dir path", "PATH");
    opts.optopt("", "score_server", "override score server address", "ADDR");

    let args: Vec<String> = std::env::args().collect();
    let matches = opts.parse(&args[1..]).unwrap_or_else(|failure| {
        eprintln!("Error: {:?}", failure);
        eprintln!("Run with --help to see available flags.");
        std::process::exit(1);
    });
    if matches.opt_present("help") {
        eprint!("{}", opts.usage(&format!("Usage: {} [options]", &args[0])));
        std::process::exit(0);
    }

    let antialiasing = matches
        .opt_default("antialiasing", "true")
        .and_then(|value| value.parse().ok());
    let fullscreen = matches
        .opt_default("fullscreen", "true")
        .and_then(|value| value.parse().ok());
    let resolution = matches.opt_str("resolution").and_then(|value| {
        let pieces: Vec<&str> = value.split('x').collect();
        if pieces.len() != 2 {
            return None;
        }
        pieces[0].parse::<i32>().ok().and_then(|width| {
            pieces[1]
                .parse::<i32>()
                .ok()
                .and_then(|height| Some(RectSize::new(width, height)))
        })
    });
    let save_dir = matches.opt_str("save_dir").map(PathBuf::from);
    let score_server_addr = matches.opt_str("score_server");
    StartupFlags {
        antialiasing,
        fullscreen,
        resolution,
        save_dir,
        score_server_addr,
    }
}

//===========================================================================//

fn run_game(flags: &StartupFlags) -> Result<(), String> {
    let mut save_dir = SaveDir::create_or_load(&flags.save_dir)?;
    let scores_dir = save_dir.create_or_load_global_scores()?;
    let mut state = GameState::new(save_dir)?;
    let mut gui_context = GuiContext::init(
        state.prefs().sound_volume_percent(),
        state.prefs().music_volume_percent(),
        flags
            .score_server_addr
            .as_ref()
            .map(String::as_str)
            .unwrap_or(DEFAULT_SCORE_SERVER_ADDR),
        scores_dir,
    )?;
    let mut window_options =
        Some(initial_window_options(flags, state.prefs())?);
    while let Some(options) = window_options {
        window_options = boot_window(&mut state, &mut gui_context, options)?;
    }
    state.save()?;
    Ok(())
}

fn initial_window_options(
    flags: &StartupFlags,
    prefs: &Prefs,
) -> Result<WindowOptions, String> {
    let antialiasing =
        flags.antialiasing.unwrap_or_else(|| prefs.antialiasing());
    let fullscreen = flags.fullscreen.unwrap_or_else(|| prefs.fullscreen());
    let resolution = flags.resolution.or_else(|| prefs.resolution());
    Ok(WindowOptions { antialiasing, fullscreen, resolution })
}

//===========================================================================//

fn boot_window(
    state: &mut GameState,
    gui_context: &mut GuiContext,
    window_options: WindowOptions,
) -> Result<Option<WindowOptions>, String> {
    let mut window = Window::create(gui_context, window_options)?;
    loop {
        match mode::run_mode(state, &mut window) {
            ModeChange::Next => continue,
            ModeChange::RebootWindow(new_options) => {
                return Ok(Some(new_options))
            }
            ModeChange::Quit => return Ok(None),
        }
    }
}

//===========================================================================//
