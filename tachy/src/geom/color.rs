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

/// Represents an RGB color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color3 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color3 {
    pub const fn new(r: f32, g: f32, b: f32) -> Color3 {
        Color3 { r, g, b }
    }

    pub const fn with_alpha(&self, alpha: f32) -> Color4 {
        Color4::new(self.r, self.g, self.b, alpha)
    }

    pub fn mix(&self, other: Color3, param: f32) -> Color3 {
        Color3 {
            r: self.r + param * (other.r - self.r),
            g: self.g + param * (other.g - self.g),
            b: self.b + param * (other.b - self.b),
        }
    }

    pub const BLACK: Color3 = Color3::new(0.0, 0.0, 0.0);
    pub const WHITE: Color3 = Color3::new(1.0, 1.0, 1.0);

    pub const CYAN0: Color3 = Color3::new(0.053, 0.143, 0.163); // #0e242a
    pub const CYAN1: Color3 = Color3::new(0.106, 0.286, 0.325); // #1b4953
    pub const CYAN2: Color3 = Color3::new(0.141, 0.384, 0.447); // #246272
    pub const CYAN3: Color3 = Color3::new(0.318, 0.851, 0.976); // #51d9f9
    pub const CYAN4: Color3 = Color3::new(0.396, 0.922, 0.984); // #65ebfb
    pub const CYAN5: Color3 = Color3::new(0.592, 0.949, 0.988); // #97f2fc

    pub const GREEN2: Color3 = Color3::new(0.110, 0.451, 0.141); // #1c7324
    pub const GREEN3: Color3 = Color3::new(0.235, 0.980, 0.310); // #3cfa4f
    pub const GREEN4: Color3 = Color3::new(0.439, 0.980, 0.412); // #70fa69

    pub const ORANGE0: Color3 = Color3::new(0.267, 0.112, 0.043); // #441d0b
    pub const ORANGE1: Color3 = Color3::new(0.533, 0.224, 0.086); // #883916
    pub const ORANGE2: Color3 = Color3::new(0.761, 0.310, 0.122); // #c24f1f
    pub const ORANGE3: Color3 = Color3::new(0.859, 0.361, 0.137); // #db5c23
    pub const ORANGE4: Color3 = Color3::new(0.851, 0.576, 0.325); // #d99353
    pub const ORANGE5: Color3 = Color3::new(0.902, 0.714, 0.533); // #e6b688

    pub const PURPLE0: Color3 = Color3::new(0.118, 0.039, 0.180); // #1e0a2e
    pub const PURPLE1: Color3 = Color3::new(0.235, 0.078, 0.361); // #3c145c
    pub const PURPLE2: Color3 = Color3::new(0.318, 0.106, 0.490); // #511b7d
    pub const PURPLE3: Color3 = Color3::new(0.643, 0.216, 0.988); // #a437fc
    pub const PURPLE4: Color3 = Color3::new(0.761, 0.341, 0.973); // #c257f8
    pub const PURPLE5: Color3 = Color3::new(0.855, 0.569, 0.996); // #da91fe

    pub const RED2: Color3 = Color3::new(0.451, 0.024, 0.059); // #73060f
    pub const RED3: Color3 = Color3::new(0.980, 0.047, 0.125); // #fa0c20

    pub const YELLOW0: Color3 = Color3::new(0.318, 0.326, 0.057); // #51530f
    pub const YELLOW1: Color3 = Color3::new(0.635, 0.651, 0.114); // #a2a61d
    pub const YELLOW2: Color3 = Color3::new(0.824, 0.847, 0.161); // #d2d829
    pub const YELLOW3: Color3 = Color3::new(0.957, 0.976, 0.153); // #f4f927
    pub const YELLOW4: Color3 = Color3::new(0.980, 0.973, 0.439); // #faf870
    pub const YELLOW5: Color3 = Color3::new(0.988, 0.980, 0.643); // #fcfaa4
}

//===========================================================================//

