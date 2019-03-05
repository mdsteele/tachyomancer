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

use super::puzzle::{Puzzle, PuzzleKind};
use std::str;
use tachy::geom::CoordsSize;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChipType {
    // Values:
    Const(u32),
    Pack,
    Unpack,
    // Arithmetic:
    Add,
    Sub,
    // Comparison:
    Cmp,
    CmpEq,
    Eq,
    // Logic:
    Not,
    And,
    Mux,
    // Events:
    Clock,
    Delay,
    Discard,
    Join,
    Latest,
    Sample,
    // Special:
    Break,
    Ram,
    Display,
    Button,
}

impl str::FromStr for ChipType {
    type Err = String;

    fn from_str(string: &str) -> Result<ChipType, String> {
        match string {
            "Add" => Ok(ChipType::Add),
            "And" => Ok(ChipType::And),
            "Break" => Ok(ChipType::Break),
            "Button" => Ok(ChipType::Button),
            "Clock" => Ok(ChipType::Clock),
            "Cmp" => Ok(ChipType::Cmp),
            "CmpEq" => Ok(ChipType::CmpEq),
            "Delay" => Ok(ChipType::Delay),
            "Discard" => Ok(ChipType::Discard),
            "Display" => Ok(ChipType::Display),
            "Eq" => Ok(ChipType::Eq),
            "Join" => Ok(ChipType::Join),
            "Latest" => Ok(ChipType::Latest),
            "Mux" => Ok(ChipType::Mux),
            "Not" => Ok(ChipType::Not),
            "Pack" => Ok(ChipType::Pack),
            "Ram" => Ok(ChipType::Ram),
            "Sample" => Ok(ChipType::Sample),
            "Sub" => Ok(ChipType::Sub),
            "Unpack" => Ok(ChipType::Unpack),
            _ => {
                if string.starts_with("Const(") && string.ends_with(')') {
                    let inner = &string[6..(string.len() - 1)];
                    if let Ok(value) = inner.parse() {
                        return Ok(ChipType::Const(value));
                    }
                }
                Err(string.to_string())
            }
        }
    }
}

impl ChipType {
    pub fn is_allowed_in(self, puzzle: Puzzle) -> bool {
        match self {
            ChipType::Clock | ChipType::Delay | ChipType::Discard |
            ChipType::Join | ChipType::Latest | ChipType::Sample |
            ChipType::Break | ChipType::Ram => puzzle.allows_events(),
            ChipType::Button => {
                match puzzle.kind() {
                    PuzzleKind::Command | PuzzleKind::Sandbox => {
                        puzzle.allows_events()
                    }
                    _ => false,
                }
            }
            _ => true,
        }
    }

    /// Returns the width and height of the chip in its default orientation.
    pub fn size(self) -> CoordsSize {
        match self {
            ChipType::Ram => CoordsSize::new(2, 2),
            ChipType::Display => CoordsSize::new(2, 1),
            _ => CoordsSize::new(1, 1),
        }
    }

    pub fn tooltip_format(self) -> String {
        let name = match self {
            ChipType::Const(_) => "Const".to_string(),
            other => format!("{:?}", other),
        };
        let description = match self {
            ChipType::Const(_) => {
                "Outputs a constant value.\n\
                 $(Right-click) on the chip to change the output value."
            }
            ChipType::Discard => {
                "Transforms value-carrying events into 0-bit events by \
                 discarding the value."
            }
            _ => "TODO",
        };
        format!("$*{}$*\n{}", name, description)
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::ChipType;

    #[test]
    fn chip_type_to_and_from_string() {
        let chip_types = &[
            ChipType::Add,
            ChipType::And,
            ChipType::Break,
            ChipType::Button,
            ChipType::Clock,
            ChipType::Cmp,
            ChipType::CmpEq,
            ChipType::Const(0),
            ChipType::Const(13),
            ChipType::Const(0xffffffff),
            ChipType::Delay,
            ChipType::Discard,
            ChipType::Display,
            ChipType::Eq,
            ChipType::Join,
            ChipType::Latest,
            ChipType::Mux,
            ChipType::Not,
            ChipType::Pack,
            ChipType::Ram,
            ChipType::Sample,
            ChipType::Sub,
            ChipType::Unpack,
        ];
        for &ctype in chip_types.iter() {
            assert_eq!(format!("{:?}", ctype).parse(), Ok(ctype));
        }
    }
}

//===========================================================================//
