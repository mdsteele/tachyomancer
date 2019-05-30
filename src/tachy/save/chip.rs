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

use std::collections::HashSet;
use std::str;
use tachy::geom::CoordsSize;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChipType {
    // Value:
    Const(u32),
    Pack,
    Unpack,
    // Arithmetic:
    Add,
    Sub,
    Mul,
    // Comparison:
    Cmp,
    CmpEq,
    Eq,
    // Logic:
    Not,
    And,
    Or,
    Xor,
    Mux,
    // Events:
    Clock,
    Delay,
    Demux,
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

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const CHIP_CATEGORIES: &[(&str, &[ChipType])] = &[
    ("Value", &[
        ChipType::Const(1),
        ChipType::Pack,
        ChipType::Unpack,
    ]),
    ("Arithmetic", &[
        ChipType::Add,
        ChipType::Sub,
        ChipType::Mul,
    ]),
    ("Comparison", &[
        ChipType::Cmp,
        ChipType::CmpEq,
        ChipType::Eq,
    ]),
    ("Logic", &[
        ChipType::Not,
        ChipType::And,
        ChipType::Or,
        ChipType::Xor,
        ChipType::Mux,
    ]),
    ("Events", &[
        ChipType::Clock,
        ChipType::Delay,
        ChipType::Demux,
        ChipType::Discard,
        ChipType::Join,
        ChipType::Latest,
        ChipType::Sample,
    ]),
    ("Special", &[
        ChipType::Break,
        ChipType::Ram,
        ChipType::Display,
        ChipType::Button,
    ]),
];

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
            "Demux" => Ok(ChipType::Demux),
            "Discard" => Ok(ChipType::Discard),
            "Display" => Ok(ChipType::Display),
            "Eq" => Ok(ChipType::Eq),
            "Join" => Ok(ChipType::Join),
            "Latest" => Ok(ChipType::Latest),
            "Mul" => Ok(ChipType::Mul),
            "Mux" => Ok(ChipType::Mux),
            "Not" => Ok(ChipType::Not),
            "Or" => Ok(ChipType::Or),
            "Pack" => Ok(ChipType::Pack),
            "Ram" => Ok(ChipType::Ram),
            "Sample" => Ok(ChipType::Sample),
            "Sub" => Ok(ChipType::Sub),
            "Unpack" => Ok(ChipType::Unpack),
            "Xor" => Ok(ChipType::Xor),
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
    pub fn is_interactive(self) -> bool {
        match self {
            ChipType::Button => true,
            _ => false,
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
            ChipType::And => "Bitwise AND".to_string(),
            ChipType::Const(_) => "Constant".to_string(),
            ChipType::Not => "Bitwise NOT".to_string(),
            ChipType::Or => "Bitwise OR".to_string(),
            ChipType::Xor => "Bitwise XOR".to_string(),
            other => format!("{:?}", other),
        };
        let description = match self {
            ChipType::And => {
                "For each bit in the wire, the output is 1 if both inputs \
                 are 1, or 0 if either input is 0."
            }
            ChipType::Const(_) => {
                "Outputs a constant value.\n\
                 $'Right-click' on the chip to change the output value."
            }
            ChipType::Discard => {
                "Transforms value-carrying events into 0-bit events by \
                 discarding the value."
            }
            ChipType::Not => {
                "Inverts bits.  Each 0 bit in the input becomes a 1 bit in \
                 the output, and vice-versa."
            }
            ChipType::Or => {
                "For each bit in the wire, the output is 1 if either input \
                 is 1, or 0 if both inputs are 0."
            }
            ChipType::Pack => {
                "Joins two input wires into a single output wire with twice \
                 as many bits.  One input wire becomes the low bits of the \
                 output, and the other becomes the high bits."
            }
            ChipType::Unpack => {
                "Splits the input wire into two output wires, each with half \
                 as many bits.  One output wire has the low bits of the \
                 input, and the other has the high bits."
            }
            ChipType::Xor => {
                "For each bit in the wire, the output is 1 if exactly one \
                 input is 1, or 0 if the inputs are both 0 or both 1."
            }
            _ => "TODO",
        };
        format!("$*{}$*\n{}", name, description)
    }
}

//===========================================================================//

pub struct ChipSet {
    ctypes: HashSet<ChipType>,
}

impl ChipSet {
    pub fn new() -> ChipSet { ChipSet { ctypes: HashSet::new() } }

    pub fn contains(&self, ctype: ChipType) -> bool {
        match ctype {
            ChipType::Const(_) => self.ctypes.contains(&ChipType::Const(0)),
            _ => self.ctypes.contains(&ctype),
        }
    }

    pub fn insert(&mut self, ctype: ChipType) {
        match ctype {
            ChipType::Const(_) => {
                self.ctypes.insert(ChipType::Const(0));
            }
            _ => {
                self.ctypes.insert(ctype);
            }
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{CHIP_CATEGORIES, ChipSet, ChipType};

    #[test]
    fn chip_type_to_and_from_string() {
        let mut chip_types = vec![
            ChipType::Const(0),
            ChipType::Const(13),
            ChipType::Const(0xffffffff),
        ];
        for &(_, ctypes) in CHIP_CATEGORIES.iter() {
            chip_types.extend_from_slice(ctypes);
        }
        for &ctype in chip_types.iter() {
            assert_eq!(format!("{:?}", ctype).parse(), Ok(ctype));
        }
    }

    #[test]
    fn chip_set() {
        let mut set = ChipSet::new();
        assert!(!set.contains(ChipType::Const(1)));
        assert!(!set.contains(ChipType::And));
        set.insert(ChipType::Const(2));
        assert!(set.contains(ChipType::Const(1)));
        assert!(!set.contains(ChipType::And));
        set.insert(ChipType::And);
        assert!(set.contains(ChipType::Const(3)));
        assert!(set.contains(ChipType::And));
    }
}

//===========================================================================//
