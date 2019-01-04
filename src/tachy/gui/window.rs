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
use super::event::Event;
use super::resource::Resources;
use gl;
use sdl2;
use std::mem;
use std::os::raw::c_void;

//===========================================================================//

const WINDOW_TITLE: &str = "Tachyomancer";

//===========================================================================//

#[derive(Debug)]
pub struct WindowOptions {
    pub fullscreen: bool,
    pub resolution: (u32, u32),
}

pub struct Window<'a> {
    gui_context: &'a mut GuiContext,
    sdl_window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    resources: Resources,
}

impl<'a> Window<'a> {
    pub fn create(gui_context: &'a mut GuiContext, options: &WindowOptions)
                  -> Result<Window<'a>, String> {
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

            // TODO: enable anti-aliasing?
        }
        let (width, height) = options.resolution;
        let sdl_window =
            if options.fullscreen {
                gui_context
                    .video_subsystem
                    .window(WINDOW_TITLE, width, height)
                    .opengl()
                    .fullscreen()
                    .build()
            } else {
                gui_context
                    .video_subsystem
                    .window(WINDOW_TITLE, width, height)
                    .opengl()
                    .position_centered()
                    .build()
            }.map_err(|err| format!("Could not create window: {}", err))?;
        let gl_context = sdl_window.gl_create_context()?;
        // According to https://wiki.libsdl.org/SDL_GL_GetProcAddress, to
        // support Windows, we should wait until after we've created the GL
        // context before calling SDL_GL_GetProcAddress.
        gl::load_with(|name| {
                          gui_context
                              .video_subsystem
                              .gl_get_proc_address(name) as
                              *const c_void
                      });
        gui_context
            .video_subsystem
            .gl_set_swap_interval(sdl2::video::SwapInterval::VSync);
        let resources = Resources::new()?;
        Ok(Window {
               gui_context,
               sdl_window,
               _gl_context: gl_context,
               resources,
           })
    }

    pub fn size(&self) -> (u32, u32) { self.sdl_window.size() }

    pub fn options(&self) -> WindowOptions {
        WindowOptions {
            fullscreen: self.sdl_window.fullscreen_state() !=
                sdl2::video::FullscreenType::Off,
            resolution: self.size(),
        }
    }

    pub fn resources(&self) -> &Resources { &self.resources }

    pub fn poll_event(&mut self) -> Option<Event> {
        loop {
            let pump = &mut self.gui_context.event_pump;
            match pump.poll_event() {
                None => return None,
                Some(sdl_event) => {
                    if let Some(event) = Event::from_sdl_event(sdl_event,
                                                               pump)
                    {
                        return Some(event);
                    }
                }
            }
        }
    }

    pub fn swap(&mut self) { self.sdl_window.gl_swap_window(); }

    pub fn pump_audio(&mut self, audio: &mut AudioQueue) {
        let mut audio_queue = self.gui_context.audio_queue.lock().unwrap();
        audio_queue.merge(mem::replace(audio, AudioQueue::new()));
    }
}

//===========================================================================//
