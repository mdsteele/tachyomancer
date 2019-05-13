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

use cgmath::Matrix4;
use std::mem;
use std::str::{Chars, FromStr};
use tachy::font::{Align, Font, Fonts};
use tachy::geom::Color4;
use tachy::gui::Resources;
use tachy::save::{Hotkey, Prefs};
use unicode_width::UnicodeWidthStr;

//===========================================================================//

const DEFAULT_ALIGN: ParserAlign = ParserAlign::Left;
const DEFAULT_COLOR: Color4 = Color4::WHITE;
const DEFAULT_FONT: Font = Font::Roman;
const GREEN: Color4 = Color4::new(0.0, 1.0, 0.0, 1.0);
const RED: Color4 = Color4::new(1.0, 0.0, 0.0, 1.0);

//===========================================================================//

pub struct Paragraph {
    lines: Vec<CompiledLine>,
    font_size: f32,
    line_height: f32,
    width: f32,
}

impl Paragraph {
    /// Compiles a paragraph of text, using format escapes to set text
    /// alignment, style, color, and font.  The following format escapes are
    /// supported:
    ///
    /// * `$$` inserts a literal `$` character.
    /// * `$<` aligns text to the left (the default).
    /// * `$=` aligns text to the center.
    /// * `$>` aligns text to the right.
    /// * `$!` sets the wrap indent position for the current line.
    /// * `$*` toggles bold text (default off).
    /// * `$/` toggles italic text (default off).
    /// * `$C` switches the text color to cyan.
    /// * `$D` switches the text color back to the default (white).
    /// * `$G` switches the text color to green.
    /// * `$K` switches the text color to black.
    /// * `$O` switches the text color to orange.
    /// * `$P` switches the text color to purple.
    /// * `$R` switches the text color to red.
    /// * `$W` switches the text color to white.
    /// * `$Y` switches the text color to yellow.
    /// * `${f}`, where `f` is the name of a font (e.g. `Alien`), switches to
    ///   that font for subsequent text.  The default font is `Roman`.
    /// * `$[h]`, where `h` is the name of a hotkey (e.g. `FlipHorz`), inserts
    ///   the name of the keycode bound to that hotkey.
    /// * `$(p)`, where `p` is the name of a special phrase, inserts the phrase
    ///   text.  Supported phrases include:
    ///     * "Command", which turns into the equivalent modifier key name
    ///       depending on the platform (e.g. "Command" on MacOS, "Control" on
    ///       Linux or Windows).
    ///     * "Right-click", which turns into the equivalent action depending
    ///       on the platform (e.g. "Control-click" on MacOS).
    pub fn compile(font_size: f32, line_height: f32, max_width: f32,
                   prefs: &Prefs, format: &str)
                   -> Paragraph {
        debug_assert!(font_size > 0.0);
        debug_assert!(max_width >= 0.0);
        let mut parser = Parser::new();
        let mut chars = format.chars();
        while let Some(chr) = chars.next() {
            if chr == '$' {
                match chars.next() {
                    Some('$') => parser.push('$'),
                    Some('<') => parser.set_align(ParserAlign::Left),
                    Some('=') => parser.set_align(ParserAlign::Center),
                    Some('>') => parser.set_align(ParserAlign::Right),
                    Some('!') => parser.set_wrap_indent_to_here(font_size),
                    Some('*') => parser.toggle_bold(),
                    Some('/') => parser.toggle_italic(),
                    Some('C') => parser.set_color(Color4::CYAN3),
                    Some('D') => parser.set_color(DEFAULT_COLOR),
                    Some('G') => parser.set_color(GREEN),
                    Some('K') => parser.set_color(Color4::BLACK),
                    Some('O') => parser.set_color(Color4::ORANGE3),
                    Some('P') => parser.set_color(Color4::PURPLE3),
                    Some('R') => parser.set_color(RED),
                    Some('W') => parser.set_color(Color4::WHITE),
                    Some('Y') => parser.set_color(Color4::YELLOW3),
                    Some('{') => parser.set_font(&parse_arg(&mut chars, '}')),
                    Some('[') => {
                        parser.push_key(&parse_arg(&mut chars, ']'), prefs)
                    }
                    Some('(') => {
                        parser.push_phrase(&parse_arg(&mut chars, ')'), prefs)
                    }
                    _ => {}
                }
            } else if chr == '\n' {
                parser.newline();
            } else {
                parser.push(chr);
            }
        }
        parser.compile(font_size, line_height, max_width)
    }

    pub fn width(&self) -> f32 { self.width }

