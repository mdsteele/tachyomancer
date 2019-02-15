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

use cgmath::{Matrix4, Point2};
use tachy::font::Align;
use tachy::geom::{Color4, Rect};
use tachy::gui::{ClockEventData, Event, Keycode, Resources};
use tachy::save::Hotkey;
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const CHECKBOX_BOX_SIZE: i32 = 28;
const CHECKBOX_BOX_SPACING: i32 = 8;
const CHECKBOX_FONT_SIZE: f32 = 20.0;

const HOTKEY_BOX_HEIGHT: i32 = 28;
const HOTKEY_BOX_WIDTH: i32 = 68;
const HOTKEY_BOX_SPACING: i32 = 8;
const HOTKEY_BOX_FONT_SIZE: f32 = 20.0;
const HOTKEY_LABEL_FONT_SIZE: f32 = 20.0;

const HOVER_PULSE_CLICK: f64 = 1.0;
const HOVER_PULSE_DECAY_RATE: f64 = HOVER_PULSE_CLICK / 0.7;
const HOVER_PULSE_MAX: f64 = 0.55;
const HOVER_PULSE_MIN: f64 = 0.35;
const HOVER_PULSE_PERIOD: f64 = 1.0;

const TEXT_BOX_CURSOR_BLINK_PERIOD: f64 = 1.0;
const TEXT_BOX_FONT_SIZE: f32 = 20.0;
const TEXT_BOX_INNER_MARGIN: f32 = 5.0;

const TEXT_BUTTON_FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct Checkbox {
    rect: Rect<i32>,
    label: String,
    hover_pulse: HoverPulse,
}

impl Checkbox {
    pub fn new(mid_left: Point2<i32>, label: &str) -> Checkbox {
        let top = mid_left.y - CHECKBOX_BOX_SIZE / 2;
        let width = CHECKBOX_BOX_SIZE + CHECKBOX_BOX_SPACING +
            (0.5 * CHECKBOX_FONT_SIZE * (label.width() as f32)).ceil() as i32;
        Checkbox {
            rect: Rect::new(mid_left.x, top, width, CHECKBOX_BOX_SIZE),
            label: label.to_string(),
            hover_pulse: HoverPulse::new(),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                checked: bool, enabled: bool) {
        let ui = resources.shaders().ui();
        let box_rect = Rect::new(self.rect.x,
                                 self.rect.y,
                                 CHECKBOX_BOX_SIZE,
                                 CHECKBOX_BOX_SIZE);
        let bg_color = if !enabled {
            Color4::new(1.0, 1.0, 1.0, 0.1)
        } else {
            Color4::PURPLE0
                .mix(Color4::PURPLE3, self.hover_pulse.brightness())
                .with_alpha(0.8)
        };
        ui.draw_checkbox(matrix,
                         &box_rect.as_f32(),
                         &Color4::ORANGE4,
                         &Color4::CYAN5,
                         &bg_color,
                         checked);
        let font = resources.fonts().roman();
        font.draw(&matrix,
                  CHECKBOX_FONT_SIZE,
                  Align::MidLeft,
                  ((box_rect.x + CHECKBOX_BOX_SIZE +
                        CHECKBOX_BOX_SPACING) as f32,
                   (box_rect.y + CHECKBOX_BOX_SIZE / 2) as f32),
                  &self.label);
    }

    pub fn on_event(&mut self, event: &Event, checked: bool, enabled: bool)
                    -> Option<bool> {
        match event {
            Event::ClockTick(tick) => {
                self.hover_pulse.on_clock_tick(tick);
            }
            Event::MouseDown(mouse) => {
                if enabled && mouse.left &&
                    self.rect.contains_point(mouse.pt)
                {
                    self.hover_pulse.on_click();
                    // TOOD: play sound
                    return Some(!checked);
                }
            }
            Event::MouseMove(mouse) => {
                let hovering = self.rect.contains_point(mouse.pt);
                if self.hover_pulse.set_hovering(hovering) {
                    // TODO: play sound
                }
            }
            Event::Unfocus => self.hover_pulse.unfocus(),
            _ => {}
        }
        return None;
    }
}

//===========================================================================//

pub enum HotkeyBoxAction {
    Listening,
    Update(Keycode),
}

pub struct HotkeyBox {
    rect: Rect<i32>,
    hotkey: Hotkey,
    listening: bool,
    hover_pulse: HoverPulse,
}

impl HotkeyBox {
    pub fn new(mid_left: Point2<i32>, hotkey: Hotkey) -> HotkeyBox {
        let top = mid_left.y - HOTKEY_BOX_HEIGHT / 2;
        let width = HOTKEY_BOX_WIDTH + HOTKEY_BOX_SPACING +
            (0.5 * HOTKEY_LABEL_FONT_SIZE *
                (hotkey.name().width() as f32))
                .ceil() as i32;
        HotkeyBox {
            rect: Rect::new(mid_left.x, top, width, HOTKEY_BOX_HEIGHT),
            hotkey,
            listening: false,
            hover_pulse: HoverPulse::new(),
        }
    }

