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

use super::circuit::ParserCircuitPiece;
use super::compile::Compiler;
use super::text::ParserTextPiece;
use super::types::{CompiledLine, ParserAlign, ParserPiece, ParserPieceSplit};
use crate::mancer::font::Font;
use crate::mancer::save::{Hotkey, Prefs};
use std::mem;
use std::str::FromStr;
use tachy::geom::{Color4, CoordsDelta, Orientation, RectSize};
use tachy::save::{ChipType, CircuitData};

//===========================================================================//

const DEFAULT_ALIGN: ParserAlign = ParserAlign::Left;
const DEFAULT_COLOR: Color4 = Color4::WHITE;
const DEFAULT_FONT: Font = Font::Roman;
const DEFAULT_MILLIS_PER_CHAR: usize = 30;

//===========================================================================//

pub struct Parser {
    current_align: ParserAlign,
    current_color: Color4,
    current_font: Font,
    current_italic: bool,
    current_line: ParserLine,
    current_millis_per_char: usize,
    current_piece: Vec<u8>,
    lines: Vec<ParserLine>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            current_align: DEFAULT_ALIGN,
            current_color: DEFAULT_COLOR,
            current_font: DEFAULT_FONT,
            current_italic: false,
            current_line: ParserLine::new(),
            current_millis_per_char: DEFAULT_MILLIS_PER_CHAR,
            current_piece: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn push_char(&mut self, chr: char) {
        self.current_piece.push(chr as u8);
    }

    fn push_str(&mut self, string: &str) {
        self.current_piece.extend(string.chars().map(|chr| chr as u8));
    }

    pub fn push_phrase(&mut self, phrase_name: &str, prefs: &Prefs) {
        let phrase = match phrase_name {
            "Command" => {
                if cfg!(any(target_os = "ios", target_os = "macos")) {
                    "Command"
                } else {
                    "Control"
                }
            }
            "Right-click" => {
                if cfg!(any(target_os = "android", target_os = "ios")) {
                    "Long-press"
                } else if cfg!(target_os = "macos") {
                    "Control-click"
                } else {
                    "Right-click"
                }
            }
            "YOURNAME" => prefs.current_profile().unwrap_or("YOURNAME"),
            _ => {
                debug_warn!(
                    "Bad phrase name {:?} in paragraph format string",
                    phrase_name
                );
                phrase_name
            }
        };
        self.push_str(phrase);
    }

    pub fn push_key(&mut self, hotkey_name: &str, prefs: &Prefs) {
        if let Ok(hotkey) = Hotkey::from_str(hotkey_name) {
            let key_name = prefs.hotkey_code(hotkey).name();
            self.push_str(&format!("[{}]", key_name));
        } else {
            debug_warn!(
                "Bad hotkey name {:?} in paragraph format string",
                hotkey_name
            );
        }
    }

    pub fn push_chip(&mut self, orient_str: &str, ctype_str: &str) {
        let orient = match Orientation::from_str(orient_str) {
            Ok(orient) => orient,
            Err(_) => {
                debug_warn!(
                    "Bad orientation {:?} in paragraph format string",
                    orient_str
                );
                return;
            }
        };
        let ctype = match ChipType::from_str(ctype_str) {
            Ok(ctype) => ctype,
            Err(_) => {
                debug_warn!(
                    "Bad chip type {:?} in paragraph format string",
                    ctype_str
                );
                return;
            }
        };
        let size = orient * ctype.size();
        let mut data = CircuitData::new(size.width, size.height);
        data.chips.insert(CoordsDelta::new(0, 0), ctype, orient);
        self.push_circuit_data(data);
    }

    pub fn push_circuit(&mut self, circuit_str: &str) {
        match CircuitData::deserialize_from_string(circuit_str) {
            Ok(data) => self.push_circuit_data(data),
            Err(_) => {
                debug_warn!("Bad circuit TOML in paragraph format string");
            }
        }
    }

    fn push_circuit_data(&mut self, data: CircuitData) {
        self.shift_text_piece();
        let piece =
            ParserCircuitPiece::new(self.current_millis_per_char, data);
        self.push_piece(Box::new(piece));
    }

    pub fn newline(&mut self) {
        self.shift_text_piece();
        let line = mem::replace(&mut self.current_line, ParserLine::new());
        self.lines.push(line);
    }

    pub fn set_align(&mut self, align: ParserAlign) {
        if align != self.current_align {
            self.shift_text_piece();
            self.current_align = align;
        }
    }

    pub fn set_color(&mut self, color: Color4) {
        if color != self.current_color {
            self.shift_text_piece();
            self.current_color = color.into();
        }
    }

    pub fn set_color_to_default(&mut self) {
        self.set_color(DEFAULT_COLOR);
    }

    pub fn set_font(&mut self, font_name: &str) {
        if let Ok(font) = Font::from_str(font_name) {
            if font != self.current_font {
                self.shift_text_piece();
                self.current_font = font;
            }
        } else {
            debug_warn!(
                "Bad font name {:?} in paragraph format string",
                font_name
            );
        }
    }

