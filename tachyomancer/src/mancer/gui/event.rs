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

use cgmath::{Point2, Rad, Vector2};
use sdl2;
pub use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::time::Duration;

//===========================================================================//

const MAX_CLOCK_TICK_SECONDS: f64 = 1.0 / 30.0;

const MULTITOUCH_SCALE_BASE: f32 = 5.0;

const SCROLL_DELTA_MULTIPLIER: i32 = 5;

//===========================================================================//

#[derive(Clone)]
pub enum Event {
    ClockTick(ClockEventData),
    Debug(String, String),
    KeyDown(KeyEventData),
    MouseDown(MouseEventData),
    MouseMove(MouseEventData),
    MouseUp(MouseEventData),
    Multitouch(MultitouchEventData),
    Quit,
    Redraw,
    Scroll(ScrollEventData),
    TextInput(String),
    Unfocus,
}

impl Event {
    pub fn new_clock_tick(elapsed: Duration) -> Event {
        let seconds = (elapsed.as_secs() as f64)
            + 1e-9 * (elapsed.subsec_nanos() as f64);
        let data =
            ClockEventData { elapsed: seconds.min(MAX_CLOCK_TICK_SECONDS) };
        Event::ClockTick(data)
    }

    pub fn new_debug(line: &str) -> Event {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().unwrap_or("").trim().to_string();
        let value = parts.next().unwrap_or("").trim().to_string();
        Event::Debug(key, value)
    }

