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

pub use sdl2::keyboard::Keycode;
use serde;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use strum::IntoEnumIterator;

//===========================================================================//

#[derive(Clone, Copy, Debug, EnumCount, EnumIter, EnumString, Eq, Hash, Ord,
         PartialEq, PartialOrd)]
pub enum Hotkey {
    EvalReset,
    EvalRunPause,
    EvalStepCycle,
    EvalStepSubcycle,
    EvalStepTime,
    FlipHorz,
    FlipVert,
    RotateCcw,
    RotateCw,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    ScrollUp,
    ZoomIn,
    ZoomOut,
}

impl Hotkey {
    /// Returns an iterator over all hotkeys.
    pub fn all() -> HotkeyIter { Hotkey::iter() }

    pub fn name(self) -> &'static str {
        match self {
            Hotkey::EvalReset => "Reset evaluation",
            Hotkey::EvalRunPause => "Run/pause evaluation",
            Hotkey::EvalStepCycle => "Advance by one cycle",
            Hotkey::EvalStepSubcycle => "Advance by one subcycle",
            Hotkey::EvalStepTime => "Advance by one time step",
            Hotkey::FlipHorz => "Flip horzizontally",
            Hotkey::FlipVert => "Flip vertically",
            Hotkey::RotateCcw => "Rotate counterclockwise",
            Hotkey::RotateCw => "Rotate clockwise",
            Hotkey::ScrollDown => "Scroll down",
            Hotkey::ScrollLeft => "Scroll left",
            Hotkey::ScrollRight => "Scroll right",
            Hotkey::ScrollUp => "Scroll up",
            Hotkey::ZoomIn => "Zoom in",
            Hotkey::ZoomOut => "Zoom out",
        }
    }

    pub fn default_keycode(self) -> Keycode {
        match self {
            Hotkey::EvalReset => Keycode::T,
            Hotkey::EvalRunPause => Keycode::R,
            Hotkey::EvalStepCycle => Keycode::D,
            Hotkey::EvalStepSubcycle => Keycode::S,
            Hotkey::EvalStepTime => Keycode::F,
            Hotkey::FlipHorz => Keycode::A,
            Hotkey::FlipVert => Keycode::W,
            Hotkey::RotateCcw => Keycode::Q,
            Hotkey::RotateCw => Keycode::E,
            Hotkey::ScrollDown => Keycode::Down,
            Hotkey::ScrollLeft => Keycode::Left,
            Hotkey::ScrollRight => Keycode::Right,
            Hotkey::ScrollUp => Keycode::Up,
            Hotkey::ZoomIn => Keycode::Equals,
            Hotkey::ZoomOut => Keycode::Minus,
        }
    }

    pub fn default_for_keycode(code: Keycode) -> Option<Hotkey> {
        match code {
            Keycode::A => Some(Hotkey::FlipHorz),
            Keycode::D => Some(Hotkey::EvalStepCycle),
            Keycode::E => Some(Hotkey::RotateCw),
            Keycode::F => Some(Hotkey::EvalStepTime),
            Keycode::Q => Some(Hotkey::RotateCcw),
            Keycode::R => Some(Hotkey::EvalRunPause),
            Keycode::S => Some(Hotkey::EvalStepSubcycle),
            Keycode::T => Some(Hotkey::EvalReset),
            Keycode::W => Some(Hotkey::FlipVert),
            Keycode::Down => Some(Hotkey::ScrollDown),
            Keycode::Equals => Some(Hotkey::ZoomIn),
            Keycode::Left => Some(Hotkey::ScrollLeft),
            Keycode::Minus => Some(Hotkey::ZoomOut),
            Keycode::Right => Some(Hotkey::ScrollRight),
            Keycode::Up => Some(Hotkey::ScrollUp),
            _ => None,
        }
    }

    pub fn keycode_name(code: Keycode) -> &'static str {
        keycode_name(code).unwrap_or("<???>")
    }

    pub fn is_valid_keycode(code: Keycode) -> bool {
        keycode_name(code).is_some()
    }
}

//===========================================================================//

