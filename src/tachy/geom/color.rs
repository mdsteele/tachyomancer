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

#[derive(Clone, Copy, PartialEq)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(dead_code)]
impl Color4 {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color4 {
        Color4 { r, g, b, a }
    }

    pub const fn with_alpha(&self, alpha: f32) -> Color4 {
        Color4 {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }

    pub const BLACK: Color4 = Color4::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color4 = Color4::new(1.0, 1.0, 1.0, 1.0);

    pub const CYAN1: Color4 = Color4::new(0.106, 0.286, 0.325, 1.0);
    pub const CYAN2: Color4 = Color4::new(0.141, 0.384, 0.447, 1.0);
    pub const CYAN3: Color4 = Color4::new(0.318, 0.851, 0.976, 1.0);
    pub const CYAN4: Color4 = Color4::new(0.396, 0.922, 0.984, 1.0);
    pub const CYAN5: Color4 = Color4::new(0.592, 0.949, 0.988, 1.0);

    pub const PURPLE0: Color4 = Color4::new(0.118, 0.039, 0.180, 1.0);
    pub const PURPLE1: Color4 = Color4::new(0.235, 0.078, 0.361, 1.0);
    pub const PURPLE2: Color4 = Color4::new(0.318, 0.106, 0.490, 1.0);
    pub const PURPLE3: Color4 = Color4::new(0.643, 0.216, 0.988, 1.0);
    pub const PURPLE4: Color4 = Color4::new(0.761, 0.341, 0.973, 1.0);
    pub const PURPLE5: Color4 = Color4::new(0.855, 0.569, 0.996, 1.0);

    pub const ORANGE1: Color4 = Color4::new(0.533, 0.224, 0.086, 1.0);
    pub const ORANGE2: Color4 = Color4::new(0.761, 0.310, 0.122, 1.0);
    pub const ORANGE3: Color4 = Color4::new(0.859, 0.361, 0.137, 1.0);
    pub const ORANGE4: Color4 = Color4::new(0.851, 0.576, 0.325, 1.0);
    pub const ORANGE5: Color4 = Color4::new(0.902, 0.714, 0.533, 1.0);

    pub const YELLOW3: Color4 = Color4::new(0.957, 0.976, 0.153, 1.0);
}

impl From<(f32, f32, f32)> for Color4 {
    fn from((r, g, b): (f32, f32, f32)) -> Color4 {
        Color4 { r, g, b, a: 1.0 }
    }
}

//===========================================================================//