    pub fn set_millis_per_char(&mut self, number_string: &str) {
        if let Ok(number) = number_string.parse::<usize>() {
            if number != self.current_millis_per_char {
                self.shift_text_piece();
                self.current_millis_per_char = number;
            }
        } else {
            debug_warn!(
                "Bad number {:?} in paragraph format string",
                number_string
            );
        }
    }

    pub fn set_wrap_indent_to_here(&mut self, font_size: f32) {
        self.shift_text_piece();
        let pieces = match self.current_align {
            ParserAlign::Left => &self.current_line.left,
            ParserAlign::Center => &self.current_line.center,
            ParserAlign::Right => &self.current_line.right,
        };
        self.current_line.wrap_indent =
            pieces.iter().map(|piece| piece.width(font_size)).sum();
    }

    pub fn toggle_bold(&mut self) {
        let next_font = match self.current_font {
            Font::Bold => Font::Roman,
            Font::Roman => Font::Bold,
            _ => return,
        };
        self.shift_text_piece();
        self.current_font = next_font;
    }

    pub fn toggle_italic(&mut self) {
        self.shift_text_piece();
        self.current_italic = !self.current_italic;
    }

    fn shift_text_piece(&mut self) {
        if !self.current_piece.is_empty() {
            let slant = if self.current_italic { 0.5 } else { 0.0 };
            let chars = mem::replace(&mut self.current_piece, Vec::new());
            let piece = ParserTextPiece::new(
                self.current_font,
                self.current_color,
                slant,
                self.current_millis_per_char,
                chars,
            );
            self.push_piece(Box::new(piece));
        }
    }

    fn push_piece(&mut self, piece: Box<dyn ParserPiece>) {
        let pieces = match self.current_align {
            ParserAlign::Left => &mut self.current_line.left,
            ParserAlign::Center => &mut self.current_line.center,
            ParserAlign::Right => &mut self.current_line.right,
        };
        pieces.push(piece);
    }

    pub fn compile(
        mut self,
        font_size: f32,
        line_gap: f32,
        max_width: f32,
    ) -> (RectSize<f32>, usize, Vec<CompiledLine>) {
        debug_assert!(font_size > 0.0);
        debug_assert!(max_width >= 0.0);
        self.shift_text_piece();
        if !self.current_line.is_empty() {
            self.newline();
        }
        let mut compiler = Compiler::new(font_size, line_gap, max_width);
        for line in self.lines {
            let wrap_indent = line.wrap_indent;
            for (align, pieces) in line.columns() {
                let mut pieces = pieces.into_iter();
                let mut next_piece = pieces.next();
                while let Some(mut pp1) = next_piece.take() {
                    let remaining_width = max_width - compiler.offset();
                    match pp1.split(font_size, remaining_width) {
                        ParserPieceSplit::AllFits => {
                            compiler.push(pp1);
                            next_piece = pieces.next();
                        }
                        ParserPieceSplit::SomeFits(pp2) => {
                            if !pp1.is_empty() {
                                compiler.push(pp1);
                            }
                            compiler.fix_x_offsets(align);
                            compiler.newline(wrap_indent);
                            next_piece = Some(pp2);
                        }
                        ParserPieceSplit::NoneFits(opt_pp2) => {
                            let line_was_empty =
                                compiler.current_line_is_empty();
                            if !line_was_empty {
                                compiler.fix_x_offsets(align);
                                compiler.newline(wrap_indent);
                            }
                            if !pp1.is_empty() {
                                compiler.push(pp1);
                                compiler.fix_x_offsets(align);
                                if line_was_empty {
                                    compiler.newline(wrap_indent);
                                }
                            }
                            next_piece = opt_pp2.or_else(|| pieces.next());
                        }
                    }
                }
                compiler.fix_x_offsets(align);
            }
            compiler.newline(0.0);
        }
        let size = compiler.actual_size();
        let total_millis = compiler.total_millis();
        let lines = compiler.into_lines();
        (size, total_millis, lines)
    }
}

//===========================================================================//

struct ParserLine {
    left: Vec<Box<dyn ParserPiece>>,
    center: Vec<Box<dyn ParserPiece>>,
    right: Vec<Box<dyn ParserPiece>>,
    wrap_indent: f32,
}

impl ParserLine {
    fn new() -> ParserLine {
        ParserLine {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
            wrap_indent: 0.0,
        }
    }

    fn is_empty(&self) -> bool {
        self.left.is_empty() && self.center.is_empty() && self.right.is_empty()
    }

    fn columns(self) -> Vec<(ParserAlign, Vec<Box<dyn ParserPiece>>)> {
        vec![
            (ParserAlign::Left, self.left),
            (ParserAlign::Center, self.center),
            (ParserAlign::Right, self.right),
        ]
    }
}

//===========================================================================//