pub struct HotkeyCodes {
    hotkeys: HashMap<Keycode, Hotkey>,
    keycodes: HashMap<Hotkey, Keycode>,
}

impl HotkeyCodes {
    pub fn are_defaults(&self) -> bool {
        self.keycodes
            .iter()
            .all(|(&hotkey, &code)| code == hotkey.default_keycode())
    }

    pub fn hotkey(&self, keycode: Keycode) -> Option<Hotkey> {
        self.hotkeys.get(&keycode).copied()
    }

    pub fn keycode(&self, hotkey: Hotkey) -> Keycode {
        if let Some(&code) = self.keycodes.get(&hotkey) {
            code
        } else {
            hotkey.default_keycode()
        }
    }

    pub fn set_keycode(&mut self, hotkey: Hotkey, new_code: Keycode) {
        if !Hotkey::is_valid_keycode(new_code) {
            return;
        }
        let old_code = self.keycodes[&hotkey];
        if old_code == new_code {
            return;
        }
        if let Some(other_hotkey) = self.hotkeys.get(&new_code).copied() {
            self.hotkeys.insert(old_code, other_hotkey);
            self.keycodes.insert(other_hotkey, old_code);
        } else {
            self.hotkeys.remove(&old_code);
        }
        self.hotkeys.insert(new_code, hotkey);
        self.keycodes.insert(hotkey, new_code);
        debug_assert_eq!(self.keycodes.len(), HOTKEY_COUNT);
        debug_assert_eq!(self.hotkeys.len(), self.keycodes.len());
    }
}

impl Default for HotkeyCodes {
    fn default() -> HotkeyCodes {
        let mut hotkeys = HashMap::<Keycode, Hotkey>::new();
        let mut keycodes = HashMap::<Hotkey, Keycode>::new();
        for hotkey in Hotkey::all() {
            let code = hotkey.default_keycode();
            hotkeys.insert(code, hotkey);
            keycodes.insert(hotkey, code);
        }
        HotkeyCodes { hotkeys, keycodes }
    }
}

impl<'d> serde::Deserialize<'d> for HotkeyCodes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let mut hotkeys = HotkeyCodes::default();
        let map = BTreeMap::<&str, &str>::deserialize(deserializer)?;
        for (hotkey_name, keycode_name) in map.into_iter() {
            if let Ok(hotkey) = Hotkey::from_str(hotkey_name) {
                if let Some(keycode) = keycode_from_name(keycode_name) {
                    hotkeys.set_keycode(hotkey, keycode);
                }
            }
        }
        Ok(hotkeys)
    }
}

impl serde::Serialize for HotkeyCodes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.keycodes
            .iter()
            .filter(|(&hotkey, &code)| code != hotkey.default_keycode())
            .filter_map(|(&hotkey, &code)| {
                            keycode_name(code)
                                .map(|name| (format!("{:?}", hotkey), name))
                        })
            .collect::<BTreeMap<String, &str>>()
            .serialize(serializer)
    }
}

//===========================================================================//