/// Represents a non-premultiplied RGBA color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4 {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color4 {
        Color4 { r, g, b, a }
    }

    pub fn mix(&self, other: Color4, param: f32) -> Color4 {
        Color4 {
            r: self.r + param * (other.r - self.r),
            g: self.g + param * (other.g - self.g),
            b: self.b + param * (other.b - self.b),
            a: self.a + param * (other.a - self.a),
        }
    }

    pub const TRANSPARENT: Color4 = Color4::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color4 = Color3::BLACK.with_alpha(1.0);
    pub const WHITE: Color4 = Color3::WHITE.with_alpha(1.0);

    pub const CYAN0: Color4 = Color3::CYAN0.with_alpha(1.0);
    pub const CYAN0_TRANSLUCENT: Color4 = Color3::CYAN0.with_alpha(0.8);
    pub const CYAN1: Color4 = Color3::CYAN1.with_alpha(1.0);
    pub const CYAN2: Color4 = Color3::CYAN2.with_alpha(1.0);
    pub const CYAN3: Color4 = Color3::CYAN3.with_alpha(1.0);
    pub const CYAN3_TRANSLUCENT: Color4 = Color3::CYAN3.with_alpha(0.8);
    pub const CYAN4: Color4 = Color3::CYAN4.with_alpha(1.0);
    pub const CYAN5: Color4 = Color3::CYAN5.with_alpha(1.0);

    pub const GREEN2: Color4 = Color3::GREEN2.with_alpha(1.0);
    pub const GREEN3: Color4 = Color3::GREEN3.with_alpha(1.0);
    pub const GREEN4: Color4 = Color3::GREEN4.with_alpha(1.0);

    pub const ORANGE0: Color4 = Color3::ORANGE0.with_alpha(1.0);
    pub const ORANGE0_TRANSLUCENT: Color4 = Color3::ORANGE0.with_alpha(0.8);
    pub const ORANGE1: Color4 = Color3::ORANGE1.with_alpha(1.0);
    pub const ORANGE2: Color4 = Color3::ORANGE2.with_alpha(1.0);
    pub const ORANGE3: Color4 = Color3::ORANGE3.with_alpha(1.0);
    pub const ORANGE4: Color4 = Color3::ORANGE4.with_alpha(1.0);
    pub const ORANGE5: Color4 = Color3::ORANGE5.with_alpha(1.0);

    pub const PURPLE0: Color4 = Color3::PURPLE0.with_alpha(1.0);
    pub const PURPLE0_TRANSLUCENT: Color4 = Color3::PURPLE0.with_alpha(0.8);
    pub const PURPLE1: Color4 = Color3::PURPLE1.with_alpha(1.0);
    pub const PURPLE2: Color4 = Color3::PURPLE2.with_alpha(1.0);
    pub const PURPLE3: Color4 = Color3::PURPLE3.with_alpha(1.0);
    pub const PURPLE3_TRANSLUCENT: Color4 = Color3::PURPLE3.with_alpha(0.8);
    pub const PURPLE4: Color4 = Color3::PURPLE4.with_alpha(1.0);
    pub const PURPLE5: Color4 = Color3::PURPLE5.with_alpha(1.0);

    pub const RED2: Color4 = Color3::RED2.with_alpha(1.0);
    pub const RED3: Color4 = Color3::RED3.with_alpha(1.0);

    pub const YELLOW0: Color4 = Color3::YELLOW0.with_alpha(1.0);
    pub const YELLOW0_TRANSLUCENT: Color4 = Color3::YELLOW0.with_alpha(0.8);
    pub const YELLOW1: Color4 = Color3::YELLOW1.with_alpha(1.0);
    pub const YELLOW2: Color4 = Color3::YELLOW2.with_alpha(1.0);
    pub const YELLOW3: Color4 = Color3::YELLOW3.with_alpha(1.0);
    pub const YELLOW3_TRANSLUCENT: Color4 = Color3::YELLOW3.with_alpha(0.8);
    pub const YELLOW4: Color4 = Color3::YELLOW4.with_alpha(1.0);
    pub const YELLOW5: Color4 = Color3::YELLOW5.with_alpha(1.0);
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Color3, Color4};

    #[test]
    fn color3_with_alpha() {
        assert_eq!(
            Color3::new(0.1, 0.2, 0.3).with_alpha(0.4),
            Color4::new(0.1, 0.2, 0.3, 0.4)
        );
    }

    #[test]
    fn color3_mix() {
        let color1 = Color3::new(0.25, 0.75, 1.0);
        let color2 = Color3::new(0.75, 0.0, 0.25);
        assert_eq!(color1.mix(color2, 0.0), color1);
        assert_eq!(
            color1.mix(color2, 0.25),
            Color3::new(0.375, 0.5625, 0.8125)
        );
        assert_eq!(color1.mix(color2, 0.5), Color3::new(0.5, 0.375, 0.625));
        assert_eq!(color1.mix(color2, 1.0), color2);
    }

    #[test]
    fn color4_mix() {
        let color1 = Color4::new(0.25, 0.75, 1.0, 0.5);
        let color2 = Color4::new(0.75, 0.0, 0.25, 1.0);
        assert_eq!(color1.mix(color2, 0.0), color1);
        assert_eq!(
            color1.mix(color2, 0.25),
            Color4::new(0.375, 0.5625, 0.8125, 0.625)
        );
        assert_eq!(
            color1.mix(color2, 0.5),
            Color4::new(0.5, 0.375, 0.625, 0.75)
        );
        assert_eq!(color1.mix(color2, 1.0), color2);
    }
}

//===========================================================================//