    pub fn hotkey(&self) -> Hotkey { self.hotkey }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                keycode: Keycode) {
        let ui = resources.shaders().ui();
        let box_rect = Rect::new(self.rect.x,
                                 self.rect.y,
                                 HOTKEY_BOX_WIDTH,
                                 HOTKEY_BOX_HEIGHT);
        let bg_color = if self.listening {
            Color4::PURPLE5
        } else {
            Color4::PURPLE0
                .mix(Color4::PURPLE3, self.hover_pulse.brightness())
                .with_alpha(0.8)
        };
        ui.draw_scroll_handle(matrix,
                              &box_rect.as_f32(),
                              &Color4::ORANGE4,
                              &Color4::CYAN5,
                              &bg_color);
        let font = resources.fonts().roman();
        if !self.listening {
            font.draw(&matrix,
                      HOTKEY_BOX_FONT_SIZE,
                      Align::MidCenter,
                      ((box_rect.x + box_rect.width / 2) as f32,
                       (box_rect.y + box_rect.height / 2) as f32),
                      Hotkey::keycode_name(keycode));
        }
        font.draw(&matrix,
                  HOTKEY_LABEL_FONT_SIZE,
                  Align::MidLeft,
                  ((box_rect.right() + HOTKEY_BOX_SPACING) as f32,
                   (box_rect.y + box_rect.height / 2) as f32),
                  self.hotkey.name());
    }

