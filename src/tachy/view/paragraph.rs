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
use tachy::gui::Resources;
use tachy::save::{Hotkey, Prefs};
use unicode_width::UnicodeWidthStr;

//===========================================================================//

pub struct Paragraph {
    lines: Vec<CompiledLine>,
    font_size: f32,
    line_height: f32,
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
    /// * `$*` toggles bold text (default off).
    /// * `$/` toggles italic text (default off).
    /// * `$B` switches the text color to blue.
    /// * `$C` switches the text color to cyan.
    /// * `$G` switches the text color to green.
    /// * `$K` switches the text color to black (the default).
    /// * `$M` switches the text color to magenta.
    /// * `$O` switches the text color to orange.
    /// * `$P` switches the text color to purple.
    /// * `$R` switches the text color to red.
    /// * `$W` switches the text color to white.
    /// * `$Y` switches the text color to yellow.
    /// * `${f}`, where `f` is the name of a font (e.g. `Alien`), switches to
    ///   that font for subsequent text.  The default font is `Roman`.
    /// * `$[h]`, where `h` is the name of a hotkey (e.g. `FlipHorz`), inserts
    ///   the name of the keycode bound to that hotkey.
    pub fn compile(font_size: f32, line_height: f32, width: f32,
                   prefs: &Prefs, format: &str)
                   -> Paragraph {
        debug_assert!(font_size > 0.0);
        debug_assert!(width >= 0.0);
        let mut parser = Parser::new();
        let mut chars = format.chars();
        while let Some(chr) = chars.next() {
            if chr == '$' {
                match chars.next() {
                    Some('$') => parser.push('$'),
                    Some('<') => parser.set_align(ParserAlign::Left),
                    Some('=') => parser.set_align(ParserAlign::Center),
                    Some('>') => parser.set_align(ParserAlign::Right),
                    Some('*') => parser.toggle_bold(),
                    Some('/') => parser.toggle_italic(),
                    Some('B') => parser.set_color((0.0, 0.0, 1.0)),
                    Some('C') => parser.set_color((0.0, 1.0, 1.0)),
                    Some('G') => parser.set_color((0.0, 1.0, 0.0)),
                    Some('K') => parser.set_color((0.0, 0.0, 0.0)),
                    Some('M') => parser.set_color((1.0, 0.0, 1.0)),
                    Some('O') => parser.set_color((1.0, 0.5, 0.0)),
                    Some('P') => parser.set_color((0.5, 0.0, 1.0)),
                    Some('R') => parser.set_color((1.0, 0.0, 0.0)),
                    Some('W') => parser.set_color((1.0, 1.0, 1.0)),
                    Some('Y') => parser.set_color((1.0, 1.0, 0.0)),
                    Some('{') => parser.set_font(&parse_arg(&mut chars, '}')),
                    Some('[') => {
                        parser.push_key(&parse_arg(&mut chars, ']'), prefs)
                    }
                    _ => {}
                }
            } else if chr == '\n' {
                parser.newline();
            } else {
                parser.push(chr);
            }
        }
        parser.compile(font_size, line_height, width)
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                top_left: (f32, f32)) {
        let fonts = resources.fonts();
        let (left, mut top) = top_left;
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
    color: (f32, f32, f32),
    text: String,
}

impl CompiledPiece {
    fn draw(&self, fonts: &Fonts, matrix: &Matrix4<f32>, font_size: f32,
            left: f32, top: f32) {
        fonts.get(self.font).draw_color(matrix,
                                        font_size,
                                        Align::TopLeft,
                                        (left + self.offset, top),
                                        self.color,
                                        &self.text);
    }
}

//===========================================================================//

struct Parser {
    current_align: ParserAlign,
    current_bold: bool,
    current_color: (f32, f32, f32),
    current_font: Font,
    current_italic: bool,
    current_line: ParserLine,
    current_piece: String,
    lines: Vec<ParserLine>,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            current_align: ParserAlign::Left,
            current_bold: false,
            current_color: (0.0, 0.0, 0.0),
            current_font: Font::Roman,
            current_italic: false,
            current_line: ParserLine::new(),
            current_piece: String::new(),
            lines: Vec::new(),
        }
    }

    fn push(&mut self, chr: char) { self.current_piece.push(chr); }

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

    fn set_color(&mut self, color: (f32, f32, f32)) {
        if color != self.current_color {
            self.shift_piece();
            self.current_color = color;
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

    fn toggle_bold(&mut self) { self.current_bold = !self.current_bold; }

    fn toggle_italic(&mut self) { self.current_italic = !self.current_italic; }

    fn shift_piece(&mut self) {
        if !self.current_piece.is_empty() {
            // TODO: Use current_bold and current_italic
            let piece = ParserPiece {
                font: self.current_font,
                color: self.current_color,
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

    fn compile(mut self, font_size: f32, line_height: f32, width: f32)
               -> Paragraph {
        debug_assert!(font_size > 0.0);
        debug_assert!(width >= 0.0);
        self.shift_piece();
        if !self.current_line.is_empty() {
            self.newline();
        }
        let mut compiler = Compiler::new(font_size, line_height, width);
        for line in self.lines {
            for (align, pieces) in line.columns() {
                let mut pieces = pieces.into_iter();
                let mut next_piece = pieces.next();
                while let Some(piece) = next_piece.take() {
                    let remaining_width = width - compiler.offset;
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
                            compiler.newline();
                            next_piece = Some(pp2);
                        }
                        ParserPieceSplit::NoneFits(pp1, opt_pp2) => {
                            if !compiler.pieces.is_empty() {
                                compiler.fix_offsets(align);
                                compiler.newline();
                            }
                            if !pp1.text.is_empty() {
                                compiler.push(pp1);
                                compiler.fix_offsets(align);
                                compiler.newline();
                            }
                            next_piece = opt_pp2.or_else(|| pieces.next());
                        }
                    }
                }
                compiler.fix_offsets(align);
                compiler.align_start = compiler.pieces.len();
            }
            compiler.newline();
        }
        compiler.finish()
    }
}

//===========================================================================//

struct Compiler {
    font_size: f32,
    line_height: f32,
    width: f32,
    lines: Vec<CompiledLine>,
    pieces: Vec<CompiledPiece>,
    align_start: usize,
    offset: f32,
}

impl Compiler {
    fn new(font_size: f32, line_height: f32, width: f32) -> Compiler {
        Compiler {
            font_size,
            line_height,
            width,
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

    fn newline(&mut self) {
        let pieces = mem::replace(&mut self.pieces, Vec::new());
        self.lines.push(CompiledLine::new(pieces));
        self.align_start = 0;
        self.offset = 0.0;
    }

    fn fix_offsets(&mut self, align: ParserAlign) {
        if self.align_start >= self.pieces.len() {
            return;
        }
        let delta = match align {
            ParserAlign::Left => return,
            ParserAlign::Center => 0.5 * (self.width - self.offset),
            ParserAlign::Right => self.width - self.offset,
        };
        for piece in self.pieces[self.align_start..].iter_mut() {
            piece.offset += delta;
        }
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
        }
    }
}

//===========================================================================//

struct ParserLine {
    left: Vec<ParserPiece>,
    center: Vec<ParserPiece>,
    right: Vec<ParserPiece>,
}

impl ParserLine {
    fn new() -> ParserLine {
        ParserLine {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
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
    color: (f32, f32, f32),
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
                text: self.text[index..].trim_start().to_string(),
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
                text: self.text[index..].trim_start().to_string(),
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

#[derive(Clone, Copy, Eq, PartialEq)]
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
                     text
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
}

//===========================================================================//
