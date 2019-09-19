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

use super::types::{CompiledPiece, ParserPiece, ParserPieceSplit};
use crate::tachy::font::{Align, Font};
use crate::tachy::geom::Color4;
use crate::tachy::gui::Resources;
use cgmath::Matrix4;
use std::char;
use std::mem;

//===========================================================================//

pub struct ParserTextPiece {
    font: Font,
    color: Color4,
    millis_per_char: usize,
    slant: f32,
    chars: Vec<u8>,
}

impl ParserTextPiece {
    pub fn new(
        font: Font,
        color: Color4,
        slant: f32,
        millis_per_char: usize,
        chars: Vec<u8>,
    ) -> ParserTextPiece {
        ParserTextPiece { font, color, slant, millis_per_char, chars }
    }
}

impl ParserPiece for ParserTextPiece {
    fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    fn width(&self, font_size: f32) -> f32 {
        font_size * self.font.ratio() * (self.chars.len() as f32)
    }

    fn height(&self, font_size: f32) -> f32 {
        font_size
    }

    fn num_millis(&self) -> usize {
        self.millis_per_char * self.chars.len()
    }

    fn split(
        &mut self,
        font_size: f32,
        remaining_width: f32,
    ) -> ParserPieceSplit {
        debug_assert!(font_size > 0.0);
        let remaining_chars: usize = if remaining_width <= 0.0 {
            0
        } else {
            (remaining_width / (font_size * self.font.ratio())).floor()
                as usize
        };

        if self.chars.len() <= remaining_chars {
            return ParserPieceSplit::AllFits;
        }

        let mut chars_index: usize = 0;
        let mut last_space: Option<usize> = None;
        for &chr in &self.chars[..remaining_chars] {
            if chr == b' ' {
                last_space = Some(chars_index);
            }
            chars_index += 1;
        }
        if let Some(index) = last_space {
            let rest = ParserTextPiece {
                font: self.font,
                color: self.color,
                millis_per_char: self.millis_per_char,
                slant: self.slant,
                chars: trim_starting_spaces(&self.chars[index..]),
            };
            self.chars.truncate(index);
            return ParserPieceSplit::SomeFits(Box::new(rest));
        }

        let mut first_space: Option<usize> = None;
        for &chr in &self.chars[chars_index..] {
            if chr == b' ' {
                first_space = Some(chars_index);
                break;
            }
            chars_index += 1;
        }
        if let Some(index) = first_space {
            let rest = ParserTextPiece {
                font: self.font,
                color: self.color,
                millis_per_char: self.millis_per_char,
                slant: self.slant,
                chars: trim_starting_spaces(&self.chars[index..]),
            };
            self.chars.truncate(index);
            return ParserPieceSplit::NoneFits(Some(Box::new(rest)));
        }
        return ParserPieceSplit::NoneFits(None);
    }

    fn compile(
        &mut self,
        x_offset: f32,
        y_offset: f32,
    ) -> Box<dyn CompiledPiece> {
        let piece = CompiledTextPiece {
            offset: (x_offset, y_offset),
            font: self.font,
            color: self.color,
            millis_per_char: self.millis_per_char,
            slant: self.slant,
            chars: mem::replace(&mut self.chars, Vec::new()),
        };
        Box::new(piece)
    }
}

fn trim_starting_spaces(chars: &[u8]) -> Vec<u8> {
    for (index, &chr) in chars.iter().enumerate() {
        if chr != b' ' {
            return chars[index..].to_vec();
        }
    }
    return chars.to_vec();
}

//===========================================================================//

struct CompiledTextPiece {
    offset: (f32, f32),
    font: Font,
    color: Color4,
    millis_per_char: usize,
    slant: f32,
    chars: Vec<u8>,
}

impl CompiledPiece for CompiledTextPiece {
    fn height(&self, font_size: f32) -> f32 {
        font_size
    }

    fn add_x_offset(&mut self, x_offset: f32) {
        self.offset.0 += x_offset;
    }

    fn add_y_offset(&mut self, y_offset: f32) {
        self.offset.1 += y_offset;
    }

    fn draw(
        &self,
        resources: &Resources,
        paragraph_matrix: &Matrix4<f32>,
        font_size: f32,
        millis_remaining: &mut usize,
    ) -> bool {
        let text_millis = self.chars.len() * self.millis_per_char;
        let (chars, finished) = if text_millis <= *millis_remaining {
            *millis_remaining -= text_millis;
            (self.chars.as_slice(), true)
        } else {
            debug_assert!(self.millis_per_char > 0);
            let substring_chars = *millis_remaining / self.millis_per_char;
            debug_assert!(substring_chars < self.chars.len());
            (&self.chars[..substring_chars], false)
        };
        let font = resources.fonts().get(self.font);
        font.draw_chars(
            paragraph_matrix,
            font_size,
            Align::TopLeft,
            self.offset,
            &self.color,
            self.slant,
            chars,
        );
        finished
    }

    fn debug_string(&self) -> String {
        self.chars
            .iter()
            .map(|&chr| char::from_u32(chr as u32).unwrap_or('?'))
            .collect()
    }
}

//===========================================================================//
