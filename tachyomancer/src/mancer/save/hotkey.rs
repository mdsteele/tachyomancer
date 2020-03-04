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
use tachy::save::HotkeyCode;

//===========================================================================//

#[derive(
    Clone,
    Copy,
    Debug,
    EnumCount,
    EnumIter,
    EnumString,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Hotkey {
    EvalFastForward,
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
    ZoomDefault,
    ZoomIn,
    ZoomOut,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const HOTKEY_CATEGORIES: &[(&str, &[Hotkey])] = &[
    ("Evaluation", &[
        Hotkey::EvalRunPause,
        Hotkey::EvalFastForward,
        Hotkey::EvalReset,
        Hotkey::EvalStepTime,
        Hotkey::EvalStepCycle,
        Hotkey::EvalStepSubcycle,
    ]),
    ("Selection", &[
        Hotkey::RotateCw,
        Hotkey::RotateCcw,
        Hotkey::FlipHorz,
        Hotkey::FlipVert,
    ]),
    ("Camera", &[
        Hotkey::ScrollUp,
        Hotkey::ScrollDown,
        Hotkey::ScrollLeft,
        Hotkey::ScrollRight,
        Hotkey::ZoomIn,
        Hotkey::ZoomOut,
        Hotkey::ZoomDefault,
    ]),
];

impl Hotkey {
    /// Returns an iterator over all hotkeys.
    pub fn all() -> HotkeyIter {
        Hotkey::iter()
    }

    pub fn name(self) -> &'static str {
        match self {
            Hotkey::EvalFastForward => "Fast-forward evaluation",
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
            Hotkey::ZoomDefault => "Zoom to actual size",
            Hotkey::ZoomIn => "Zoom in",
            Hotkey::ZoomOut => "Zoom out",
        }
    }

    pub fn default_keycode(self) -> HotkeyCode {
        match self {
            Hotkey::EvalFastForward => HotkeyCode::G,
            Hotkey::EvalReset => HotkeyCode::T,
            Hotkey::EvalRunPause => HotkeyCode::R,
            Hotkey::EvalStepCycle => HotkeyCode::D,
            Hotkey::EvalStepSubcycle => HotkeyCode::S,
            Hotkey::EvalStepTime => HotkeyCode::F,
            Hotkey::FlipHorz => HotkeyCode::A,
            Hotkey::FlipVert => HotkeyCode::W,
            Hotkey::RotateCcw => HotkeyCode::Q,
            Hotkey::RotateCw => HotkeyCode::E,
            Hotkey::ScrollDown => HotkeyCode::Down,
            Hotkey::ScrollLeft => HotkeyCode::Left,
            Hotkey::ScrollRight => HotkeyCode::Right,
            Hotkey::ScrollUp => HotkeyCode::Up,
            Hotkey::ZoomDefault => HotkeyCode::Num0,
            Hotkey::ZoomIn => HotkeyCode::Equals,
            Hotkey::ZoomOut => HotkeyCode::Minus,
        }
    }

    pub fn default_for_keycode(code: HotkeyCode) -> Option<Hotkey> {
        match code {
            HotkeyCode::A => Some(Hotkey::FlipHorz),
            HotkeyCode::D => Some(Hotkey::EvalStepCycle),
            HotkeyCode::E => Some(Hotkey::RotateCw),
            HotkeyCode::F => Some(Hotkey::EvalStepTime),
            HotkeyCode::G => Some(Hotkey::EvalFastForward),
            HotkeyCode::Q => Some(Hotkey::RotateCcw),
            HotkeyCode::R => Some(Hotkey::EvalRunPause),
            HotkeyCode::S => Some(Hotkey::EvalStepSubcycle),
            HotkeyCode::T => Some(Hotkey::EvalReset),
            HotkeyCode::W => Some(Hotkey::FlipVert),
            HotkeyCode::Num0 => Some(Hotkey::ZoomDefault),
            HotkeyCode::Down => Some(Hotkey::ScrollDown),
            HotkeyCode::Equals => Some(Hotkey::ZoomIn),
            HotkeyCode::Left => Some(Hotkey::ScrollLeft),
            HotkeyCode::Minus => Some(Hotkey::ZoomOut),
            HotkeyCode::Right => Some(Hotkey::ScrollRight),
            HotkeyCode::Up => Some(Hotkey::ScrollUp),
            _ => None,
        }
    }
}