fn keycode_name(keycode: Keycode) -> Option<&'static str> {
    match keycode {
        Keycode::Up => Some("Up"),
        Keycode::Down => Some("Down"),
        Keycode::Left => Some("Left"),
        Keycode::Right => Some("Right"),
        Keycode::A => Some("A"),
        Keycode::B => Some("B"),
        Keycode::C => Some("C"),
        Keycode::D => Some("D"),
        Keycode::E => Some("E"),
        Keycode::F => Some("F"),
        Keycode::G => Some("G"),
        Keycode::H => Some("H"),
        Keycode::I => Some("I"),
        Keycode::J => Some("J"),
        Keycode::K => Some("K"),
        Keycode::L => Some("L"),
        Keycode::M => Some("M"),
        Keycode::N => Some("N"),
        Keycode::O => Some("O"),
        Keycode::P => Some("P"),
        Keycode::Q => Some("Q"),
        Keycode::R => Some("R"),
        Keycode::S => Some("S"),
        Keycode::T => Some("T"),
        Keycode::U => Some("U"),
        Keycode::V => Some("V"),
        Keycode::W => Some("W"),
        Keycode::X => Some("X"),
        Keycode::Y => Some("Y"),
        Keycode::Z => Some("Z"),
        Keycode::Kp0 => Some("KP0"),
        Keycode::Kp1 => Some("KP1"),
        Keycode::Kp2 => Some("KP2"),
        Keycode::Kp3 => Some("KP3"),
        Keycode::Kp4 => Some("KP4"),
        Keycode::Kp5 => Some("KP5"),
        Keycode::Kp6 => Some("KP6"),
        Keycode::Kp7 => Some("KP7"),
        Keycode::Kp8 => Some("KP8"),
        Keycode::Kp9 => Some("KP9"),
        Keycode::Num0 => Some("0"),
        Keycode::Num1 => Some("1"),
        Keycode::Num2 => Some("2"),
        Keycode::Num3 => Some("3"),
        Keycode::Num4 => Some("4"),
        Keycode::Num5 => Some("5"),
        Keycode::Num6 => Some("6"),
        Keycode::Num7 => Some("7"),
        Keycode::Num8 => Some("8"),
        Keycode::Num9 => Some("9"),
        Keycode::Backquote => Some("`"),
        Keycode::Backslash => Some("\\"),
        Keycode::LeftBracket => Some("["),
        Keycode::RightBracket => Some("]"),
        Keycode::Comma => Some(","),
        Keycode::Equals => Some("="),
        Keycode::Minus => Some("-"),
        Keycode::Period => Some("."),
        Keycode::Quote => Some("'"),
        Keycode::Semicolon => Some(";"),
        Keycode::Slash => Some("/"),
        Keycode::Space => Some("Space"),
        Keycode::Tab => Some("Tab"),
        _ => None,
    }
}

