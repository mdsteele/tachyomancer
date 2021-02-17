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
use super::size::WireSize;
use crate::geom::{CoordsSize, Fixed};
use std::collections::HashSet;
use std::fmt;
use std::mem;
use std::str;

//===========================================================================//

pub const MAX_COMMENT_CHARS: usize = 5;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChipType {
    AAdd,
    ACmp,
    AMul,
    Add,
    Add2Bit,
    And,
    Break(bool),
    Buffer,
    Button(Option<HotkeyCode>),
    Clock,
    Cmp,
    CmpEq,
    Coerce(WireSize),
    Comment([u8; MAX_COMMENT_CHARS]),
    Const(u8),
    Counter,
    Delay,
    Demux,
    Discard,
    Display,
    DocAn([u8; MAX_COMMENT_CHARS]),
    DocBv(WireSize, [u8; MAX_COMMENT_CHARS]),
    DocEv(WireSize, [u8; MAX_COMMENT_CHARS]),
    EggTimer,
    Eq,
    Halve,
    Inc,
    Integrate,
    Join,
    Latch,
    Latest,
    Meter,
    Mul,
    Mul4Bit,
    Mux,
    Neg,
    Not,
    Or,
    Pack,
    Queue,
    Ram,
    Random,
    Relay,
    Sample,
    Screen,
    Stack,
    Stopwatch,
    Sub,
    Toggle(bool),
    Unpack,
    Vref(Fixed),
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
        ChipType::Coerce(WireSize::Eight),
        ChipType::Vref(Fixed::ONE),
        ChipType::Random,
    ]),
    ("Arithmetic", &[
        ChipType::Add,
        ChipType::Add2Bit,
        ChipType::Inc,
        ChipType::Neg,
        ChipType::Sub,
        ChipType::Mul,
        ChipType::Mul4Bit,
        ChipType::Halve,
        ChipType::AAdd,
        ChipType::AMul,
    ]),
    ("Comparison", &[
        ChipType::Cmp,
        ChipType::CmpEq,
        ChipType::Eq,
        ChipType::ACmp,
    ]),
    ("Logic", &[
        ChipType::Not,
        ChipType::And,
        ChipType::Or,
        ChipType::Xor,
        ChipType::Mux,
        ChipType::Demux,
        ChipType::Relay,
    ]),
    ("Memory", &[
        ChipType::Latest,
        ChipType::Latch,
        ChipType::Counter,
        ChipType::Ram,
        ChipType::Stack,
        ChipType::Queue,
        ChipType::Integrate,
        ChipType::Screen,
    ]),
    ("Timing", &[
        ChipType::Delay,
        ChipType::Clock,
        ChipType::EggTimer,
        ChipType::Stopwatch,
        ChipType::Buffer,
    ]),
    ("Debug", &[
        ChipType::Comment(*b"#    "),
        ChipType::Display,
        ChipType::Break(true),
        ChipType::Meter,
        ChipType::Toggle(false),
        ChipType::Button(None),
    ]),
];

impl ChipType {
    /// Returns the width and height of the chip in its default orientation.
    pub fn size(self) -> CoordsSize {
        match self {
            ChipType::Counter
            | ChipType::Display
            | ChipType::EggTimer
            | ChipType::Stopwatch => CoordsSize::new(2, 1),
            ChipType::Meter
            | ChipType::Queue
            | ChipType::Ram
            | ChipType::Stack => CoordsSize::new(2, 2),
            ChipType::Screen => CoordsSize::new(5, 5),
            _ => CoordsSize::new(1, 1),
        }
    }