    pub fn height(&self) -> f32 {
        (((self.lines.len() as f32) - 1.0) * self.line_height + self.font_size)
            .max(0.0)
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                topleft: (f32, f32)) {
        let fonts = resources.fonts();
        let (left, mut top) = topleft;
        for line in self.lines.iter() {
            line.draw(fonts, matrix, self.font_size, left, top);
            top += self.line_height;
        }
    }
}

fn parse_arg(chars: &mut Chars, close: char) -> String {
    let mut result = String::new();
    while let Some(chr) = chars.next() {
        if chr == close {
            break;
        } else {
            result.push(chr);
        }
    }
    result
}

//===========================================================================//

struct CompiledLine {
    pieces: Vec<CompiledPiece>,
}

impl CompiledLine {
    fn new(pieces: Vec<CompiledPiece>) -> CompiledLine {
        CompiledLine { pieces }
    }

    fn draw(&self, fonts: &Fonts, matrix: &Matrix4<f32>, font_size: f32,
            left: f32, top: f32) {
        for piece in self.pieces.iter() {
            piece.draw(fonts, matrix, font_size, left, top);
        }
    }
}

//===========================================================================//

struct CompiledPiece {
    offset: f32,
    font: Font,
    color: Color4,
    slant: f32,
    text: String,
}

impl CompiledPiece {
    fn draw(&self, fonts: &Fonts, matrix: &Matrix4<f32>, font_size: f32,
            left: f32, top: f32) {
        fonts.get(self.font).draw_style(matrix,
                                        font_size,
                                        Align::TopLeft,
                                        (left + self.offset, top),
                                        &self.color,
                                        self.slant,
                                        &self.text);
    }
}

//===========================================================================//

struct Parser {
    current_align: ParserAlign,
    current_color: Color4,
    current_font: Font,
    current_italic: bool,
    current_line: ParserLine,
    current_piece: String,
    lines: Vec<ParserLine>,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            current_align: DEFAULT_ALIGN,
            current_color: DEFAULT_COLOR,
            current_font: DEFAULT_FONT,
            current_italic: false,
            current_line: ParserLine::new(),
            current_piece: String::new(),
            lines: Vec::new(),
        }
    }

    fn push(&mut self, chr: char) { self.current_piece.push(chr); }

    fn push_phrase(&mut self, phrase_name: &str, prefs: &Prefs) {
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
                debug_log!("WARNING: Bad phrase name {:?} in paragraph format \
                            string",
                           phrase_name);
                phrase_name
            }
        };
        self.current_piece.push_str(phrase);
    }

    fn push_key(&mut self, hotkey_name: &str, prefs: &Prefs) {
        if let Ok(hotkey) = Hotkey::from_str(hotkey_name) {
            let key_name = Hotkey::keycode_name(prefs.hotkey_code(hotkey));
            self.current_piece.push_str(&format!("[{}]", key_name));
        } else {
            debug_log!("WARNING: Bad hotkey name {:?} in paragraph format \
                        string",
                       hotkey_name);
        }
    }

    fn newline(&mut self) {
        self.shift_piece();
        let line = mem::replace(&mut self.current_line, ParserLine::new());
        self.lines.push(line);
    }

    fn set_align(&mut self, align: ParserAlign) {
        if align != self.current_align {
            self.shift_piece();
            self.current_align = align;
        }
    }

    fn set_color(&mut self, color: Color4) {
        if color != self.current_color {
            self.shift_piece();
            self.current_color = color.into();
        }
    }

    fn set_font(&mut self, font_name: &str) {
        if let Ok(font) = Font::from_str(font_name) {
            if font != self.current_font {
                self.shift_piece();
                self.current_font = font;
            }
        } else {
            debug_log!("WARNING: Bad font name {:?} in paragraph format \
                        string",
                       font_name);
        }
    }

    fn set_wrap_indent_to_here(&mut self, font_size: f32) {
        self.shift_piece();
        let pieces = match self.current_align {
            ParserAlign::Left => &self.current_line.left,
            ParserAlign::Center => &self.current_line.center,
            ParserAlign::Right => &self.current_line.right,
        };
        self.current_line.wrap_indent =
            pieces.iter().map(|piece| piece.width(font_size)).sum();
    }

    fn toggle_bold(&mut self) {
        let next_font = match self.current_font {
            Font::Bold => Font::Roman,
            Font::Roman => Font::Bold,
            _ => return,
        };
        self.shift_piece();
        self.current_font = next_font;
    }

    fn toggle_italic(&mut self) {
        self.shift_piece();
        self.current_italic = !self.current_italic;
    }

    fn shift_piece(&mut self) {
        if !self.current_piece.is_empty() {
            let piece = ParserPiece {
                font: self.current_font,
                color: self.current_color,
                slant: if self.current_italic { 0.5 } else { 0.0 },
                text: mem::replace(&mut self.current_piece, String::new()),
            };
            let pieces = match self.current_align {
                ParserAlign::Left => &mut self.current_line.left,
                ParserAlign::Center => &mut self.current_line.center,
                ParserAlign::Right => &mut self.current_line.right,
            };
            pieces.push(piece);
        }
    }

    fn compile(mut self, font_size: f32, line_height: f32, max_width: f32)
               -> Paragraph {
        debug_assert!(font_size > 0.0);
        debug_assert!(max_width >= 0.0);
        self.shift_piece();
        if !self.current_line.is_empty() {
            self.newline();
        }
        let mut compiler = Compiler::new(font_size, line_height, max_width);
        for line in self.lines {
            let wrap_indent = line.wrap_indent;
            for (align, pieces) in line.columns() {
                let mut pieces = pieces.into_iter();
                let mut next_piece = pieces.next();
                while let Some(piece) = next_piece.take() {
                    let remaining_width = max_width - compiler.offset;
                    match piece.split(font_size, remaining_width) {
                        ParserPieceSplit::AllFits(pp) => {
                            compiler.push(pp);
                            next_piece = pieces.next();
                        }
                        ParserPieceSplit::SomeFits(pp1, pp2) => {
                            if !pp1.text.is_empty() {
                                compiler.push(pp1);
                            }
                            compiler.fix_offsets(align);
                            compiler.newline(wrap_indent);
                            next_piece = Some(pp2);
                        }
                        ParserPieceSplit::NoneFits(pp1, opt_pp2) => {
                            let line_was_empty = compiler.pieces.is_empty();
                            if !line_was_empty {
                                compiler.fix_offsets(align);
                                compiler.newline(wrap_indent);
                            }
                            if !pp1.text.is_empty() {
                                compiler.push(pp1);
                                compiler.fix_offsets(align);
                                if line_was_empty {
                                    compiler.newline(wrap_indent);
                                }
                            }
                            next_piece = opt_pp2.or_else(|| pieces.next());
                        }
                    }
                }
                compiler.fix_offsets(align);
            }
            compiler.newline(0.0);
        }
        compiler.finish()
    }
}