fn keycode_from_name(name: &str) -> Option<Keycode> {
    match name {
        "Up" => Some(Keycode::Up),
        "Down" => Some(Keycode::Down),
        "Left" => Some(Keycode::Left),
        "Right" => Some(Keycode::Right),
        "A" => Some(Keycode::A),
        "B" => Some(Keycode::B),
        "C" => Some(Keycode::C),
        "D" => Some(Keycode::D),
        "E" => Some(Keycode::E),
        "F" => Some(Keycode::F),
        "G" => Some(Keycode::G),
        "H" => Some(Keycode::H),
        "I" => Some(Keycode::I),
        "J" => Some(Keycode::J),
        "K" => Some(Keycode::K),
        "L" => Some(Keycode::L),
        "M" => Some(Keycode::M),
        "N" => Some(Keycode::N),
        "O" => Some(Keycode::O),
        "P" => Some(Keycode::P),
        "Q" => Some(Keycode::Q),
        "R" => Some(Keycode::R),
        "S" => Some(Keycode::S),
        "T" => Some(Keycode::T),
        "U" => Some(Keycode::U),
        "V" => Some(Keycode::V),
        "W" => Some(Keycode::W),
        "X" => Some(Keycode::X),
        "Y" => Some(Keycode::Y),
        "Z" => Some(Keycode::Z),
        "KP0" => Some(Keycode::Kp0),
        "KP1" => Some(Keycode::Kp1),
        "KP2" => Some(Keycode::Kp2),
        "KP3" => Some(Keycode::Kp3),
        "KP4" => Some(Keycode::Kp4),
        "KP5" => Some(Keycode::Kp5),
        "KP6" => Some(Keycode::Kp6),
        "KP7" => Some(Keycode::Kp7),
        "KP8" => Some(Keycode::Kp8),
        "KP9" => Some(Keycode::Kp9),
        "0" => Some(Keycode::Num0),
        "1" => Some(Keycode::Num1),
        "2" => Some(Keycode::Num2),
        "3" => Some(Keycode::Num3),
        "4" => Some(Keycode::Num4),
        "5" => Some(Keycode::Num5),
        "6" => Some(Keycode::Num6),
        "7" => Some(Keycode::Num7),
        "8" => Some(Keycode::Num8),
        "9" => Some(Keycode::Num9),
        "`" => Some(Keycode::Backquote),
        "\\" => Some(Keycode::Backslash),
        "[" => Some(Keycode::LeftBracket),
        "]" => Some(Keycode::RightBracket),
        "," => Some(Keycode::Comma),
        "=" => Some(Keycode::Equals),
        "-" => Some(Keycode::Minus),
        "." => Some(Keycode::Period),
        "'" => Some(Keycode::Quote),
        ";" => Some(Keycode::Semicolon),
        "/" => Some(Keycode::Slash),
        "Space" => Some(Keycode::Space),
        "Tab" => Some(Keycode::Tab),
        _ => None,
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Hotkey, HotkeyCodes, Keycode};
    use std::str::{self, FromStr};
    use toml;

    #[test]
    fn hotkey_to_and_from_string() {
        for hotkey in Hotkey::all() {
            assert_eq!(Hotkey::from_str(&format!("{:?}", hotkey)), Ok(hotkey));
        }
    }

    #[test]
    fn default_hotkey_code_round_trip() {
        for hotkey in Hotkey::all() {
            assert_eq!(Hotkey::default_for_keycode(hotkey.default_keycode()),
                       Some(hotkey));
        }
    }

    #[test]
    fn default_has_defaults() {
        let hotkeys = HotkeyCodes::default();
        for hotkey in Hotkey::all() {
            assert_eq!(hotkeys.keycode(hotkey), hotkey.default_keycode());
        }
    }

    #[test]
    fn set_keycode_different() {
        let mut hotkeys = HotkeyCodes::default();
        let hotkey = Hotkey::RotateCw;
        let old_keycode = hotkeys.keycode(hotkey);
        let new_keycode = Keycode::Space;
        assert_ne!(old_keycode, new_keycode);
        hotkeys.set_keycode(hotkey, new_keycode);
        assert_eq!(hotkeys.keycode(hotkey), new_keycode);
    }

    #[test]
    fn set_keycode_same() {
        let mut hotkeys = HotkeyCodes::default();
        let hotkey = Hotkey::RotateCw;
        let keycode = hotkeys.keycode(hotkey);
        hotkeys.set_keycode(hotkey, keycode);
        assert_eq!(hotkeys.keycode(hotkey), keycode);
    }

    #[test]
    fn set_keycode_invalid() {
        let mut hotkeys = HotkeyCodes::default();
        let hotkey = Hotkey::RotateCw;
        let old_keycode = hotkeys.keycode(hotkey);
        let new_keycode = Keycode::LShift;
        assert_ne!(old_keycode, new_keycode);
        assert!(!Hotkey::is_valid_keycode(new_keycode));
        hotkeys.set_keycode(hotkey, new_keycode);
        assert_eq!(hotkeys.keycode(hotkey), old_keycode);
    }

    #[test]
    fn swap_keycodes() {
        let mut hotkeys = HotkeyCodes::default();
        let flip_horz_keycode = hotkeys.keycode(Hotkey::FlipHorz);
        let flip_vert_keycode = hotkeys.keycode(Hotkey::FlipVert);
        assert_ne!(flip_horz_keycode, flip_vert_keycode);
        hotkeys.set_keycode(Hotkey::FlipHorz, flip_vert_keycode);
        assert_eq!(hotkeys.keycode(Hotkey::FlipHorz), flip_vert_keycode);
        assert_eq!(hotkeys.keycode(Hotkey::FlipVert), flip_horz_keycode);
    }

    #[test]
    fn serialization() {
        let mut hotkeys = HotkeyCodes::default();
        hotkeys.set_keycode(Hotkey::FlipHorz, Keycode::Semicolon);
        hotkeys.set_keycode(Hotkey::FlipVert, Keycode::Tab);
        let bytes = toml::to_vec(&hotkeys).unwrap();
        assert_eq!(str::from_utf8(&bytes).unwrap(),
                   "FlipHorz = \";\"\n\
                    FlipVert = \"Tab\"\n");
        let hotkeys: HotkeyCodes = toml::from_slice(&bytes).unwrap();
        assert_eq!(hotkeys.keycode(Hotkey::FlipHorz), Keycode::Semicolon);
        assert_eq!(hotkeys.keycode(Hotkey::FlipVert), Keycode::Tab);
    }
}

//===========================================================================//
