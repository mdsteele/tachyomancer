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

use crate::mancer::gui::Resources;
use cgmath::Matrix4;

//===========================================================================//

pub struct CompiledLine {
    pieces: Vec<Box<dyn CompiledPiece>>,
}

impl CompiledLine {
    pub fn new(pieces: Vec<Box<dyn CompiledPiece>>) -> CompiledLine {
        CompiledLine { pieces }
    }

    pub fn pieces(&self) -> &[Box<dyn CompiledPiece>] {
        &self.pieces
    }

    /// Draws part of the line, up to `num_millis` worth.  If the whole line is
    /// drawn, subtracts the millis consumed from `num_millis` and returns
    /// zero.  Otherwise, returns the number of additional millis needed to
    /// draw anything more.
    pub fn draw(
        &self,
        resources: &Resources,
        paragraph_matrix: &Matrix4<f32>,
        font_size: f32,
        num_millis: &mut usize,
    ) -> usize {
        for piece in self.pieces.iter() {
            let needed_for_next =
                piece.draw(resources, paragraph_matrix, font_size, num_millis);
            if needed_for_next > 0 {
                return needed_for_next;
            }
        }
        return 0;
    }
}

//===========================================================================//

pub trait CompiledPiece {
    fn height(&self, font_size: f32) -> f32;

    fn add_x_offset(&mut self, x_offset: f32);

    fn add_y_offset(&mut self, y_offset: f32);

    /// Draws part of the piece, up to `num_millis` worth.  If the whole piece
    /// is drawn, subtracts the millis consumed from `num_millis` and returns
    /// zero.  Otherwise, returns the number of additional millis needed to
    /// draw anything more.
    fn draw(
        &self,
        resources: &Resources,
        paragraph_matrix: &Matrix4<f32>,
        font_size: f32,
        millis_remaining: &mut usize,
    ) -> usize;

    fn debug_string(&self) -> String;
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParserAlign {
    Left,
    Center,
    Right,
}

//===========================================================================//

pub trait ParserPiece {
    fn is_empty(&self) -> bool;

    fn width(&self, font_size: f32) -> f32;

    fn height(&self, font_size: f32) -> f32;

    fn num_millis(&self) -> usize;

    fn split(
        &mut self,
        font_size: f32,
        remaining_width: f32,
    ) -> ParserPieceSplit;

    fn compile(
        &mut self,
        x_offset: f32,
        y_offset: f32,
    ) -> Box<dyn CompiledPiece>;
}

//===========================================================================//

pub enum ParserPieceSplit {
    AllFits,
    SomeFits(Box<dyn ParserPiece>),
    NoneFits(Option<Box<dyn ParserPiece>>),
}

//===========================================================================//
