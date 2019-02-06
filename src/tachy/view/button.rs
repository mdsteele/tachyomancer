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
use tachy::geom::Rect;
use tachy::gui::{Event, Keycode, Resources};
use tachy::save::Hotkey;
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const CHECKBOX_BOX_SIZE: i32 = 28;
const CHECKBOX_BOX_SPACING: i32 = 8;
const CHECKBOX_CHECK_PADDING: i32 = 4;
const CHECKBOX_CHECK_SIZE: i32 = CHECKBOX_BOX_SIZE -
    2 * CHECKBOX_CHECK_PADDING;
const CHECKBOX_FONT_SIZE: f32 = 20.0;

const HOTKEY_BOX_HEIGHT: i32 = 28;
const HOTKEY_BOX_WIDTH: i32 = 68;
const HOTKEY_BOX_SPACING: i32 = 8;
const HOTKEY_BOX_FONT_SIZE: f32 = 20.0;
const HOTKEY_LABEL_FONT_SIZE: f32 = 20.0;

const TEXT_BOX_CURSOR_BLINK_PERIOD: f64 = 1.0;
const TEXT_BOX_FONT_SIZE: f32 = 20.0;
const TEXT_BOX_INNER_MARGIN: f32 = 5.0;

const TEXT_BUTTON_FONT_SIZE: f32 = 20.0;

//===========================================================================//

pub struct Checkbox {
    rect: Rect<i32>,
    label: String,
    hovering: bool,
}

impl Checkbox {
    pub fn new(mid_left: Point2<i32>, label: &str) -> Checkbox {
        let top = mid_left.y - CHECKBOX_BOX_SIZE / 2;
        let width = CHECKBOX_BOX_SIZE + CHECKBOX_BOX_SPACING +
            (0.5 * CHECKBOX_FONT_SIZE * (label.width() as f32)).ceil() as i32;
        Checkbox {
            rect: Rect::new(mid_left.x, top, width, CHECKBOX_BOX_SIZE),
            label: label.to_string(),
            hovering: false,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                checked: bool, enabled: bool) {
        let box_rect = Rect::new(self.rect.x,
                                 self.rect.y,
                                 CHECKBOX_BOX_SIZE,
                                 CHECKBOX_BOX_SIZE);
        let (box_color, check_color) = if !enabled {
            ((0.4, 0.4, 0.4), (0.8, 0.8, 0.8))
        } else if self.hovering {
            ((1.0, 0.2, 0.2), (1.0, 0.8, 0.8))
        } else {
            ((0.7, 0.1, 0.1), (1.0, 0.5, 0.5))
        };
        resources
            .shaders()
            .solid()
            .fill_rect(&matrix, box_color, box_rect.as_f32());
        if checked {
            let check_rect = Rect::new(box_rect.x + CHECKBOX_CHECK_PADDING,
                                       box_rect.y + CHECKBOX_CHECK_PADDING,
                                       CHECKBOX_CHECK_SIZE,
                                       CHECKBOX_CHECK_SIZE);
            resources
                .shaders()
                .solid()
                .fill_rect(&matrix, check_color, check_rect.as_f32());
        }
        resources.fonts().roman().draw(&matrix,
                                       CHECKBOX_FONT_SIZE,
                                       Align::MidLeft,
                                       ((box_rect.x + CHECKBOX_BOX_SIZE +
                                             CHECKBOX_BOX_SPACING) as
                                            f32,
                                        (box_rect.y + CHECKBOX_BOX_SIZE / 2) as
                                            f32),
                                       &self.label);
    }

    pub fn on_event(&mut self, event: &Event, checked: bool, enabled: bool)
                    -> Option<bool> {
        match event {
            Event::MouseDown(mouse) => {
                if enabled && mouse.left &&
                    self.rect.contains_point(mouse.pt)
                {
                    return Some(!checked);
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
            }
            Event::Unfocus => self.hovering = false,
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
    hovering: bool,
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
            hovering: false,
        }
    }

    pub fn hotkey(&self) -> Hotkey { self.hotkey }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                keycode: Keycode) {
        let box_rect = Rect::new(self.rect.x,
                                 self.rect.y,
                                 HOTKEY_BOX_WIDTH,
                                 HOTKEY_BOX_HEIGHT);
        let box_color = if self.listening {
            (1.0, 1.0, 0.4)
        } else if self.hovering {
            (1.0, 0.2, 0.2)
        } else {
            (0.7, 0.1, 0.1)
        };
        resources
            .shaders()
            .solid()
            .fill_rect(&matrix, box_color, box_rect.as_f32());
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
            Event::KeyDown(key) => {
                if self.listening && Hotkey::is_valid_keycode(key.code) {
                    self.listening = false;
                    return Some(HotkeyBoxAction::Update(key.code));
                }
            }
            Event::MouseDown(mouse) if mouse.left => {
                if self.rect.contains_point(mouse.pt) {
                    self.listening = true;
                    return Some(HotkeyBoxAction::Listening);
                } else {
                    self.listening = false;
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
            }
            Event::Unfocus => {
                self.listening = false;
                self.hovering = false;
            }
            _ => {}
        }
        return None;
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

pub enum SliderAction {
    Update(i32),
    Release,
}

pub struct Slider {
    rect: Rect<i32>,
    value: i32,
    maximum: i32,
    drag: Option<(i32, i32)>,
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
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = (0.0, 0.2, 0.2);
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);

        let color = if self.drag.is_some() {
            (0.2, 0.8, 0.8)
        } else {
            (0.1, 0.6, 0.6)
        };
        let rect = self.handle_rect().as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
    }

    pub fn on_event(&mut self, event: &Event) -> Option<SliderAction> {
        match event {
            Event::MouseDown(mouse) => {
                if mouse.left && self.handle_rect().contains_point(mouse.pt) {
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
                }
            }
            Event::MouseUp(_) => {
                if self.drag.take().is_some() {
                    return Some(SliderAction::Release);
                }
            }
            Event::Unfocus => self.drag = None,
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
        resources.fonts().roman().draw(&matrix,
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
    hovering: bool,
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
            hovering: false,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                enabled: bool) {
        let color = if !enabled {
            (0.4, 0.4, 0.4)
        } else if self.hovering {
            (1.0, 0.2, 0.2)
        } else {
            (0.7, 0.1, 0.1)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().bold().draw(&matrix,
                                      TEXT_BUTTON_FONT_SIZE,
                                      Align::MidCenter,
                                      (rect.x + 0.5 * rect.width,
                                       rect.y + 0.5 * rect.height),
                                      &self.label);
    }

    pub fn on_event(&mut self, event: &Event, enabled: bool) -> Option<T> {
        match event {
            Event::KeyDown(key) => {
                if enabled && Some(key.code) == self.keycode {
                    // TODO: play sound
                    return Some(self.value.clone());
                }
            }
            Event::MouseDown(mouse) => {
                if enabled && mouse.left &&
                    self.rect.contains_point(mouse.pt)
                {
                    // TODO: play sound
                    return Some(self.value.clone());
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
            }
            Event::Unfocus => self.hovering = false,
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
