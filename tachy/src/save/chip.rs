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

use super::hotkey::HotkeyCode;
use crate::geom::CoordsSize;
use std::collections::HashSet;
use std::fmt;
use std::str;

//===========================================================================//

pub const MAX_COMMENT_CHARS: usize = 5;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChipType {
    Add,
    Add2Bit,
    And,
    Break(bool),
    Button(Option<HotkeyCode>),
    Clock,
    Cmp,
    CmpEq,
    Comment([u8; MAX_COMMENT_CHARS]),
    Const(u16),
    Delay,
    Demux,
    Discard,
    Display,
    EggTimer,
    Eq,
    Filter,
    Halve,
    Inc,
    Join,
    Latest,
    Mul,
    Mul4Bit,
    Mux,
    Not,
    Or,
    Pack,
    Ram,
    Random,
    Sample,
    Stopwatch,
    Sub,
    Toggle(bool),
    Unpack,
    Xor,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const CHIP_CATEGORIES: &[(&str, &[ChipType])] = &[
    ("Value", &[
        ChipType::Const(1),
        ChipType::Pack,
        ChipType::Unpack,
        ChipType::Discard,
        ChipType::Sample,
        ChipType::Join,
        ChipType::Random,
    ]),
    ("Arithmetic", &[
        ChipType::Add,
        ChipType::Add2Bit,
        ChipType::Inc,
        ChipType::Sub,
        ChipType::Mul,
        ChipType::Mul4Bit,
        ChipType::Halve,
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
        ChipType::Filter,
        ChipType::Demux,
    ]),
    ("Timing", &[
        ChipType::Delay,
        ChipType::Clock,
        ChipType::EggTimer,
        ChipType::Stopwatch,
    ]),
    ("Memory", &[
        ChipType::Latest,
        ChipType::Ram,
    ]),
    ("Debug", &[
        ChipType::Comment(*b"#    "),
        ChipType::Display,
        ChipType::Toggle(false),
        ChipType::Break(true),
        ChipType::Button(None),
    ]),
];

impl ChipType {
    /// Returns the width and height of the chip in its default orientation.
    pub fn size(self) -> CoordsSize {
        match self {
            ChipType::Ram => CoordsSize::new(2, 2),
            ChipType::Display | ChipType::EggTimer | ChipType::Stopwatch => {
                CoordsSize::new(2, 1)
            }
            _ => CoordsSize::new(1, 1),
        }
    }

    pub fn tooltip_format(self) -> String {
        let name = match self {
            ChipType::Add2Bit => "2-Bit Add".to_string(),
            ChipType::Break(false) => "Breakpoint (disabled)".to_string(),
            ChipType::Break(true) => "Breakpoint (enabled)".to_string(),
            ChipType::Comment(_) => "Comment".to_string(),
            ChipType::Const(value) => format!("Constant ({})", value),
            ChipType::EggTimer => "Egg Timer".to_string(),
            ChipType::Mul4Bit => "4-Bit Mul".to_string(),
            ChipType::Toggle(false) => "Toggle Switch (off)".to_string(),
            ChipType::Toggle(true) => "Toggle Switch (on)".to_string(),
            other => format!("{}", other),
        };
        let description = match self {
            ChipType::Add => {
                "Outputs the sum of the two inputs.  If the sum is too large \
                 for the wire size, the value will wrap around (for example, \
                 5 + 12 on a 4-bit wire will result in 1 instead of 17)."
            }
            ChipType::And => {
                "For each bit in the wire, the output is 1 if both inputs \
                 are 1, or 0 if either input is 0."
            }
            ChipType::Break(_) => {
                "Passes events through unchanged.  When enabled, \
                 automatically pauses the simulation whenever an event goes \
                 through.\n\
                 $'Right-click' to toggle whether the breakpoint is enabled."
            }
            ChipType::Clock => {
                "Sends an event at the beginning of a time step if it \
                 received at least one event during the previous time step."
            }
            ChipType::Cmp => {
                "Outputs 1 if the one input is strictly less than the other; \
                 outputs 0 otherwise."
            }
            ChipType::CmpEq => {
                "Outputs 1 if the one input is less than or equal to the \
                 other; outputs 0 otherwise."
            }
            ChipType::Comment(_) => {
                "Visually annotates part of the circuit with a short label, \
                 but has no effect while the circuit is running.\n\
                 $'Right-click' to change the comment text."
            }
            ChipType::Const(_) => {
                "Outputs a constant value.\n\
                 $'Right-click' on the chip to change the output value."
            }
            ChipType::Delay => {
                "Delays events by one cycle.  Allows for loops within \
                 circuits."
            }
            ChipType::Demux => {
                "When the behavior wire is 0, incoming events are sent \
                 through the antipodal port.  When the behavior wire is 1, \
                 incoming events are sent through the lateral port instead."
            }
            ChipType::Discard => {
                "Transforms value-carrying events into 0-bit events by \
                 discarding the value."
            }
            ChipType::Eq => {
                "Outputs 1 if the two inputs are equal; outputs 0 otherwise."
            }
            ChipType::Filter => {
                "When the behavior wire is 0, events pass through unchanged.  \
                 When the behavior wire is 1, incoming events are ignored."
            }
            ChipType::Halve => "Outputs half the input, rounded down.",
            ChipType::Join => {
                "Merges two event streams into one; when an event arrives at \
                 either input port, it is sent to the output port.  If an \
                 event arrives at both inputs simultaneously, the event from \
                 the antipodal input port is sent on, and the event from the \
                 lateral input port is ignored."
            }
            ChipType::Latest => {
                "Outputs the value of the most recent event to arrive (or \
                 zero if no events have arrived yet)."
            }
            ChipType::Mul => {
                "Outputs the product of the two inputs.  If the product is \
                 too large for the wire size, the value will wrap around (for \
                 example, 3 \u{d7} 6 on a 4-bit wire will result in 2 instead \
                 of 18)."
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
                 as many bits.  The antipodal input becomes the low bits of \
                 the output, and the lateral input becomes the high bits."
            }
            ChipType::Random => {
                "When an event arrives, generates a random output value, \
                 evenly distributed among all possible values for the size of \
                 the output wire."
            }
            ChipType::Sample => {
                "Transforms 0-bit events into value-carrying events by \
                 sampling the value of the behavior wire when an event \
                 arrives."
            }
            ChipType::Sub => {
                "Outputs the difference between the two inputs.  The result \
                 is always positive (for example, if the inputs are 3 and 5, \
                 the output will be 2, regardless of which input is which)."
            }
            ChipType::Toggle(_) => {
                "Outputs 0 or 1.  Can be toggled manually while the circuit \
                 is running.\n\
                 $'Right-click' to change the initial switch position."
            }
            ChipType::Unpack => {
                "Splits the input wire into two output wires, each with half \
                 as many bits.  The antipodal output has the low bits of the \
                 input, and the lateral output has the high bits."
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

impl fmt::Display for ChipType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ChipType::Button(None) => formatter.pad("Button"),
            ChipType::Button(Some(code)) => {
                formatter.pad(&format!("Button({:?})", code))
            }
            ChipType::Comment(bytes) => formatter
                .pad(&format!("Comment('{}')", escape(bytes).trim_end())),
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}

impl str::FromStr for ChipType {
    type Err = String;

    fn from_str(string: &str) -> Result<ChipType, String> {
        match string {
            "Add" => Ok(ChipType::Add),
            "Add2Bit" => Ok(ChipType::Add2Bit),
            "And" => Ok(ChipType::And),
            "Break" => Ok(ChipType::Break(true)),
            "Break(false)" => Ok(ChipType::Break(false)),
            "Break(true)" => Ok(ChipType::Break(true)),
            "Button" => Ok(ChipType::Button(None)),
            "Button(None)" => Ok(ChipType::Button(None)),
            "Clock" => Ok(ChipType::Clock),
            "Cmp" => Ok(ChipType::Cmp),
            "CmpEq" => Ok(ChipType::CmpEq),
            "Delay" => Ok(ChipType::Delay),
            "Demux" => Ok(ChipType::Demux),
            "Discard" => Ok(ChipType::Discard),
            "Display" => Ok(ChipType::Display),
            "EggTimer" => Ok(ChipType::EggTimer),
            "Eq" => Ok(ChipType::Eq),
            "Filter" => Ok(ChipType::Filter),
            "Halve" => Ok(ChipType::Halve),
            "Inc" => Ok(ChipType::Inc),
            "Join" => Ok(ChipType::Join),
            "Latest" => Ok(ChipType::Latest),
            "Mul" => Ok(ChipType::Mul),
            "Mul4Bit" => Ok(ChipType::Mul4Bit),
            "Mux" => Ok(ChipType::Mux),
            "Not" => Ok(ChipType::Not),
            "Or" => Ok(ChipType::Or),
            "Pack" => Ok(ChipType::Pack),
            "Ram" => Ok(ChipType::Ram),
            "Random" => Ok(ChipType::Random),
            "Sample" => Ok(ChipType::Sample),
            "Stopwatch" => Ok(ChipType::Stopwatch),
            "Sub" => Ok(ChipType::Sub),
            "Toggle(false)" => Ok(ChipType::Toggle(false)),
            "Toggle(true)" => Ok(ChipType::Toggle(true)),
            "Unpack" => Ok(ChipType::Unpack),
            "Xor" => Ok(ChipType::Xor),
            _ => {
                if let Some(inner) = within(string, "Button(Some(", "))") {
                    if let Ok(code) = inner.parse() {
                        return Ok(ChipType::Button(Some(code)));
                    }
                } else if let Some(inner) = within(string, "Button(", ")") {
                    if let Ok(code) = inner.parse() {
                        return Ok(ChipType::Button(Some(code)));
                    }
                } else if let Some(inner) = within(string, "Const(", ")") {
                    if let Ok(value) = inner.parse() {
                        return Ok(ChipType::Const(value));
                    }
                } else if let Some(inner) = within(string, "Comment('", "')") {
                    if let Some(bytes) = unescape(inner) {
                        return Ok(ChipType::Comment(bytes));
                    }
                } else if let Some(inner) = within(string, "Comment([", "])") {
                    let parts: Vec<&str> = inner.split(", ").collect();
                    if parts.len() <= MAX_COMMENT_CHARS {
                        let mut bytes = [b' '; MAX_COMMENT_CHARS];
                        for (index, part) in parts.into_iter().enumerate() {
                            if let Ok(byte) = part.parse() {
                                bytes[index] = byte;
                            } else {
                                return Err(string.to_string());
                            }
                        }
                        return Ok(ChipType::Comment(bytes));
                    }
                }
                return Err(string.to_string());
            }
        }
    }
}

fn escape(bytes: &[u8]) -> String {
    let mut escaped = String::new();
    for &byte in bytes.iter() {
        if byte == b'\'' {
            escaped.push_str("\\'");
        } else if byte == b'\\' {
            escaped.push_str("\\\\");
        } else if byte < b' ' || byte > b'~' {
            escaped = format!("{}\\x{:02x}", escaped, byte);
        } else {
            escaped.push(char::from(byte));
        }
    }
    escaped
}

fn unescape(string: &str) -> Option<[u8; MAX_COMMENT_CHARS]> {
    let mut bytes = [b' '; MAX_COMMENT_CHARS];
    let mut index: usize = 0;
    let mut chars = string.chars();
    while let Some(chr) = chars.next() {
        if index >= MAX_COMMENT_CHARS {
            return None;
        } else if chr == '\\' {
            match chars.next() {
                Some('\'') => bytes[index] = b'\'',
                Some('\\') => bytes[index] = b'\\',
                Some('x') => {
                    let next = chars.next();
                    match (next, chars.next()) {
                        (Some(c1), Some(c2)) => {
                            let cs = format!("{}{}", c1, c2);
                            match u8::from_str_radix(&cs, 16) {
                                Ok(byte) => {
                                    bytes[index] = byte;
                                }
                                _ => return None,
                            }
                        }
                        _ => return None,
                    }
                }
                _ => return None,
            }
        } else if chr >= ' ' && chr <= '~' && chr != '\'' {
            bytes[index] = chr as u8;
        } else {
            return None;
        }
        index += 1;
    }
    return Some(bytes);
}

fn within<'a>(string: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    if string.starts_with(prefix) && string.ends_with(suffix) {
        Some(&string[prefix.len()..(string.len() - suffix.len())])
    } else {
        None
    }
}

//===========================================================================//

pub struct ChipSet {
    ctypes: HashSet<ChipType>,
}

impl ChipSet {
    pub fn new() -> ChipSet {
        ChipSet { ctypes: HashSet::new() }
    }

    pub fn contains(&self, ctype: ChipType) -> bool {
        match ctype {
            ChipType::Break(_) => {
                self.ctypes.contains(&ChipType::Break(false))
            }
            ChipType::Button(_) => {
                self.ctypes.contains(&ChipType::Button(None))
            }
            ChipType::Comment(_) => {
                self.ctypes.contains(&ChipType::Comment(*b"     "))
            }
            ChipType::Const(_) => self.ctypes.contains(&ChipType::Const(0)),
            ChipType::Toggle(_) => {
                self.ctypes.contains(&ChipType::Toggle(false))
            }
            _ => self.ctypes.contains(&ctype),
        }
    }

    pub fn insert(&mut self, ctype: ChipType) {
        match ctype {
            ChipType::Break(_) => {
                self.ctypes.insert(ChipType::Break(false));
            }
            ChipType::Button(_) => {
                self.ctypes.insert(ChipType::Button(None));
            }
            ChipType::Comment(_) => {
                self.ctypes.insert(ChipType::Comment(*b"     "));
            }
            ChipType::Const(_) => {
                self.ctypes.insert(ChipType::Const(0));
            }
            ChipType::Toggle(_) => {
                self.ctypes.insert(ChipType::Toggle(false));
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
    use super::super::hotkey::HotkeyCode;
    use super::{ChipSet, ChipType, CHIP_CATEGORIES};
    use std::u16;

    #[test]
    fn chip_type_to_and_from_string() {
        let mut chip_types = vec![
            ChipType::Break(false),
            ChipType::Button(Some(HotkeyCode::M)),
            ChipType::Button(Some(HotkeyCode::Kp5)),
            ChipType::Comment(*b"Blarg"),
            ChipType::Comment(*b" \x1b\"~ "),
            ChipType::Const(0),
            ChipType::Const(13),
            ChipType::Const(u16::MAX),
            ChipType::Toggle(true),
        ];
        for &(_, ctypes) in CHIP_CATEGORIES.iter() {
            chip_types.extend_from_slice(ctypes);
        }
        for &ctype in chip_types.iter() {
            assert_eq!(format!("{}", ctype).parse(), Ok(ctype));
        }
        for &ctype in chip_types.iter() {
            assert_eq!(format!("{:?}", ctype).parse(), Ok(ctype));
        }
    }

    #[test]
    fn display_comment() {
        assert_eq!(
            format!("{}", ChipType::Comment(*b" \x1b\"~ ")),
            "Comment(' \\x1b\"~')"
        );
        assert_eq!(
            format!("{}", ChipType::Comment(*b"'\\'  ")),
            "Comment('\\'\\\\\\'')"
        );
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

        assert!(!set.contains(ChipType::Toggle(true)));
        set.insert(ChipType::Toggle(false));
        assert!(set.contains(ChipType::Toggle(true)));

        assert!(!set.contains(ChipType::Break(false)));
        set.insert(ChipType::Break(true));
        assert!(set.contains(ChipType::Break(false)));

        assert!(!set.contains(ChipType::Comment(*b"foo  ")));
        set.insert(ChipType::Comment(*b"bar  "));
        assert!(set.contains(ChipType::Comment(*b"foo  ")));
    }
}

//===========================================================================//