    pub fn on_event(&mut self, event: &Event) -> Option<HotkeyBoxAction> {
        match event {
            Event::ClockTick(tick) => {
                self.hover_pulse.on_clock_tick(tick);
            }
            Event::KeyDown(key) => {
                if self.listening && Hotkey::is_valid_keycode(key.code) {
                    self.listening = false;
                    return Some(HotkeyBoxAction::Update(key.code));
                }
            }
            Event::MouseDown(mouse) if mouse.left => {
                if self.rect.contains_point(mouse.pt) && !self.listening {
                    self.hover_pulse.on_click();
                    self.listening = true;
                    return Some(HotkeyBoxAction::Listening);
                } else {
                    self.listening = false;
                }
            }
            Event::MouseMove(mouse) => {
                let hovering = self.rect.contains_point(mouse.pt);
                if self.hover_pulse.set_hovering(hovering) {
                    if !self.listening {
                        // TODO: play sound
                    }
                }
            }
            Event::Unfocus => {
                self.listening = false;
                self.hover_pulse.unfocus();
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//

struct HoverPulse {
    hovering: bool,
    timer: f64,
    brightness: f64,
}

impl HoverPulse {
    pub fn new() -> HoverPulse {
        HoverPulse {
            hovering: false,
            timer: 0.0,
            brightness: 0.0,
        }
    }

    pub fn brightness(&self) -> f32 { self.brightness as f32 }

    pub fn on_click(&mut self) { self.brightness = HOVER_PULSE_CLICK; }

    pub fn on_clock_tick(&mut self, tick: &ClockEventData) {
        if self.hovering {
            if self.brightness > HOVER_PULSE_MAX {
                self.brightness = (self.brightness -
                                       tick.elapsed * HOVER_PULSE_DECAY_RATE)
                    .max(HOVER_PULSE_MAX);
                self.timer = 0.0;
            } else {
                self.timer = (self.timer + tick.elapsed) % HOVER_PULSE_PERIOD;
                let mut param = 2.0 * self.timer / HOVER_PULSE_PERIOD;
                if param > 1.0 {
                    param = 2.0 - param;
                }
                self.brightness = HOVER_PULSE_MAX -
                    param * (HOVER_PULSE_MAX - HOVER_PULSE_MIN);
            }
        } else {
            self.brightness = (self.brightness -
                                   tick.elapsed * HOVER_PULSE_DECAY_RATE)
                .max(0.0);
        }
    }

    pub fn set_hovering(&mut self, hovering: bool) -> bool {
        if hovering == self.hovering {
            false
        } else if hovering {
            self.hovering = true;
            self.brightness = self.brightness.max(HOVER_PULSE_MAX);
            self.timer = 0.0;
            true
        } else {
            self.unfocus();
            false
        }
    }

    pub fn unfocus(&mut self) {
        self.hovering = false;
        self.timer = 0.0;
    }
}

//===========================================================================//

pub struct RadioButton<T> {
    inner: TextButton<T>,
}

impl<T: Clone + PartialEq> RadioButton<T> {
    pub fn new(rect: Rect<i32>, label: &str, value: T) -> RadioButton<T> {
        RadioButton { inner: TextButton::new(rect, label, value) }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                value: &T) {
        self.inner.draw(resources, matrix, value != &self.inner.value);
    }

    pub fn on_event(&mut self, event: &Event, value: &T) -> Option<T> {
        let enabled = value != &self.inner.value;
        self.inner.on_event(event, enabled)
    }
}

//===========================================================================//

pub struct RadioCheckbox<T> {
    inner: Checkbox,
    value: T,
}

impl<T: Clone + PartialEq> RadioCheckbox<T> {
    pub fn new(mid_left: Point2<i32>, label: &str, value: T)
               -> RadioCheckbox<T> {
        RadioCheckbox {
            inner: Checkbox::new(mid_left, label),
            value,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                value: &T) {
        self.inner.draw(resources, matrix, value == &self.value, true);
    }

    pub fn on_event(&mut self, event: &Event, value: &T) -> Option<T> {
        let checked = value == &self.value;
        if let Some(true) = self.inner.on_event(event, checked, true) {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

//===========================================================================//

pub struct Scrollbar {
    rect: Rect<i32>,
    scroll_top: i32,
    scroll_max: i32,
    drag: Option<i32>,
}

impl Scrollbar {
    pub fn new(rect: Rect<i32>) -> Scrollbar {
        Scrollbar {
            rect,
            scroll_top: 0,
            scroll_max: 0,
            drag: None,
        }
    }

    pub fn is_visible(&self) -> bool { self.scroll_max != 0 }

    pub fn scroll_top(&self) -> i32 { self.scroll_top }

    pub fn set_total_height(&mut self, total_height: i32) {
        self.scroll_max = (total_height - self.rect.height).max(0);
        self.scroll_top = self.scroll_top.min(self.scroll_max);
    }

    pub fn scroll_by(&mut self, delta: i32) {
        let new_scroll_top = self.scroll_top + delta;
        self.scroll_top = new_scroll_top.max(0).min(self.scroll_max);
    }

    pub fn scroll_to(&mut self, middle: i32) {
        self.scroll_top =
            (middle - self.rect.height / 2).max(0).min(self.scroll_max);
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        if let Some(handle_rect) = self.handle_rect() {
            let ui = resources.shaders().ui();
            ui.draw_scroll_bar(matrix,
                               &self.rect.as_f32(),
                               &Color4::ORANGE3,
                               &Color4::CYAN2,
                               &Color4::PURPLE0);
            let (fg_color, bg_color) = if self.drag.is_some() {
                (&Color4::ORANGE4, &Color4::PURPLE3)
            } else {
                (&Color4::ORANGE3, &Color4::PURPLE1)
            };
            ui.draw_scroll_handle(matrix,
                                  &handle_rect.as_f32(),
                                  fg_color,
                                  &Color4::CYAN2,
                                  bg_color);
        }
    }

    pub fn on_event(&mut self, event: &Event) {
        match event {
            Event::MouseDown(mouse) if mouse.left => {
                if let Some(handle_rect) = self.handle_rect() {
                    if handle_rect.contains_point(mouse.pt) {
                        self.drag = Some(mouse.pt.y - handle_rect.y);
                    }
                    // TODO: support jumping up/down page
                }
            }
            Event::MouseMove(mouse) => {
                if let Some(drag_offset) = self.drag {
                    let new_handle_y = mouse.pt.y - drag_offset - self.rect.y;
                    let total_height = self.scroll_max + self.rect.height;
                    let new_scroll_top = div_round(total_height *
                                                       new_handle_y,
                                                   self.rect.height);
                    self.scroll_top =
                        new_scroll_top.max(0).min(self.scroll_max);
                }
            }
            Event::MouseUp(mouse) if mouse.left => {
                self.drag = None;
            }
            Event::Unfocus => {
                self.drag = None;
            }
            _ => {}
        }
    }

    fn handle_rect(&self) -> Option<Rect<i32>> {
        if self.scroll_max != 0 {
            let total_height = self.scroll_max + self.rect.height;
            Some(Rect::new(self.rect.x,
                           self.rect.y +
                               div_round(self.rect.height * self.scroll_top,
                                         total_height),
                           self.rect.width,
                           div_round(self.rect.height * self.rect.height,
                                     total_height)))
        } else {
            None
        }
    }
}

//===========================================================================//

pub enum SliderAction {
    Update(i32),
    Release,
}

pub struct Slider {
    rect: Rect<i32>,
    value: i32,
    maximum: i32,
    drag: Option<(i32, i32)>,
    hover_pulse: HoverPulse,
}

impl Slider {
    pub fn new(rect: Rect<i32>, value: i32, maximum: i32) -> Slider {
        debug_assert!(rect.width > rect.height);
        let maximum = maximum.max(1);
        Slider {
            rect,
            value: value.max(0).min(maximum),
            maximum,
            drag: None,
            hover_pulse: HoverPulse::new(),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let ui = resources.shaders().ui();
        ui.draw_scroll_bar(matrix,
                           &self.rect.as_f32(),
                           &Color4::ORANGE3,
                           &Color4::CYAN2,
                           &Color4::PURPLE0);
        let (fg_color, bg_color) = if self.drag.is_some() {
            (&Color4::ORANGE5, Color4::PURPLE3)
        } else {
            (&Color4::ORANGE4,
             Color4::PURPLE1
                 .mix(Color4::PURPLE3, self.hover_pulse.brightness()))
        };
        ui.draw_scroll_handle(matrix,
                              &self.handle_rect().as_f32(),
                              fg_color,
                              &Color4::CYAN2,
                              &bg_color);
    }

    pub fn on_event(&mut self, event: &Event) -> Option<SliderAction> {
        match event {
            Event::ClockTick(tick) => {
                self.hover_pulse.on_clock_tick(tick);
            }
            Event::MouseDown(mouse) => {
                if mouse.left && self.handle_rect().contains_point(mouse.pt) {
                    self.hover_pulse.on_click();
                    self.drag = Some((mouse.pt.x, 0));
                }
            }
            Event::MouseMove(mouse) => {
                if let Some((start, _)) = self.drag.take() {
                    let old_left = self.handle_left();
                    let delta = mouse.pt.x - start;
                    let range = self.rect.width - self.rect.height;
                    let value = div_round(range * self.value +
                                              delta * self.maximum,
                                          range);
                    let value = value.max(0).min(self.maximum);
                    if value != self.value {
                        self.value = value.max(0).min(self.maximum);
                        let new_left = self.handle_left();
                        let new_start = start + new_left - old_left;
                        let new_delta = mouse.pt.x - new_start;
                        self.drag = Some((new_start, new_delta));
                        return Some(SliderAction::Update(self.value));
                    } else {
                        self.drag = Some((start, delta));
                    }
                } else {
                    let hovering = self.handle_rect().contains_point(mouse.pt);
                    if self.hover_pulse.set_hovering(hovering) {
                        // TODO: play sound
                    }
                }
            }
            Event::MouseUp(_) => {
                if self.drag.take().is_some() {
                    return Some(SliderAction::Release);
                }
            }
            Event::Unfocus => {
                self.drag = None;
                self.hover_pulse.unfocus();
            }
            _ => {}
        }
        return None;
    }

    fn handle_left(&self) -> i32 {
        self.rect.x +
            div_round((self.rect.width - self.rect.height) * self.value,
                      self.maximum)
    }

    fn handle_rect(&self) -> Rect<i32> {
        let mut left = self.handle_left();
        if let Some((_, delta)) = self.drag {
            left = (left + delta)
                .max(self.rect.x)
                .min(self.rect.right() - self.rect.height);
        }
        Rect::new(left, self.rect.y, self.rect.height, self.rect.height)
    }
}

//===========================================================================//

pub struct TextBox {
    rect: Rect<i32>,
    string: String,
    max_len: usize,
    cursor_byte: usize,
    cursor_char: usize,
    cursor_blink: f64,
}

impl TextBox {
    pub fn new(rect: Rect<i32>, initial: &str, max_len: usize) -> TextBox {
        TextBox {
            rect,
            string: initial.to_string(),
            max_len,
            cursor_byte: initial.len(),
            cursor_char: initial.width(),
            cursor_blink: 0.0,
        }
    }

    pub fn string(&self) -> &str { &self.string }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Box:
        let color = (0.0, 0.0, 0.0);
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        // Text:
        let font = resources.fonts().roman();
        font.draw(&matrix,
                  TEXT_BOX_FONT_SIZE,
                  Align::MidLeft,
                  (rect.x + TEXT_BOX_INNER_MARGIN,
                   rect.y + 0.5 * rect.height),
                  &self.string);
        // Cursor:
        if self.cursor_blink < 0.5 * TEXT_BOX_CURSOR_BLINK_PERIOD {
            let color = (0.5, 0.5, 0.0);
            let cursor_rect =
                Rect::new(rect.x + TEXT_BOX_INNER_MARGIN +
                              0.5 * TEXT_BOX_FONT_SIZE *
                                  self.cursor_char as f32,
                          rect.y + 0.5 * (rect.height - TEXT_BOX_FONT_SIZE),
                          1.0,
                          TEXT_BOX_FONT_SIZE);
            resources.shaders().solid().fill_rect(&matrix, color, cursor_rect);
        }
    }

    pub fn on_event(&mut self, event: &Event) {
        match event {
            Event::ClockTick(tick) => {
                self.cursor_blink = (self.cursor_blink + tick.elapsed) %
                    TEXT_BOX_CURSOR_BLINK_PERIOD;
            }
            Event::KeyDown(key) => {
                match key.code {
                    Keycode::Backspace => {
                        let rest = self.string.split_off(self.cursor_byte);
                        if let Some(chr) = self.string.pop() {
                            self.cursor_byte -= chr.len_utf8();
                            self.cursor_char -= 1;
                            self.cursor_blink = 0.0;
                            // TODO: play sound
                        }
                        self.string.push_str(&rest);
                    }
                    Keycode::Delete => {
                        if self.cursor_byte < self.string.len() {
                            self.string.remove(self.cursor_byte);
                            // TODO: play sound
                        }
                    }
                    Keycode::Up | Keycode::PageUp | Keycode::Home => {
                        self.cursor_byte = 0;
                        self.cursor_char = 0;
                        self.cursor_blink = 0.0;
                    }
                    Keycode::Down | Keycode::PageDown | Keycode::End => {
                        self.cursor_byte = self.string.len();
                        self.cursor_char = self.string.width();
                        self.cursor_blink = 0.0;
                    }
                    Keycode::Left => {
                        let (part, _) = self.string.split_at(self.cursor_byte);
                        if let Some(chr) = part.chars().next_back() {
                            self.cursor_byte -= chr.len_utf8();
                            self.cursor_char -= 1;
                            self.cursor_blink = 0.0;
                        }
                    }
                    Keycode::Right => {
                        let (_, part) = self.string.split_at(self.cursor_byte);
                        if let Some(chr) = part.chars().next() {
                            self.cursor_byte += chr.len_utf8();
                            self.cursor_char += 1;
                            self.cursor_blink = 0.0;
                        }
                    }
                    _ => {}
                }
            }
            Event::TextInput(text) => {
                for chr in text.chars() {
                    if self.string.width() >= self.max_len {
                        break;
                    }
                    if (chr >= ' ' && chr <= '~') ||
                        (chr >= '\u{a1}' && chr <= '\u{ff}')
                    {
                        self.string.insert(self.cursor_byte, chr);
                        self.cursor_byte += chr.len_utf8();
                        self.cursor_char += 1;
                        self.cursor_blink = 0.0;
                        // TODO: play sound
                    }
                }
            }
            _ => {}
        }
    }
}

//===========================================================================//

pub struct TextButton<T> {
    rect: Rect<i32>,
    label: String,
    value: T,
    keycode: Option<Keycode>,
    hover_pulse: HoverPulse,
}

impl<T: Clone> TextButton<T> {
    pub fn new(rect: Rect<i32>, label: &str, value: T) -> TextButton<T> {
        TextButton::new_with_key(rect, label, value, None)
    }

    pub fn new_with_key(rect: Rect<i32>, label: &str, value: T,
                        keycode: Option<Keycode>)
                        -> TextButton<T> {
        TextButton {
            rect,
            label: label.to_string(),
            value,
            keycode,
            hover_pulse: HoverPulse::new(),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                enabled: bool) {
        let bg_color = if !enabled {
            Color4::new(1.0, 1.0, 1.0, 0.1)
        } else {
            Color4::PURPLE0
                .mix(Color4::PURPLE3, self.hover_pulse.brightness())
                .with_alpha(0.8)
        };
        let rect = self.rect.as_f32();
        resources.shaders().ui().draw_box4(&matrix,
                                           &rect,
                                           &Color4::ORANGE5,
                                           &Color4::CYAN3,
                                           &bg_color);
        resources.fonts().bold().draw(&matrix,
                                      TEXT_BUTTON_FONT_SIZE,
                                      Align::MidCenter,
                                      (rect.x + 0.5 * rect.width,
                                       rect.y + 0.5 * rect.height),
                                      &self.label);
    }

    pub fn on_event(&mut self, event: &Event, enabled: bool) -> Option<T> {
        match event {
            Event::ClockTick(tick) => {
                self.hover_pulse.on_clock_tick(tick);
            }
            Event::KeyDown(key) => {
                if enabled && Some(key.code) == self.keycode {
                    self.hover_pulse.on_click();
                    // TODO: play sound
                    return Some(self.value.clone());
                }
            }
            Event::MouseDown(mouse) => {
                if enabled && mouse.left &&
                    self.rect.contains_point(mouse.pt)
                {
                    self.hover_pulse.on_click();
                    // TODO: play sound
                    return Some(self.value.clone());
                }
            }
            Event::MouseMove(mouse) => {
                let hovering = self.rect.contains_point(mouse.pt);
                if self.hover_pulse.set_hovering(hovering) {
                    // TODO: play sound
                }
            }
            Event::Unfocus => self.hover_pulse.unfocus(),
            _ => {}
        }
        return None;
    }
}

//===========================================================================//

fn div_round(a: i32, b: i32) -> i32 {
    ((a as f64) / (b as f64)).round() as i32
}

//===========================================================================//