    pub fn tooltip_format(self) -> String {
        let name = match self {
            ChipType::Add2Bit => "2-Bit Add".to_string(),
            ChipType::Break(false) => "Breakpoint (disabled)".to_string(),
            ChipType::Break(true) => "Breakpoint (enabled)".to_string(),
            ChipType::Coerce(size) => {
                format!("Coerce ({}-bit)", size.num_bits())
            }
            ChipType::Comment(_) => "Comment".to_string(),
            ChipType::Const(value) => format!("Constant ({})", value),
            ChipType::EggTimer => "Egg Timer".to_string(),
            ChipType::Mul4Bit => "4-Bit Mul".to_string(),
            ChipType::Neg => "Negate".to_string(),
            ChipType::Toggle(false) => "Toggle Switch (off)".to_string(),
            ChipType::Toggle(true) => "Toggle Switch (on)".to_string(),
            ChipType::Vref(value) => format!("Vref ({:+})", value),
            other => format!("{}", other),
        };
        let description = match self {
            ChipType::AAdd => {
                "Outputs the sum of the two input voltages, clamped to the \
                 range [-1, +1]."
            }
            ChipType::ACmp => {
                "When an input event arrives, sends an output event with a \
                 value of 1 if the one input voltage is lower than the other, \
                 or 0 otherwise."
            }
            ChipType::AMul => "Outputs the product of the two input voltages.",
            ChipType::Add => {
                "Outputs the sum of the two inputs.\n\
                 $=$#size = [5, 2]\n\
                 [chips]\n\
                 p0p0 = \"f0-DocBv(4, '7')\"\n\
                 p0p1 = \"f0-DocBv(4, '5')\"\n\
                 p2p0 = 'f0-Add'\n\
                 p4p0 = \"f0-DocBv(4, '12')\"\n\
                 [wires]\n\
                 p0p0e = 'Stub'\n\
                 p0p1e = 'Stub'\n\
                 p1p0e = 'Straight'\n\
                 p1p0w = 'Straight'\n\
                 p1p1e = 'Straight'\n\
                 p1p1w = 'Straight'\n\
                 p2p0e = 'Stub'\n\
                 p2p0s = 'Stub'\n\
                 p2p0w = 'Stub'\n\
                 p2p1n = 'TurnRight'\n\
                 p2p1w = 'TurnLeft'\n\
                 p3p0e = 'Straight'\n\
                 p3p0w = 'Straight'\n\
                 p4p0w = 'Stub'\n\
                 #\n\
                 $<If the sum is too large for the wire size, the value will \
                 wrap around."
            }
            ChipType::Add2Bit => "TODO",
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
            ChipType::Buffer => {
                "Allows for feedback loops within analog circuits.  It takes \
                 one cycle for a change in voltage to propagate through the \
                 buffer."
            }
            ChipType::Button(_) => "TODO",
            ChipType::Clock => {
                "Sends an event at the beginning of a time step if it \
                 received at least one event during the previous time step.  \
                 Allows for loops within circuits."
            }
            ChipType::Cmp => {
                "Outputs 1 if the one input is strictly less than the other; \
                 outputs 0 otherwise."
            }
            ChipType::CmpEq => {
                "Outputs 1 if the one input is less than or equal to the \
                 other; outputs 0 otherwise."
            }
            ChipType::Coerce(_) => {
                "Passes the value through unchanged.  Use this to force a \
                 wire to be a particular size.\n\
                 $'Right-click' on the chip to change the wire size."
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
            ChipType::Counter => "TODO",
            ChipType::Delay => {
                "Delays events by one cycle.  Allows for loops within \
                 circuits."
            }
            ChipType::Demux => {
                "When $*Ctrl$* is 0, incoming events are sent to $*Out0$*.  \
                 When $*Ctrl$* is 1, incoming events are sent to $*Out1$* \
                 instead.\n\
                 $=$#size = [5, 3]\n\
                 [chips]\n\
                 p0p1 = \"f0-DocEv(0, 'In')\"\n\
                 p2p0 = \"f1-DocBv(1, 'Ctrl')\"\n\
                 p2p1 = 'f0-Demux'\n\
                 p4p1 = \"f0-DocEv(0, 'Out0')\"\n\
                 p4p2 = \"f0-DocEv(0, 'Out1')\"\n\
                 [wires]\n\
                 p0p1e = 'Stub'\n\
                 p1p1e = 'Straight'\n\
                 p1p1w = 'Straight'\n\
                 p2p0s = 'Stub'\n\
                 p2p1e = 'Stub'\n\
                 p2p1n = 'Stub'\n\
                 p2p1s = 'Stub'\n\
                 p2p1w = 'Stub'\n\
                 p2p2e = 'TurnRight'\n\
                 p2p2n = 'TurnLeft'\n\
                 p3p1e = 'Straight'\n\
                 p3p1w = 'Straight'\n\
                 p3p2e = 'Straight'\n\
                 p3p2w = 'Straight'\n\
                 p4p1w = 'Stub'\n\
                 p4p2w = 'Stub'\n\
                 #"
            }
            ChipType::Discard => {
                "Transforms value-carrying events into 0-bit events by \
                 discarding the value.\n\
                 $=$#size = [5, 1]\n\
                 [chips]\n\
                 p0p0 = \"f0-DocEv(4, 'In')\"\n\
                 p2p0 = 'f0-Discard'\n\
                 p4p0 = \"f0-DocEv(0, 'Out')\"\n\
                 [wires]\n\
                 p0p0e = 'Stub'\n\
                 p1p0e = 'Straight'\n\
                 p1p0w = 'Straight'\n\
                 p2p0e = 'Stub'\n\
                 p2p0w = 'Stub'\n\
                 p3p0e = 'Straight'\n\
                 p3p0w = 'Straight'\n\
                 p4p0w = 'Stub'\n\
                 #"
            }
            ChipType::Display => "Displays the input value.",
            ChipType::DocAn(_) => "",
            ChipType::DocBv(_, _) => "",
            ChipType::DocEv(_, _) => "",
            ChipType::EggTimer => "TODO",
            ChipType::Eq => {
                "Outputs 1 if the two inputs are equal; outputs 0 otherwise."
            }
            ChipType::Halve => "Outputs half the input, rounded down.",
            ChipType::Inc => "TODO",
            ChipType::Integrate => {
                "Outputs the integral of the input voltage over time, \
                 starting from a initial condition of zero.  Resets the \
                 output back to zero whenever a reset event arrives."
            }
            ChipType::Join => {
                "Merges two event streams into one; when an event arrives at \
                 either input port, it is sent to the output port.  If an \
                 event arrives at both inputs simultaneously, the event from \
                 the antipodal input port is sent on, and the event from the \
                 lateral input port is ignored."
            }
            ChipType::Latch => "TODO",
            ChipType::Latest => {
                "Outputs the value of the most recent event to arrive (or \
                 zero if no events have arrived yet).\n\
                 $=$#size = [5, 1]\n\
                 [chips]\n\
                 p0p0 = \"f0-DocEv(4, 'In')\"\n\
                 p2p0 = 'f0-Latest'\n\
                 p4p0 = \"f0-DocBv(4, 'Out')\"\n\
                 [wires]\n\
                 p0p0e = 'Stub'\n\
                 p1p0e = 'Straight'\n\
                 p1p0w = 'Straight'\n\
                 p2p0e = 'Stub'\n\
                 p2p0w = 'Stub'\n\
                 p3p0e = 'Straight'\n\
                 p3p0w = 'Straight'\n\
                 p4p0w = 'Stub'\n\
                 #"
            }
            ChipType::Meter => "Measures and displays the input voltage.",
            ChipType::Mul => {
                "Outputs the product of the two inputs.\n\
                 $=$#size = [5, 2]\n\
                 [chips]\n\
                 p0p0 = \"f0-DocBv(4, '4')\"\n\
                 p0p1 = \"f0-DocBv(4, '3')\"\n\
                 p2p0 = 'f0-Mul'\n\
                 p4p0 = \"f0-DocBv(4, '12')\"\n\
                 [wires]\n\
                 p0p0e = 'Stub'\n\
                 p0p1e = 'Stub'\n\
                 p1p0e = 'Straight'\n\
                 p1p0w = 'Straight'\n\
                 p1p1e = 'Straight'\n\
                 p1p1w = 'Straight'\n\
                 p2p0e = 'Stub'\n\
                 p2p0s = 'Stub'\n\
                 p2p0w = 'Stub'\n\
                 p2p1n = 'TurnRight'\n\
                 p2p1w = 'TurnLeft'\n\
                 p3p0e = 'Straight'\n\
                 p3p0w = 'Straight'\n\
                 p4p0w = 'Stub'\n\
                 #\n\
                 $<If the product is too large for the wire size, the value \
                 will wrap around."
            }
            ChipType::Mul4Bit => "TODO",
            ChipType::Mux => {
                "When $*Ctrl$* is 0, outputs the value of $*In0$*.  When \
                 $*Ctrl$* is 1, outputs the value of $*In1$* instead.\n\
                 $=$#size = [5, 3]\n\
                 [chips]\n\
                 p0p1 = \"f0-DocBv(1, 'In0')\"\n\
                 p0p2 = \"f0-DocBv(1, 'In1')\"\n\
                 p2p0 = \"f1-DocBv(1, 'Ctrl')\"\n\
                 p2p1 = 'f0-Mux'\n\
                 p4p1 = \"f0-DocBv(1, 'Out')\"\n\
                 [wires]\n\
                 p0p1e = 'Stub'\n\
                 p0p2e = 'Stub'\n\
                 p1p1e = 'Straight'\n\
                 p1p1w = 'Straight'\n\
                 p1p2e = 'Straight'\n\
                 p1p2w = 'Straight'\n\
                 p2p0s = 'Stub'\n\
                 p2p1e = 'Stub'\n\
                 p2p1n = 'Stub'\n\
                 p2p1s = 'Stub'\n\
                 p2p1w = 'Stub'\n\
                 p2p2n = 'TurnRight'\n\
                 p2p2w = 'TurnLeft'\n\
                 p3p1e = 'Straight'\n\
                 p3p1w = 'Straight'\n\
                 p4p1w = 'Stub'\n\
                 #"
            }
            ChipType::Neg => {
                "Negates the input, wrapping around based on the wire size.  \
                 Use this with an $*Add$* chip to perform subtraction.\n\
                 $=$#size = [5, 2]\n\
                 [chips]\n\
                 p0p0 = \"f0-DocBv(4, '12')\"\n\
                 p0p1 = \"f0-DocBv(4, '5')\"\n\
                 p1p1 = 'f0-Neg'\n\
                 p2p0 = 'f0-Add'\n\
                 p4p0 = \"f0-DocBv(4, '7')\"\n\
                 [wires]\n\
                 p0p0e = 'Stub'\n\
                 p0p1e = 'Stub'\n\
                 p1p0e = 'Straight'\n\
                 p1p0w = 'Straight'\n\
                 p1p1e = 'Stub'\n\
                 p1p1w = 'Stub'\n\
                 p2p0e = 'Stub'\n\
                 p2p0s = 'Stub'\n\
                 p2p0w = 'Stub'\n\
                 p2p1n = 'TurnRight'\n\
                 p2p1w = 'TurnLeft'\n\
                 p3p0e = 'Straight'\n\
                 p3p0w = 'Straight'\n\
                 p4p0w = 'Stub'\n\
                 #"
            }
            ChipType::Not => {
                "Inverts bits.  Each 0 bit in the input becomes a 1 bit in \
                 the output, and vice-versa.\n\
                 $=$#size = [5, 1]\n\
                 [chips]\n\
                 p0p0 = \"f0-DocBv(1, 'In')\"\n\
                 p2p0 = 'f0-Not'\n\
                 p4p0 = \"f0-DocBv(1, 'Out')\"\n\
                 [wires]\n\
                 p0p0e = 'Stub'\n\
                 p1p0e = 'Straight'\n\
                 p1p0w = 'Straight'\n\
                 p2p0e = 'Stub'\n\
                 p2p0w = 'Stub'\n\
                 p3p0e = 'Straight'\n\
                 p3p0w = 'Straight'\n\
                 p4p0w = 'Stub'\n\
                 #"
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
            ChipType::Queue => "TODO",
            ChipType::Ram => {
                "Stores an array of values (initially all zero).  Each \
                 $*Val$* port gives the current value of the cell specified \
                 by the corresponding $*Addr$* port.  When a $*Set$* event \
                 arrives, sets the value of that cell.\n\
                 $=$#size = [4, 4]\n\
                 [chips]\n\
                 p0p0 = \"f0-DocEv(4, 'Set1')\"\n\
                 p0p1 = \"f0-DocBv(8, 'Addr1')\"\n\
                 p0p2 = \"f2-DocBv(4, 'Val1')\"\n\
                 p1p1 = 'f0-Ram'\n\
                 p3p1 = \"f0-DocBv(4, 'Val2')\"\n\
                 p3p2 = \"f2-DocBv(8, 'Addr2')\"\n\
                 p3p3 = \"f2-DocEv(4, 'Set2')\"\n\
                 [wires]\n\
                 p0p0e = 'Stub'\n\
                 p0p1e = 'Stub'\n\
                 p0p2e = 'Stub'\n\
                 p1p0s = 'TurnLeft'\n\
                 p1p0w = 'TurnRight'\n\
                 p1p1n = 'Stub'\n\
                 p1p1w = 'Stub'\n\
                 p1p2w = 'Stub'\n\
                 p2p1e = 'Stub'\n\
                 p2p2e = 'Stub'\n\
                 p2p2s = 'Stub'\n\
                 p2p3e = 'TurnRight'\n\
                 p2p3n = 'TurnLeft'\n\
                 p3p1w = 'Stub'\n\
                 p3p2w = 'Stub'\n\
                 p3p3w = 'Stub'\n\
                 #"
            }
            ChipType::Random => {
                "When an event arrives, generates a random output value, \
                 evenly distributed among all possible values for the size of \
                 the output wire."
            }
            ChipType::Relay => {
                "When $*Ctrl$* is 0, outputs the $*In0$* voltage.  When \
                 $*Ctrl$* is 1, outputs the $*In1$* voltage instead.\n\
                 $=$#size = [5, 3]\n\
                 [chips]\n\
                 p0p1 = \"f0-DocAn('In0')\"\n\
                 p0p2 = \"f0-DocAn('In1')\"\n\
                 p2p0 = \"f1-DocBv(1, 'Ctrl')\"\n\
                 p2p1 = 'f0-Relay'\n\
                 p4p1 = \"f0-DocAn('Out')\"\n\
                 [wires]\n\
                 p0p1e = 'Stub'\n\
                 p0p2e = 'Stub'\n\
                 p1p1e = 'Straight'\n\
                 p1p1w = 'Straight'\n\
                 p1p2e = 'Straight'\n\
                 p1p2w = 'Straight'\n\
                 p2p0s = 'Stub'\n\
                 p2p1e = 'Stub'\n\
                 p2p1n = 'Stub'\n\
                 p2p1s = 'Stub'\n\
                 p2p1w = 'Stub'\n\
                 p2p2n = 'TurnRight'\n\
                 p2p2w = 'TurnLeft'\n\
                 p3p1e = 'Straight'\n\
                 p3p1w = 'Straight'\n\
                 p4p1w = 'Stub'\n\
                 #"
            }
            ChipType::Sample => {
                "Transforms 0-bit events into value-carrying events by \
                 sampling the value of the behavior wire when an event \
                 arrives."
            }
            ChipType::Screen => "TODO",
            ChipType::Stack => "TODO",
            ChipType::Stopwatch => "TODO",
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
            ChipType::Vref(_) => {
                "Outputs a constant reference voltage.\n\
                 $'Right-click' on the chip to change the output voltage."
            }
            ChipType::Xor => {
                "For each bit in the wire, the output is 1 if exactly one \
                 input is 1, or 0 if the inputs are both 0 or both 1."
            }
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
            ChipType::Coerce(size) => {
                formatter.pad(&format!("Coerce({})", size.num_bits()))
            }
            ChipType::Comment(bytes) => formatter
                .pad(&format!("Comment('{}')", escape(bytes).trim_end())),
            ChipType::DocAn(bytes) => formatter
                .pad(&format!("DocAn('{}')", escape(bytes).trim_end())),
            ChipType::DocBv(size, bytes) => formatter.pad(&format!(
                "DocBv({}, '{}')",
                size.num_bits(),
                escape(bytes).trim_end()
            )),
            ChipType::DocEv(size, bytes) => formatter.pad(&format!(
                "DocEv({}, '{}')",
                size.num_bits(),
                escape(bytes).trim_end()
            )),
            ChipType::Vref(value) => {
                formatter.pad(&format!("Vref({:+})", value))
            }
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}

impl str::FromStr for ChipType {
    type Err = String;

    fn from_str(string: &str) -> Result<ChipType, String> {
        match string {
            "AAdd" => Ok(ChipType::AAdd),
            "ACmp" => Ok(ChipType::ACmp),
            "AMul" => Ok(ChipType::AMul),
            "Add" => Ok(ChipType::Add),
            "Add2Bit" => Ok(ChipType::Add2Bit),
            "And" => Ok(ChipType::And),
            "Break" => Ok(ChipType::Break(true)),
            "Break(false)" => Ok(ChipType::Break(false)),
            "Break(true)" => Ok(ChipType::Break(true)),
            "Buffer" => Ok(ChipType::Buffer),
            "Button" => Ok(ChipType::Button(None)),
            "Button(None)" => Ok(ChipType::Button(None)),
            "Clock" => Ok(ChipType::Clock),
            "Cmp" => Ok(ChipType::Cmp),
            "CmpEq" => Ok(ChipType::CmpEq),
            "Counter" => Ok(ChipType::Counter),
            "Delay" => Ok(ChipType::Delay),
            "Demux" => Ok(ChipType::Demux),
            "Discard" => Ok(ChipType::Discard),
            "Display" => Ok(ChipType::Display),
            "EggTimer" => Ok(ChipType::EggTimer),
            "Eq" => Ok(ChipType::Eq),
            "Halve" => Ok(ChipType::Halve),
            "Inc" => Ok(ChipType::Inc),
            "Integrate" => Ok(ChipType::Integrate),
            "Join" => Ok(ChipType::Join),
            "Latch" => Ok(ChipType::Latch),
            "Latest" => Ok(ChipType::Latest),
            "Meter" => Ok(ChipType::Meter),
            "Mul" => Ok(ChipType::Mul),
            "Mul4Bit" => Ok(ChipType::Mul4Bit),
            "Mux" => Ok(ChipType::Mux),
            "Neg" => Ok(ChipType::Neg),
            "Not" => Ok(ChipType::Not),
            "Or" => Ok(ChipType::Or),
            "Pack" => Ok(ChipType::Pack),
            "Queue" => Ok(ChipType::Queue),
            "Ram" => Ok(ChipType::Ram),
            "Random" => Ok(ChipType::Random),
            "Relay" => Ok(ChipType::Relay),
            "Sample" => Ok(ChipType::Sample),
            "Screen" => Ok(ChipType::Screen),
            "Stack" => Ok(ChipType::Stack),
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
                } else if let Some(inner) = within(string, "Coerce(", ")") {
                    if let Ok(size) = inner.parse() {
                        if size != WireSize::Zero {
                            return Ok(ChipType::Coerce(size));
                        }
                    }
                } else if let Some(inner) = within(string, "Const(", ")") {
                    if let Ok(value) = inner.parse() {
                        return Ok(ChipType::Const(value));
                    }
                } else if let Some(inner) = within(string, "Comment(", ")") {
                    if let Some(bytes) = parse_comment_bytes(inner) {
                        return Ok(ChipType::Comment(bytes));
                    }
                } else if let Some(inner) = within(string, "DocAn(", ")") {
                    if let Some(bytes) = parse_comment_bytes(inner) {
                        return Ok(ChipType::DocAn(bytes));
                    }
                } else if let Some(inner) = within(string, "DocBv(", ")") {
                    if let Some((size, bytes)) = parse_size_and_comment(inner)
                    {
                        if size != WireSize::Zero {
                            return Ok(ChipType::DocBv(size, bytes));
                        }
                    }
                } else if let Some(inner) = within(string, "DocEv(", ")") {
                    if let Some((size, bytes)) = parse_size_and_comment(inner)
                    {
                        return Ok(ChipType::DocEv(size, bytes));
                    }
                } else if let Some(inner) = within(string, "Vref(Fixed(", "))")
                {
                    if let Ok(value) = inner.parse::<i32>() {
                        return Ok(ChipType::Vref(Fixed::new(value)));
                    }
                } else if let Some(inner) = within(string, "Vref(", ")") {
                    if let Ok(value) = inner.parse::<f64>() {
                        return Ok(ChipType::Vref(Fixed::from_f64(value)));
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

fn parse_list(string: &str) -> Option<[u8; MAX_COMMENT_CHARS]> {
    let parts: Vec<&str> = string.split(", ").collect();
    if parts.len() <= MAX_COMMENT_CHARS {
        let mut bytes = [b' '; MAX_COMMENT_CHARS];
        for (index, part) in parts.into_iter().enumerate() {
            if let Ok(byte) = part.parse() {
                bytes[index] = byte;
            } else {
                return None;
            }
        }
        return Some(bytes);
    }
    return None;
}

fn parse_comment_bytes(string: &str) -> Option<[u8; MAX_COMMENT_CHARS]> {
    if let Some(inner) = within(string, "'", "'") {
        unescape(inner)
    } else if let Some(inner) = within(string, "[", "]") {
        parse_list(inner)
    } else {
        None
    }
}

fn parse_size_and_comment(
    string: &str,
) -> Option<(WireSize, [u8; MAX_COMMENT_CHARS])> {
    let parts: Vec<&str> = string.splitn(2, ", ").collect();
    if parts.len() == 2 {
        if let Ok(wire_size) = parts[0].parse() {
            if let Some(bytes) = parse_comment_bytes(parts[1]) {
                return Some((wire_size, bytes));
            }
        }
    }
    return None;
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
    inner: HashSet<mem::Discriminant<ChipType>>,
}

impl ChipSet {
    pub fn new() -> ChipSet {
        ChipSet { inner: HashSet::new() }
    }

    pub fn contains(&self, ctype: ChipType) -> bool {
        self.inner.contains(&mem::discriminant(&ctype))
    }

    pub fn insert(&mut self, ctype: ChipType) {
        self.inner.insert(mem::discriminant(&ctype));
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::super::hotkey::HotkeyCode;
    use super::super::size::WireSize;
    use super::{ChipSet, ChipType, CHIP_CATEGORIES};
    use crate::geom::Fixed;
    use std::u8;

    #[test]
    fn chip_type_is_small() {
        assert!(std::mem::size_of::<ChipType>() <= std::mem::size_of::<u64>());
    }

    #[test]
    fn chip_type_to_and_from_string() {
        let mut chip_types = vec![
            ChipType::Break(false),
            ChipType::Button(Some(HotkeyCode::M)),
            ChipType::Button(Some(HotkeyCode::Kp5)),
            ChipType::Coerce(WireSize::One),
            ChipType::Coerce(WireSize::Two),
            ChipType::Coerce(WireSize::Four),
            ChipType::Comment(*b"Blarg"),
            ChipType::Comment(*b" \x1b\"~ "),
            ChipType::Const(0),
            ChipType::Const(13),
            ChipType::Const(u8::MAX),
            ChipType::DocAn(*b"'\"): "),
            ChipType::DocBv(WireSize::One, *b"Blarg"),
            ChipType::DocBv(WireSize::Two, *b" \x1b\"~ "),
            ChipType::DocBv(WireSize::Four, *b"Foo  "),
            ChipType::DocBv(WireSize::Eight, *b" Bar "),
            ChipType::DocEv(WireSize::Zero, *b"'\"):@"),
            ChipType::DocEv(WireSize::One, *b"Blarg"),
            ChipType::DocEv(WireSize::Two, *b" \x1b\"~ "),
            ChipType::DocEv(WireSize::Four, *b"Foo  "),
            ChipType::DocEv(WireSize::Eight, *b" Bar "),
            ChipType::Toggle(true),
            ChipType::Vref(Fixed::ZERO),
            ChipType::Vref(-Fixed::ONE),
            ChipType::Vref(Fixed::from_f64(0.25)),
            ChipType::Vref(Fixed::from_f64(std::f64::consts::FRAC_1_PI)),
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

        assert!(!set.contains(ChipType::Coerce(WireSize::Two)));
        set.insert(ChipType::Coerce(WireSize::Four));
        assert!(set.contains(ChipType::Coerce(WireSize::Two)));

        assert!(!set.contains(ChipType::Comment(*b"foo  ")));
        set.insert(ChipType::Comment(*b"bar  "));
        assert!(set.contains(ChipType::Comment(*b"foo  ")));
    }
}

//===========================================================================//
