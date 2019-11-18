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

//===========================================================================//

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, Hash, PartialEq, Serialize,
)]
pub enum HotkeyCode {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Backquote,
    Backslash,
    LeftBracket,
    RightBracket,
    Comma,
    Equals,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    Space,
    Tab,
}

impl HotkeyCode {
    pub fn name(self) -> &'static str {
        match self {
            HotkeyCode::Up => "Up",
            HotkeyCode::Down => "Down",
            HotkeyCode::Left => "Left",
            HotkeyCode::Right => "Right",
            HotkeyCode::A => "A",
            HotkeyCode::B => "B",
            HotkeyCode::C => "C",
            HotkeyCode::D => "D",
            HotkeyCode::E => "E",
            HotkeyCode::F => "F",
            HotkeyCode::G => "G",
            HotkeyCode::H => "H",
            HotkeyCode::I => "I",
            HotkeyCode::J => "J",
            HotkeyCode::K => "K",
            HotkeyCode::L => "L",
            HotkeyCode::M => "M",
            HotkeyCode::N => "N",
            HotkeyCode::O => "O",
            HotkeyCode::P => "P",
            HotkeyCode::Q => "Q",
            HotkeyCode::R => "R",
            HotkeyCode::S => "S",
            HotkeyCode::T => "T",
            HotkeyCode::U => "U",
            HotkeyCode::V => "V",
            HotkeyCode::W => "W",
            HotkeyCode::X => "X",
            HotkeyCode::Y => "Y",
            HotkeyCode::Z => "Z",
            HotkeyCode::Kp0 => "KP0",
            HotkeyCode::Kp1 => "KP1",
            HotkeyCode::Kp2 => "KP2",
            HotkeyCode::Kp3 => "KP3",
            HotkeyCode::Kp4 => "KP4",
            HotkeyCode::Kp5 => "KP5",
            HotkeyCode::Kp6 => "KP6",
            HotkeyCode::Kp7 => "KP7",
            HotkeyCode::Kp8 => "KP8",
            HotkeyCode::Kp9 => "KP9",
            HotkeyCode::Num0 => "0",
            HotkeyCode::Num1 => "1",
            HotkeyCode::Num2 => "2",
            HotkeyCode::Num3 => "3",
            HotkeyCode::Num4 => "4",
            HotkeyCode::Num5 => "5",
            HotkeyCode::Num6 => "6",
            HotkeyCode::Num7 => "7",
            HotkeyCode::Num8 => "8",
            HotkeyCode::Num9 => "9",
            HotkeyCode::Backquote => "`",
            HotkeyCode::Backslash => "\\",
            HotkeyCode::LeftBracket => "[",
            HotkeyCode::RightBracket => "]",
            HotkeyCode::Comma => ",",
            HotkeyCode::Equals => "=",
            HotkeyCode::Minus => "-",
            HotkeyCode::Period => ".",
            HotkeyCode::Quote => "'",
            HotkeyCode::Semicolon => ";",
            HotkeyCode::Slash => "/",
            HotkeyCode::Space => "Space",
            HotkeyCode::Tab => "Tab",
        }
    }
}

//===========================================================================//
