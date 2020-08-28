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

use super::audio::AudioQueue;
use super::context::GuiContext;
use super::cursor::NextCursor;
use super::event::Event;
use super::resource::Resources;
use super::ui::Ui;
use crate::mancer::font::Align;
use gl;
use sdl2;
use std::mem;
use std::os::raw::c_void;
use std::time::Instant;
use tachy::geom::{AsFloat, Color4, RectSize};
use tachy::save::SolutionData;

//===========================================================================//

const WINDOW_MIN_WIDTH: i32 = 1024;
const WINDOW_MIN_HEIGHT: i32 = 640;
const WINDOW_TITLE: &str = "Tachyomancer";

//===========================================================================//

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowOptions {
    pub antialiasing: bool,
    pub fullscreen: bool,
    pub resolution: Option<RectSize<i32>>,
}

pub struct Window<'a> {
    gui_context: &'a mut GuiContext,
    sdl_window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    resources: Resources,
    possible_resolutions: Vec<RectSize<i32>>,
    options: WindowOptions,
    audio: AudioQueue,
    next_cursor: NextCursor,
    last_clock_tick: Instant,
    last_event_was_clock_tick: bool,
    redraw_requested: bool,
    debug_counter: i32,
}

impl<'a> Window<'a> {
    pub fn create(
        gui_context: &'a mut GuiContext,
        options: WindowOptions,
    ) -> Result<Window<'a>, String> {
        debug_log!("Creating window: {:?}", options);
        {
            let gl_attr = gui_context.video_subsystem.gl_attr();

            // According to https://stackoverflow.com/a/20932820, for MacOS at
            // least we need to explicitly select the Core Profile, because
            // otherwise we will default to the Legacy Profile and our
            // "#version 330 core" shaders won't work.
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 3); // OpenGL 3.3

            // Disable deprecated functionality.
            gl_attr.set_context_flags().forward_compatible().set();

            // Make sure we have a stencil buffer (1 bit is all we need).
            gl_attr.set_stencil_size(1);

            // Optionally enable multisample antialiasing.
            if options.antialiasing {
                gl_attr.set_multisample_buffers(1);
                gl_attr.set_multisample_samples(4);
            }
        }

        let native_resolution = gui_context.get_native_resolution()?;
        let resolution = options.resolution.unwrap_or(native_resolution);
        let sdl_window = {
            let width = resolution
                .width
                .max(WINDOW_MIN_WIDTH)
                .min(native_resolution.width);
            let height = resolution
                .height
                .max(WINDOW_MIN_HEIGHT)
                .min(native_resolution.height);
            let mut builder = gui_context.video_subsystem.window(
                WINDOW_TITLE,
                width as u32,
                height as u32,
            );
            builder.opengl();
            if options.fullscreen {
                if options.resolution.is_none() {
                    builder.fullscreen_desktop();
                } else {
                    builder.fullscreen();
                }
            } else {
                builder.position_centered();
            };
            builder
                .build()
                .map_err(|err| format!("Could not create window: {}", err))?
        };

        let gl_context = sdl_window.gl_create_context()?;
        // According to https://wiki.libsdl.org/SDL_GL_GetProcAddress, to
        // support Windows, we should wait until after we've created the GL
        // context before calling SDL_GL_GetProcAddress.
        gl::load_with(|name| {
            gui_context.video_subsystem.gl_get_proc_address(name)
                as *const c_void
        });
        gui_context
            .video_subsystem
            .gl_set_swap_interval(sdl2::video::SwapInterval::VSync)?;
        unsafe {
            // Our shaders all use premultiplied colors, so set the blend
            // function appropriately.
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            // We keep the depth test disabled for most rendering, and
            // selectively enable it for rendering certain things.
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            debug_assert_eq!(gl::GetError(), gl::NO_ERROR);
        }

        let (width, height) = sdl_window.size();
        let resources = Resources::new(
            RectSize::new(width as i32, height as i32),
            gui_context.score_client.global_scores().clone(),
        )?;

        let mut possible_resolutions =
            gui_context.get_possible_resolutions()?;
        possible_resolutions.retain(|res| {
            res.width >= WINDOW_MIN_WIDTH && res.height >= WINDOW_MIN_HEIGHT
        });

        Ok(Window {
            gui_context,
            sdl_window,
            _gl_context: gl_context,
            resources,
            possible_resolutions,
            options,
            audio: AudioQueue::new(),
            next_cursor: NextCursor::new(),
            last_clock_tick: Instant::now(),
            last_event_was_clock_tick: false,
            redraw_requested: true,
            debug_counter: 0,
        })
    }

    pub fn size(&self) -> RectSize<i32> {
        let (width, height) = self.sdl_window.size();
        RectSize::new(width as i32, height as i32)
    }

    pub fn possible_resolutions(&self) -> &[RectSize<i32>] {
        &self.possible_resolutions
    }

    pub fn options(&self) -> &WindowOptions {
        &self.options
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn ui(&mut self) -> Ui {
        Ui::new(
            &mut self.audio,
            &mut self.gui_context.clipboard,
            &mut self.next_cursor,
            &self.gui_context.event_pump,
            &mut self.redraw_requested,
        )
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.gui_context.sdl_context.mouse().show_cursor(visible);
    }

    pub fn next_event(&mut self) -> Event {
        let mut should_block = false;
        if self.last_event_was_clock_tick {
            self.last_event_was_clock_tick = false;
            if self.redraw_requested {
                self.redraw_requested = false;
                return Event::Redraw;
            } else {
                should_block = true;
            }
        }
        if cfg!(debug_assertions) {
            if let Some(line) = self.gui_context.stdin_reader.pop_line() {
                return Event::new_debug(&line);
            }
        }
        loop {
            let pump = &mut self.gui_context.event_pump;
            let maybe_sdl_event = if should_block {
                should_block = false;
                pump.wait_event_timeout(30)
            } else {
                pump.poll_event()
            };
            match maybe_sdl_event {
                None => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(self.last_clock_tick);
                    self.last_clock_tick = now;
                    self.last_event_was_clock_tick = true;
                    return Event::new_clock_tick(elapsed);
                }
                Some(sdl_event) => {
                    if let Some(event) = Event::from_sdl_event(sdl_event, pump)
                    {
                        return event;
                    }
                }
            }
        }
    }

    pub fn pump_audio(&mut self) {
        let mut queue = self.gui_context.audio_queue.lock().unwrap();
        queue.merge(mem::replace(&mut self.audio, AudioQueue::new()));
    }

    pub fn pump_cursor(&mut self) {
        let cursor = mem::replace(&mut self.next_cursor, NextCursor::new());
        self.gui_context.cursors.set(cursor);
    }

    pub fn pump_video(&mut self) {
        if cfg!(debug_assertions) {
            let size = self.size().as_f32();
            let matrix =
                cgmath::ortho(0.0, size.width, size.height, 0.0, -1.0, 1.0);
            let font = self.resources.fonts().roman();
            font.draw_style(
                &matrix,
                20.0,
                Align::TopRight,
                (size.width - 5.0, 5.0),
                &Color4::WHITE,
                0.0,
                &format!("{:03}", self.debug_counter),
            );
            self.debug_counter = (self.debug_counter + 1) % 1000;
        }
        self.sdl_window.gl_swap_window();
    }

    pub fn submit_solution(&mut self, solution: SolutionData) {
        self.gui_context.score_client.submit_solution(solution);
    }
}

//===========================================================================//