    pub(super) fn from_sdl_event(
        sdl_event: sdl2::event::Event,
        pump: &sdl2::EventPump,
    ) -> Option<Event> {
        // TODO: On iOS/Android, treat long-press as right-click.
        match sdl_event {
            sdl2::event::Event::KeyDown { keycode, keymod, .. } => {
                if let Some(code) = keycode {
                    let mouse = pump.mouse_state();
                    let mouse_pt = Point2::new(mouse.x(), mouse.y());
                    let data = KeyEventData::new(code, keymod, mouse_pt);
                    if data.code == Keycode::Q && data.command {
                        Some(Event::Quit)
                    } else {
                        Some(Event::KeyDown(data))
                    }
                } else {
                    None
                }
            }
            sdl2::event::Event::MouseButtonDown {
                x, y, mouse_btn, ..
            } => match mouse_btn {
                MouseButton::Left | MouseButton::Right => {
                    let data = MouseEventData::new(x, y, mouse_btn);
                    Some(Event::MouseDown(data))
                }
                _ => None,
            },
            sdl2::event::Event::MouseButtonUp { x, y, mouse_btn, .. } => {
                match mouse_btn {
                    MouseButton::Left | MouseButton::Right => {
                        let data = MouseEventData::new(x, y, mouse_btn);
                        Some(Event::MouseUp(data))
                    }
                    _ => None,
                }
            }
            sdl2::event::Event::MouseMotion { x, y, mousestate, .. } => {
                let data = MouseEventData {
                    pt: Point2 { x, y },
                    left: mousestate.left(),
                    right: mousestate.right(),
                };
                Some(Event::MouseMove(data))
            }
            sdl2::event::Event::MouseWheel { x, y, .. } => {
                let mouse = pump.mouse_state();
                let data = ScrollEventData {
                    pt: Point2::new(mouse.x(), mouse.y()),
                    delta: Vector2::new(x, -y) * SCROLL_DELTA_MULTIPLIER,
                };
                Some(Event::Scroll(data))
            }
            sdl2::event::Event::MultiGesture { d_dist, d_theta, .. } => {
                let mouse = pump.mouse_state();
                let data = MultitouchEventData {
                    pt: Point2::new(mouse.x(), mouse.y()),
                    scale: MULTITOUCH_SCALE_BASE.powf(d_dist),
                    rotate: Rad(d_theta),
                };
                Some(Event::Multitouch(data))
            }
            sdl2::event::Event::Quit { .. } => Some(Event::Quit),
            sdl2::event::Event::TextInput { text, .. } => {
                Some(Event::TextInput(text))
            }
            sdl2::event::Event::Window { win_event, .. } => match win_event {
                sdl2::event::WindowEvent::FocusGained => {
                    let mouse = pump.mouse_state();
                    let data = MouseEventData {
                        pt: Point2::new(mouse.x(), mouse.y()),
                        left: mouse.left(),
                        right: mouse.right(),
                    };
                    Some(Event::MouseMove(data))
                }
                sdl2::event::WindowEvent::FocusLost => Some(Event::Unfocus),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_clock_tick(&self) -> bool {
        match self {
            Event::ClockTick(_) => true,
            _ => false,
        }
    }

    pub fn is_mouse(&self) -> bool {
        match self {
            Event::MouseDown(_) | Event::MouseMove(_) | Event::MouseUp(_) => {
                true
            }
            _ => false,
        }
    }

    pub fn relative_to(&self, origin: Point2<i32>) -> Event {
        match self {
            Event::MouseDown(mouse) => {
                Event::MouseDown(mouse.relative_to(origin))
            }
            Event::MouseMove(mouse) => {
                Event::MouseMove(mouse.relative_to(origin))
            }
            Event::MouseUp(mouse) => Event::MouseUp(mouse.relative_to(origin)),
            Event::Multitouch(touch) => {
                Event::Multitouch(touch.relative_to(origin))
            }
            Event::Scroll(scroll) => Event::Scroll(scroll.relative_to(origin)),
            _ => self.clone(),
        }
    }
}

//===========================================================================//

#[derive(Clone)]
pub struct ClockEventData {
    /// The time elapsed since the last clock tick, in seconds.
    pub elapsed: f64,
}

//===========================================================================//

#[derive(Clone)]
pub struct KeyEventData {
    pub code: Keycode,
    pub command: bool,
    pub shift: bool,
    pub mouse_pt: Point2<i32>,
}

impl KeyEventData {
    fn new(
        keycode: Keycode,
        keymod: sdl2::keyboard::Mod,
        mouse_pt: Point2<i32>,
    ) -> KeyEventData {
        let shift =
            sdl2::keyboard::Mod::LSHIFTMOD | sdl2::keyboard::Mod::RSHIFTMOD;
        let command = if cfg!(any(target_os = "ios", target_os = "macos")) {
            sdl2::keyboard::Mod::LGUIMOD | sdl2::keyboard::Mod::RGUIMOD
        } else {
            sdl2::keyboard::Mod::LCTRLMOD | sdl2::keyboard::Mod::RCTRLMOD
        };
        KeyEventData {
            code: keycode,
            shift: keymod.intersects(shift),
            command: keymod.intersects(command),
            mouse_pt,
        }
    }
}

//===========================================================================//

#[derive(Clone)]
pub struct MouseEventData {
    pub pt: Point2<i32>,
    pub left: bool,
    pub right: bool,
}

impl MouseEventData {
    fn new(x: i32, y: i32, button: MouseButton) -> MouseEventData {
        MouseEventData {
            pt: Point2 { x, y },
            left: button == MouseButton::Left,
            right: button == MouseButton::Right,
        }
    }

    fn relative_to(&self, origin: Point2<i32>) -> MouseEventData {
        MouseEventData {
            pt: Point2 { x: self.pt.x - origin.x, y: self.pt.y - origin.y },
            left: self.left,
            right: self.right,
        }
    }
}

//===========================================================================//

#[derive(Clone)]
pub struct MultitouchEventData {
    pub pt: Point2<i32>,
    pub scale: f32,
    pub rotate: Rad<f32>,
}

impl MultitouchEventData {
    fn relative_to(&self, origin: Point2<i32>) -> MultitouchEventData {
        MultitouchEventData {
            pt: Point2 { x: self.pt.x - origin.x, y: self.pt.y - origin.y },
            scale: self.scale,
            rotate: self.rotate,
        }
    }
}

//===========================================================================//

#[derive(Clone)]
pub struct ScrollEventData {
    pub pt: Point2<i32>,
    pub delta: Vector2<i32>,
}

impl ScrollEventData {
    fn relative_to(&self, origin: Point2<i32>) -> ScrollEventData {
        ScrollEventData {
            pt: Point2 { x: self.pt.x - origin.x, y: self.pt.y - origin.y },
            delta: self.delta,
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Event, MAX_CLOCK_TICK_SECONDS};
    use std::time::Duration;

    #[test]
    fn make_clock_tick_event() {
        assert!(MAX_CLOCK_TICK_SECONDS > 0.025);
        match Event::new_clock_tick(Duration::from_millis(25)) {
            Event::ClockTick(tick) => {
                assert_eq!(tick.elapsed, 0.025);
            }
            _ => panic!("wrong event type"),
        }

        assert!(MAX_CLOCK_TICK_SECONDS < 1.5);
        match Event::new_clock_tick(Duration::from_millis(1500)) {
            Event::ClockTick(tick) => {
                assert_eq!(tick.elapsed, MAX_CLOCK_TICK_SECONDS);
            }
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn make_debug_event() {
        match Event::new_debug("foobar = baz = quux\n") {
            Event::Debug(key, value) => {
                assert_eq!(key, "foobar");
                assert_eq!(value, "baz = quux");
            }
            _ => panic!("wrong event type"),
        }

        match Event::new_debug(" foo-bar ") {
            Event::Debug(key, value) => {
                assert_eq!(key, "foo-bar");
                assert_eq!(value, "");
            }
            _ => panic!("wrong event type"),
        }
    }
}

//===========================================================================//
