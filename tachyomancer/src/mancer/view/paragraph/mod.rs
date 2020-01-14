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

mod circuit;
mod compile;
mod parse;
mod text;
mod types;

use self::parse::Parser;
use self::types::{CompiledLine, ParserAlign};
use crate::mancer::gui::{Resources, Ui};
use crate::mancer::save::Prefs;
use cgmath::Matrix4;
use std::cell::Cell;
use std::str::Chars;
use tachy::geom::{Color4, MatrixExt, RectSize};

//===========================================================================//

const GREEN: Color4 = Color4::new(0.0, 1.0, 0.0, 1.0);
const RED: Color4 = Color4::new(1.0, 0.0, 0.0, 1.0);

//===========================================================================//

pub struct Paragraph {
    lines: Vec<CompiledLine>,
    font_size: f32,
    total_millis: usize,
    size: RectSize<f32>,
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
    /// * `$(n)`, where `n` is a decimal number, switches the text speed to
    ///   that many milliseconds per character.
    /// * `$()` switches the text speed back to the default.
    /// * `$'p'`, where `p` is the name of a special phrase, inserts the phrase
    ///   text.  Supported phrases include:
    ///     * "Command", which turns into the equivalent modifier key name
    ///       depending on the platform (e.g. "Command" on MacOS, "Control" on
    ///       Linux or Windows).
    ///     * "Right-click", which turns into the equivalent action depending
    ///       on the platform (e.g. "Control-click" on MacOS).
    ///     * "YOURNAME", which turns into the name of the current profile.
    /// * `$|o-c|`, where `o` is an orientation and `c` is a chip type, inserts
    ///   an image of that chip.
    /// * `$#t#`, where `t` is TOML for a circuit, inserts an image of that
    ///   circuit.
    pub fn compile(
        font_size: f32,
        line_height: f32,
        max_width: f32,
        prefs: &Prefs,
        format: &str,
    ) -> Paragraph {
        debug_assert!(font_size > 0.0);
        debug_assert!(max_width >= 0.0);
        let line_gap = line_height - font_size;
        let mut parser = Parser::new();
        let mut chars = format.chars();
        while let Some(chr) = chars.next() {
            if chr == '$' {
                match chars.next() {
                    Some('$') => parser.push_char('$'),
                    Some('<') => parser.set_align(ParserAlign::Left),
                    Some('=') => parser.set_align(ParserAlign::Center),
                    Some('>') => parser.set_align(ParserAlign::Right),
                    Some('!') => parser.set_wrap_indent_to_here(font_size),
                    Some('*') => parser.toggle_bold(),
                    Some('/') => parser.toggle_italic(),
                    Some('C') => parser.set_color(Color4::CYAN3),
                    Some('D') => parser.set_color_to_default(),
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
                        parser.set_millis_per_char(&parse_arg(&mut chars, ')'))
                    }
                    Some('\'') => {
                        parser.push_phrase(&parse_arg(&mut chars, '\''), prefs)
                    }
                    Some('|') => {
                        let orient_str = parse_arg(&mut chars, '-');
                        let ctype_str = parse_arg(&mut chars, '|');
                        parser.push_chip(&orient_str, &ctype_str);
                    }
                    Some('#') => {
                        parser.push_circuit(&parse_arg(&mut chars, '#'))
                    }
                    Some(ch) => {
                        debug_warn!("Invalid paragraph escape: ${}", ch);
                    }
                    None => {
                        debug_warn!(
                            "Incomplete paragraph escape at end of \
                             format string"
                        );
                    }
                }
            } else if chr == '\n' {
                parser.newline();
            } else {
                parser.push_char(chr);
            }
        }
        let (size, total_millis, lines) =
            parser.compile(font_size, line_gap, max_width);
        Paragraph { lines, font_size, total_millis, size }
    }

    pub fn escape(string: &str) -> String {
        string.replace('$', "$$")
    }

    pub fn width(&self) -> f32 {
        self.size.width
    }

    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn total_millis(&self) -> usize {
        self.total_millis
    }

    /// Draws the whole paragraph.
    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        left_top: (f32, f32),
    ) {
        self.draw_partial(resources, matrix, left_top, self.total_millis);
    }

    /// Draws part of the paragraph, up to `num_millis` worth.  Returns the
    /// minimum number of additional millis needed to draw anything more (or
    /// zero if the whole paragraph was drawn).
    pub fn draw_partial(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        left_top: (f32, f32),
        mut num_millis: usize,
    ) -> usize {
        let (left, top) = left_top;
        let paragraph_matrix = matrix * Matrix4::trans2(left, top);
        for line in self.lines.iter() {
            let needed_for_next = line.draw(
                resources,
                &paragraph_matrix,
                self.font_size,
                &mut num_millis,
            );
            if needed_for_next > 0 {
                return needed_for_next;
            }
        }
        return 0;
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

pub struct StreamingParagraph {
    paragraph: Paragraph,
    millis: f64,
    millis_for_next: Cell<f64>,
}

impl StreamingParagraph {
    pub fn new(paragraph: Paragraph) -> StreamingParagraph {
        StreamingParagraph {
            paragraph,
            millis: 0.0,
            millis_for_next: Cell::new(0.0),
        }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        left_top: (f32, f32),
    ) {
        let millis = self.millis as usize;
        let needed_for_next =
            self.paragraph.draw_partial(resources, matrix, left_top, millis);
        self.millis_for_next.set((millis + needed_for_next) as f64);
    }

    pub fn tick(&mut self, elapsed: f64, ui: &mut Ui) {
        if self.millis < self.paragraph.total_millis() as f64 {
            self.millis += elapsed * 1000.0;
            if self.millis >= self.millis_for_next.get() {
                ui.request_redraw();
            }
        }
    }

    pub fn skip_to_end(&mut self, ui: &mut Ui) {
        let total_millis = self.paragraph.total_millis() as f64;
        if self.millis < total_millis {
            self.millis = total_millis;
            ui.request_redraw();
        }
    }

    pub fn is_done(&self) -> bool {
        (self.millis as usize) >= self.paragraph.total_millis()
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::Paragraph;
    use crate::mancer::font::Font;
    use crate::mancer::save::Prefs;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn get_lines(paragraph: &Paragraph) -> Vec<String> {
        paragraph.lines.iter().map(|line| {
            let mut text = String::new();
            for piece in line.pieces() {
                text.push_str(&piece.debug_string());
            }
            text.trim_end_matches(' ').to_string()
        }).collect()
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
        assert_eq!(
            get_lines(&paragraph),
            vec!["a b c", "d", "ThisWordIsTooLong", "e f"]
        );
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
