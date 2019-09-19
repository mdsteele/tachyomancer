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

use super::types::{CompiledLine, CompiledPiece, ParserAlign, ParserPiece};
use crate::tachy::geom::RectSize;
use std::mem;

//===========================================================================//

pub struct Compiler {
    font_size: f32,
    line_gap: f32,
    max_width: f32,
    actual_width: f32,
    actual_height: f32,
    lines: Vec<CompiledLine>,
    current_line_pieces: Vec<Box<dyn CompiledPiece>>,
    current_line_top: f32,
    current_line_height: f32,
    align_start: usize,
    x_offset: f32,
    total_millis: usize,
}

impl Compiler {
    pub fn new(font_size: f32, line_gap: f32, max_width: f32) -> Compiler {
        Compiler {
            font_size,
            line_gap,
            max_width,
            actual_width: 0.0,
            actual_height: 0.0,
            lines: Vec::new(),
            current_line_pieces: Vec::new(),
            current_line_top: 0.0,
            current_line_height: font_size,
            align_start: 0,
            x_offset: 0.0,
            total_millis: 0,
        }
    }

    pub fn offset(&self) -> f32 {
        self.x_offset
    }

    pub fn current_line_is_empty(&self) -> bool {
        self.current_line_pieces.is_empty()
    }

    pub fn push(&mut self, mut piece: Box<dyn ParserPiece>) {
        self.total_millis += piece.num_millis();
        let piece_width = piece.width(self.font_size);
        let piece_height = piece.height(self.font_size);
        self.current_line_pieces
            .push(piece.compile(self.x_offset, self.current_line_top));
        self.current_line_height = self.current_line_height.max(piece_height);
        self.x_offset += piece_width;
    }

    pub fn newline(&mut self, indent: f32) {
        for piece in self.current_line_pieces.iter_mut() {
            let piece_height = piece.height(self.font_size);
            let y_offset = 0.5 * (self.current_line_height - piece_height);
            piece.add_y_offset(y_offset);
        }
        let line_pieces =
            mem::replace(&mut self.current_line_pieces, Vec::new());
        self.lines.push(CompiledLine::new(line_pieces));
        self.actual_width = self.actual_width.max(self.x_offset);
        self.actual_height = self.current_line_top + self.current_line_height;
        self.current_line_top = self.actual_height + self.line_gap;
        self.current_line_height = self.font_size;
        self.align_start = 0;
        self.x_offset = indent;
    }

    pub fn fix_x_offsets(&mut self, align: ParserAlign) {
        let align_start = self.align_start;
        self.align_start = self.current_line_pieces.len();
        if align_start >= self.current_line_pieces.len() {
            return;
        }
        let delta = match align {
            ParserAlign::Left => return,
            ParserAlign::Center => 0.5 * (self.max_width - self.x_offset),
            ParserAlign::Right => self.max_width - self.x_offset,
        };
        for piece in self.current_line_pieces[align_start..].iter_mut() {
            piece.add_x_offset(delta);
        }
        self.x_offset += delta;
    }

    pub fn actual_size(&self) -> RectSize<f32> {
        RectSize::new(self.actual_width, self.actual_height)
    }

    pub fn total_millis(&self) -> usize {
        self.total_millis
    }

    pub fn into_lines(mut self) -> Vec<CompiledLine> {
        while let Some(line) = self.lines.pop() {
            if !line.pieces().is_empty() {
                self.lines.push(line);
                break;
            }
        }
        self.lines
    }
}

//===========================================================================//