//===========================================================================//

struct Compiler {
    font_size: f32,
    line_height: f32,
    max_width: f32,
    actual_width: f32,
    lines: Vec<CompiledLine>,
    pieces: Vec<CompiledPiece>,
    align_start: usize,
    offset: f32,
}

impl Compiler {
    fn new(font_size: f32, line_height: f32, max_width: f32) -> Compiler {
        Compiler {
            font_size,
            line_height,
            max_width,
            actual_width: 0.0,
            lines: Vec::new(),
            pieces: Vec::new(),
            align_start: 0,
            offset: 0.0,
        }
    }

    fn push(&mut self, piece: ParserPiece) {
        let piece_width = piece.width(self.font_size);
        self.pieces.push(piece.compile(self.offset));
        self.offset += piece_width;
    }

    fn newline(&mut self, indent: f32) {
        let pieces = mem::replace(&mut self.pieces, Vec::new());
        self.lines.push(CompiledLine::new(pieces));
        self.actual_width = self.actual_width.max(self.offset);
        self.align_start = 0;
        self.offset = indent;
    }

    fn fix_offsets(&mut self, align: ParserAlign) {
        let align_start = self.align_start;
        self.align_start = self.pieces.len();
        if align_start >= self.pieces.len() {
            return;
        }
        let delta = match align {
            ParserAlign::Left => return,
            ParserAlign::Center => 0.5 * (self.max_width - self.offset),
            ParserAlign::Right => self.max_width - self.offset,
        };
        for piece in self.pieces[align_start..].iter_mut() {
            piece.offset += delta;
        }
        self.offset += delta;
    }

    fn finish(mut self) -> Paragraph {
        while let Some(line) = self.lines.pop() {
            if !line.pieces.is_empty() {
                self.lines.push(line);
                break;
            }
        }
        Paragraph {
            lines: self.lines,
            font_size: self.font_size,
            line_height: self.line_height,
            width: self.actual_width,
        }
    }
}

//===========================================================================//

struct ParserLine {
    left: Vec<ParserPiece>,
    center: Vec<ParserPiece>,
    right: Vec<ParserPiece>,
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

    fn columns(self) -> Vec<(ParserAlign, Vec<ParserPiece>)> {
        vec![
            (ParserAlign::Left, self.left),
            (ParserAlign::Center, self.center),
            (ParserAlign::Right, self.right),
        ]
    }
}

//===========================================================================//

struct ParserPiece {
    font: Font,
    color: Color4,
    slant: f32,
    text: String,
}

impl ParserPiece {
    fn width(&self, font_size: f32) -> f32 {
        font_size * self.font.ratio() * (self.text.width() as f32)
    }