//===========================================================================//

pub struct HotkeyCodes {
    hotkeys: HashMap<HotkeyCode, Hotkey>,
    keycodes: HashMap<Hotkey, HotkeyCode>,
}

impl HotkeyCodes {
    pub fn are_defaults(&self) -> bool {
        self.keycodes
            .iter()
            .all(|(&hotkey, &code)| code == hotkey.default_keycode())
    }

    pub fn hotkey(&self, keycode: HotkeyCode) -> Option<Hotkey> {
        self.hotkeys.get(&keycode).copied()
    }

    pub fn keycode(&self, hotkey: Hotkey) -> HotkeyCode {
        if let Some(&code) = self.keycodes.get(&hotkey) {
            code
        } else {
            hotkey.default_keycode()
        }
    }

    pub fn set_keycode(&mut self, hotkey: Hotkey, new_code: HotkeyCode) {
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
        let mut hotkeys = HashMap::<HotkeyCode, Hotkey>::new();
        let mut keycodes = HashMap::<Hotkey, HotkeyCode>::new();
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
        let map = BTreeMap::<&str, HotkeyCode>::deserialize(deserializer)?;
        for (hotkey_name, code) in map.into_iter() {
            if let Ok(hotkey) = Hotkey::from_str(hotkey_name) {
                hotkeys.set_keycode(hotkey, code);
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
            .map(|(&hotkey, &code)| (format!("{:?}", hotkey), code))
            .collect::<BTreeMap<String, HotkeyCode>>()
            .serialize(serializer)
    }
}

//===========================================================================//

pub trait HotkeyCodeExt
where
    Self: Sized,
{
    fn from_keycode(keycode: Keycode) -> Option<Self>;

    fn to_keycode(&self) -> Keycode;
}

impl HotkeyCodeExt for HotkeyCode {
    fn from_keycode(keycode: Keycode) -> Option<HotkeyCode> {
        match keycode {
            Keycode::Up => Some(HotkeyCode::Up),
            Keycode::Down => Some(HotkeyCode::Down),
            Keycode::Left => Some(HotkeyCode::Left),
            Keycode::Right => Some(HotkeyCode::Right),
            Keycode::A => Some(HotkeyCode::A),
            Keycode::B => Some(HotkeyCode::B),
            Keycode::C => Some(HotkeyCode::C),
            Keycode::D => Some(HotkeyCode::D),
            Keycode::E => Some(HotkeyCode::E),
            Keycode::F => Some(HotkeyCode::F),
            Keycode::G => Some(HotkeyCode::G),
            Keycode::H => Some(HotkeyCode::H),
            Keycode::I => Some(HotkeyCode::I),
            Keycode::J => Some(HotkeyCode::J),
            Keycode::K => Some(HotkeyCode::K),
            Keycode::L => Some(HotkeyCode::L),
            Keycode::M => Some(HotkeyCode::M),
            Keycode::N => Some(HotkeyCode::N),
            Keycode::O => Some(HotkeyCode::O),
            Keycode::P => Some(HotkeyCode::P),
            Keycode::Q => Some(HotkeyCode::Q),
            Keycode::R => Some(HotkeyCode::R),
            Keycode::S => Some(HotkeyCode::S),
            Keycode::T => Some(HotkeyCode::T),
            Keycode::U => Some(HotkeyCode::U),
            Keycode::V => Some(HotkeyCode::V),
            Keycode::W => Some(HotkeyCode::W),
            Keycode::X => Some(HotkeyCode::X),
            Keycode::Y => Some(HotkeyCode::Y),
            Keycode::Z => Some(HotkeyCode::Z),
            Keycode::Kp0 => Some(HotkeyCode::Kp0),
            Keycode::Kp1 => Some(HotkeyCode::Kp1),
            Keycode::Kp2 => Some(HotkeyCode::Kp2),
            Keycode::Kp3 => Some(HotkeyCode::Kp3),
            Keycode::Kp4 => Some(HotkeyCode::Kp4),
            Keycode::Kp5 => Some(HotkeyCode::Kp5),
            Keycode::Kp6 => Some(HotkeyCode::Kp6),
            Keycode::Kp7 => Some(HotkeyCode::Kp7),
            Keycode::Kp8 => Some(HotkeyCode::Kp8),
            Keycode::Kp9 => Some(HotkeyCode::Kp9),
            Keycode::Num0 => Some(HotkeyCode::Num0),
            Keycode::Num1 => Some(HotkeyCode::Num1),
            Keycode::Num2 => Some(HotkeyCode::Num2),
            Keycode::Num3 => Some(HotkeyCode::Num3),
            Keycode::Num4 => Some(HotkeyCode::Num4),
            Keycode::Num5 => Some(HotkeyCode::Num5),
            Keycode::Num6 => Some(HotkeyCode::Num6),
            Keycode::Num7 => Some(HotkeyCode::Num7),
            Keycode::Num8 => Some(HotkeyCode::Num8),
            Keycode::Num9 => Some(HotkeyCode::Num9),
            Keycode::Backquote => Some(HotkeyCode::Backquote),
            Keycode::Backslash => Some(HotkeyCode::Backslash),
            Keycode::LeftBracket => Some(HotkeyCode::LeftBracket),
            Keycode::RightBracket => Some(HotkeyCode::RightBracket),
            Keycode::Comma => Some(HotkeyCode::Comma),
            Keycode::Equals => Some(HotkeyCode::Equals),
            Keycode::Minus => Some(HotkeyCode::Minus),
            Keycode::Period => Some(HotkeyCode::Period),
            Keycode::Quote => Some(HotkeyCode::Quote),
            Keycode::Semicolon => Some(HotkeyCode::Semicolon),
            Keycode::Slash => Some(HotkeyCode::Slash),
            Keycode::Space => Some(HotkeyCode::Space),
            Keycode::Tab => Some(HotkeyCode::Tab),
            _ => None,
        }
    }

    fn to_keycode(&self) -> Keycode {
        match *self {
            HotkeyCode::Up => Keycode::Up,
            HotkeyCode::Down => Keycode::Down,
            HotkeyCode::Left => Keycode::Left,
            HotkeyCode::Right => Keycode::Right,
            HotkeyCode::A => Keycode::A,
            HotkeyCode::B => Keycode::B,
            HotkeyCode::C => Keycode::C,
            HotkeyCode::D => Keycode::D,
            HotkeyCode::E => Keycode::E,
            HotkeyCode::F => Keycode::F,
            HotkeyCode::G => Keycode::G,
            HotkeyCode::H => Keycode::H,
            HotkeyCode::I => Keycode::I,
            HotkeyCode::J => Keycode::J,
            HotkeyCode::K => Keycode::K,
            HotkeyCode::L => Keycode::L,
            HotkeyCode::M => Keycode::M,
            HotkeyCode::N => Keycode::N,
            HotkeyCode::O => Keycode::O,
            HotkeyCode::P => Keycode::P,
            HotkeyCode::Q => Keycode::Q,
            HotkeyCode::R => Keycode::R,
            HotkeyCode::S => Keycode::S,
            HotkeyCode::T => Keycode::T,
            HotkeyCode::U => Keycode::U,
            HotkeyCode::V => Keycode::V,
            HotkeyCode::W => Keycode::W,
            HotkeyCode::X => Keycode::X,
            HotkeyCode::Y => Keycode::Y,
            HotkeyCode::Z => Keycode::Z,
            HotkeyCode::Kp0 => Keycode::Kp0,
            HotkeyCode::Kp1 => Keycode::Kp1,
            HotkeyCode::Kp2 => Keycode::Kp2,
            HotkeyCode::Kp3 => Keycode::Kp3,
            HotkeyCode::Kp4 => Keycode::Kp4,
            HotkeyCode::Kp5 => Keycode::Kp5,
            HotkeyCode::Kp6 => Keycode::Kp6,
            HotkeyCode::Kp7 => Keycode::Kp7,
            HotkeyCode::Kp8 => Keycode::Kp8,
            HotkeyCode::Kp9 => Keycode::Kp9,
            HotkeyCode::Num0 => Keycode::Num0,
            HotkeyCode::Num1 => Keycode::Num1,
            HotkeyCode::Num2 => Keycode::Num2,
            HotkeyCode::Num3 => Keycode::Num3,
            HotkeyCode::Num4 => Keycode::Num4,
            HotkeyCode::Num5 => Keycode::Num5,
            HotkeyCode::Num6 => Keycode::Num6,
            HotkeyCode::Num7 => Keycode::Num7,
            HotkeyCode::Num8 => Keycode::Num8,
            HotkeyCode::Num9 => Keycode::Num9,
            HotkeyCode::Backquote => Keycode::Backquote,
            HotkeyCode::Backslash => Keycode::Backslash,
            HotkeyCode::LeftBracket => Keycode::LeftBracket,
            HotkeyCode::RightBracket => Keycode::RightBracket,
            HotkeyCode::Comma => Keycode::Comma,
            HotkeyCode::Equals => Keycode::Equals,
            HotkeyCode::Minus => Keycode::Minus,
            HotkeyCode::Period => Keycode::Period,
            HotkeyCode::Quote => Keycode::Quote,
            HotkeyCode::Semicolon => Keycode::Semicolon,
            HotkeyCode::Slash => Keycode::Slash,
            HotkeyCode::Space => Keycode::Space,
            HotkeyCode::Tab => Keycode::Tab,
        }
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Hotkey, HotkeyCode, HotkeyCodes, HOTKEY_CATEGORIES};
    use std::collections::HashSet;
    use std::str::{self, FromStr};
    use toml;

    #[test]
    fn categories_contain_all_hotkeys() {
        let mut remaining: HashSet<Hotkey> = Hotkey::all().collect();
        for &(_, hotkeys) in HOTKEY_CATEGORIES.iter() {
            for hotkey in hotkeys {
                assert!(remaining.contains(&hotkey));
                remaining.remove(&hotkey);
            }
        }
        assert!(remaining.is_empty());
    }

    #[test]
    fn hotkey_to_and_from_string() {
        for hotkey in Hotkey::all() {
            assert_eq!(Hotkey::from_str(&format!("{:?}", hotkey)), Ok(hotkey));
        }
    }

    #[test]
    fn default_hotkey_code_round_trip() {
        for hotkey in Hotkey::all() {
            assert_eq!(
                Hotkey::default_for_keycode(hotkey.default_keycode()),
                Some(hotkey)
            );
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
        let new_keycode = HotkeyCode::Space;
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
        hotkeys.set_keycode(Hotkey::FlipHorz, HotkeyCode::Semicolon);
        hotkeys.set_keycode(Hotkey::FlipVert, HotkeyCode::Tab);
        let bytes = toml::to_vec(&hotkeys).unwrap();
        assert_eq!(
            str::from_utf8(&bytes).unwrap(),
            "FlipHorz = \"Semicolon\"\n\
             FlipVert = \"Tab\"\n"
        );
        let hotkeys: HotkeyCodes = toml::from_slice(&bytes).unwrap();
        assert_eq!(hotkeys.keycode(Hotkey::FlipHorz), HotkeyCode::Semicolon);
        assert_eq!(hotkeys.keycode(Hotkey::FlipVert), HotkeyCode::Tab);
    }
}

//===========================================================================//
