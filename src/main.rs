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
extern crate getopts;
extern crate gl;
extern crate sdl2;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod tachy;

use self::tachy::gui::{Event, GuiContext, Window, WindowOptions};
use self::tachy::save::SaveDir;
use std::path::PathBuf;

// ========================================================================= //

#[derive(Debug)]
struct StartupOptions {
    fullscreen: Option<bool>,
    resolution: Option<(u32, u32)>,
    save_dir: Option<PathBuf>,
}

fn main() {
    match start_game(parse_options()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(1);
        }
    }
}

fn parse_options() -> StartupOptions {
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflagopt("", "fullscreen", "override fullscreen setting", "BOOL");
    opts.optopt("", "resolution", "override window/screen resolution", "WxH");
    opts.optopt("", "save_dir", "override save dir path", "PATH");

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

    let fullscreen = matches
        .opt_default("fullscreen", "true")
        .and_then(|value| value.parse().ok());
    let resolution = matches.opt_str("resolution").and_then(|value| {
        let pieces: Vec<&str> = value.split('x').collect();
        if pieces.len() != 2 {
            return None;
        }
        pieces[0].parse::<u32>().ok().and_then(|width| {
            pieces[1]
                .parse::<u32>()
                .ok()
                .and_then(|height| Some((width, height)))
        })
    });
    let save_dir = matches.opt_str("save_dir").map(PathBuf::from);
    StartupOptions {
        fullscreen,
        resolution,
        save_dir,
    }
}

fn start_game(options: StartupOptions) -> Result<(), String> {
    let savedir = SaveDir::create_or_load(&options.save_dir)?;
    let mut gui_context = GuiContext::init()?;
    let fullscreen =
        options.fullscreen.unwrap_or_else(|| savedir.prefs().fullscreen());
    let resolution = if let Some(res) = options.resolution {
        res
    } else if let Some(res) = savedir.prefs().resolution() {
        res
    } else {
        gui_context.get_native_resolution()?
    };
    let window_options = WindowOptions {
        fullscreen,
        resolution,
    };
    boot_window(&mut gui_context, &window_options)?;
    Ok(())
}

// ========================================================================= //

fn boot_window(context: &mut GuiContext, options: &WindowOptions)
               -> Result<(), String> {
    let mut window = Window::create(context, options)?;
    loop {
        match window.poll_event() {
            Some(Event::Quit) => return Ok(()),
            None => {
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }
                window.swap();
            }
        }
    }
}

//===========================================================================//