    fn split(mut self, font_size: f32, remaining_width: f32)
             -> ParserPieceSplit {
        debug_assert!(font_size > 0.0);
        let remaining_chars: usize = if remaining_width <= 0.0 {
            0
        } else {
            (remaining_width / (font_size * self.font.ratio())).floor() as
                usize
        };

        if self.text.width() <= remaining_chars {
            return ParserPieceSplit::AllFits(self);
        }

        let mut str_index: usize = 0;
        let mut last_space: Option<usize> = None;
        for chr in self.text.chars().take(remaining_chars) {
            if chr == ' ' {
                last_space = Some(str_index);
            }
            str_index += chr.len_utf8();
        }
        if let Some(index) = last_space {
            let rest = ParserPiece {
                font: self.font,
                color: self.color,
                slant: self.slant,
                text: self.text[index..].trim_start_matches(' ').to_string(),
            };
            self.text.truncate(index);
            return ParserPieceSplit::SomeFits(self, rest);
        }

        let mut first_space: Option<usize> = None;
        for chr in self.text[str_index..].chars() {
            if chr == ' ' {
                first_space = Some(str_index);
                break;
            }
            str_index += chr.len_utf8();
        }
        if let Some(index) = first_space {
            let rest = ParserPiece {
                font: self.font,
                color: self.color,
                slant: self.slant,
                text: self.text[index..].trim_start_matches(' ').to_string(),
            };
            self.text.truncate(index);
            return ParserPieceSplit::NoneFits(self, Some(rest));
        }
        return ParserPieceSplit::NoneFits(self, None);
    }

    fn compile(self, offset: f32) -> CompiledPiece {
        CompiledPiece {
            offset,
            font: self.font,
            color: self.color,
            slant: self.slant,
            text: self.text,
        }
    }
}

//===========================================================================//

enum ParserPieceSplit {
    AllFits(ParserPiece),
    SomeFits(ParserPiece, ParserPiece),
    NoneFits(ParserPiece, Option<ParserPiece>),
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParserAlign {
    Left,
    Center,
    Right,
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::Paragraph;
    use tachy::font::Font;
    use tachy::save::Prefs;

    fn get_lines(paragraph: &Paragraph) -> Vec<String> {
        paragraph
            .lines
            .iter()
            .map(|line| {
                     let mut text = String::new();
                     for piece in line.pieces.iter() {
                         text.push_str(&piece.text);
                     }
                     text.trim_end_matches(' ').to_string()
                 })
            .collect()
    }

    #[test]
    fn simple_text_wrap() {
        let size = 12.0;
        let width = (18.0 * size * Font::Alien.ratio()).ceil();
        let prefs = Prefs::for_testing();
        let format = "${Alien}Lorem ipsum dolor sit amet, consectetur \
                      adipiscing elit.";
        let paragraph = Paragraph::compile(size, size, width, &prefs, format);
        assert_eq!(
            get_lines(&paragraph),
            vec![
                "Lorem ipsum dolor",
                "sit amet,",
                "consectetur",
                "adipiscing elit.",
            ]
        );
    }

    #[test]
    fn wrap_word_too_big_for_line() {
        let size = 12.0;
        let width = (6.0 * size * Font::Roman.ratio()).ceil();
        let prefs = Prefs::for_testing();
        let format = "a b c d ThisWordIsTooLong e f";
        let paragraph = Paragraph::compile(size, size, width, &prefs, format);
        assert_eq!(get_lines(&paragraph),
                   vec!["a b c", "d", "ThisWordIsTooLong", "e f"]);
    }

    #[test]
    fn explicit_linebreaks() {
        let size = 12.0;
        let width = (18.0 * size * Font::Roman.ratio()).ceil();
        let prefs = Prefs::for_testing();
        let format = "Lorem ipsum dolor sit amet.\n\n\
                      Consectetur adipiscing\n\
                      elit.\n\n";
        let paragraph = Paragraph::compile(size, size, width, &prefs, format);
        assert_eq!(
            get_lines(&paragraph),
            vec![
                "Lorem ipsum dolor",
                "sit amet.",
                "",
                "Consectetur",
                "adipiscing",
                "elit.",
            ]
        );
    }

    #[test]
    fn multiple_styles() {
        let size = 20.0;
        let width = (38.0 * size * Font::Roman.ratio()).ceil();
        let prefs = Prefs::for_testing();
        let format = "$CEvent$D wire loops can be broken with \
                      a $*Clock$* or $*Delay$* chip.";
        let paragraph = Paragraph::compile(size, size, width, &prefs, format);
        assert_eq!(
            get_lines(&paragraph),
            vec![
                "Event wire loops can be broken with a",
                "Clock or Delay chip.",
            ]
        );
    }
}

//===========================================================================//
