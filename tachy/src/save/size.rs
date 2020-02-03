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

use cgmath::Bounded;
use std::str;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WireSize {
    Zero,
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl WireSize {
    pub fn min_for_value(value: u16) -> WireSize {
        if value > 0xff {
            WireSize::Sixteen
        } else if value > 0xf {
            WireSize::Eight
        } else if value > 3 {
            WireSize::Four
        } else if value > 1 {
            WireSize::Two
        } else if value > 0 {
            WireSize::One
        } else {
            WireSize::Zero
        }
    }

    pub fn half(self) -> WireSize {
        match self {
            WireSize::Zero => WireSize::Zero,
            WireSize::One => WireSize::Zero,
            WireSize::Two => WireSize::One,
            WireSize::Four => WireSize::Two,
            WireSize::Eight => WireSize::Four,
            WireSize::Sixteen => WireSize::Eight,
        }
    }

    fn double(self) -> Option<WireSize> {
        match self {
            WireSize::Zero => Some(WireSize::Zero),
            WireSize::One => Some(WireSize::Two),
            WireSize::Two => Some(WireSize::Four),
            WireSize::Four => Some(WireSize::Eight),
            WireSize::Eight => Some(WireSize::Sixteen),
            WireSize::Sixteen => None,
        }
    }

    pub fn num_bits(self) -> u32 {
        match self {
            WireSize::Zero => 0,
            WireSize::One => 1,
            WireSize::Two => 2,
            WireSize::Four => 4,
            WireSize::Eight => 8,
            WireSize::Sixteen => 16,
        }
    }

    pub fn mask(self) -> u32 {
        match self {
            WireSize::Zero => 0x0,
            WireSize::One => 0x1,
            WireSize::Two => 0x3,
            WireSize::Four => 0xf,
            WireSize::Eight => 0xff,
            WireSize::Sixteen => 0xffff,
        }
    }
}

impl Bounded for WireSize {
    fn min_value() -> WireSize {
        WireSize::Zero
    }
    fn max_value() -> WireSize {
        WireSize::Sixteen
    }
}

impl str::FromStr for WireSize {
    type Err = ();

    fn from_str(string: &str) -> Result<WireSize, ()> {
        match string {
            "0" => Ok(WireSize::Zero),
            "1" => Ok(WireSize::One),
            "2" => Ok(WireSize::Two),
            "4" => Ok(WireSize::Four),
            "8" => Ok(WireSize::Eight),
            "16" => Ok(WireSize::Sixteen),
            "Zero" => Ok(WireSize::Zero),
            "One" => Ok(WireSize::One),
            "Two" => Ok(WireSize::Two),
            "Four" => Ok(WireSize::Four),
            "Eight" => Ok(WireSize::Eight),
            "Sixteen" => Ok(WireSize::Sixteen),
            _ => Err(()),
        }
    }
}

//===========================================================================//

#[derive(Clone, Copy, Debug)]
pub struct WireSizeInterval {
    lo: WireSize,
    hi: WireSize,
}

impl WireSizeInterval {
    pub fn new(lo: WireSize, hi: WireSize) -> WireSizeInterval {
        WireSizeInterval { lo, hi }
    }

    pub fn empty() -> WireSizeInterval {
        WireSizeInterval::new(WireSize::max_value(), WireSize::min_value())
    }

    pub fn full() -> WireSizeInterval {
        WireSizeInterval::new(WireSize::min_value(), WireSize::max_value())
    }

    pub fn exactly(size: WireSize) -> WireSizeInterval {
        WireSizeInterval::new(size, size)
    }

    pub fn at_least(size: WireSize) -> WireSizeInterval {
        WireSizeInterval::new(size, WireSize::max_value())
    }

    pub fn is_empty(&self) -> bool {
        self.lo > self.hi
    }

    pub fn is_ambiguous(&self) -> bool {
        self.lo < self.hi
    }

    pub fn lower_bound(&self) -> Option<WireSize> {
        if self.is_empty() {
            None
        } else {
            Some(self.lo)
        }
    }

    pub fn make_at_least(&mut self, size: WireSize) -> bool {
        if !self.is_empty() && self.lo < size {
            self.lo = size;
            true
        } else {
            false
        }
    }

    pub fn make_at_most(&mut self, size: WireSize) -> bool {
        if !self.is_empty() && self.hi > size {
            self.hi = size;
            true
        } else {
            false
        }
    }

    pub fn intersection(&self, other: WireSizeInterval) -> WireSizeInterval {
        WireSizeInterval {
            lo: self.lo.max(other.lo),
            hi: self.hi.min(other.hi),
        }
    }

    pub fn half(&self) -> WireSizeInterval {
        if self.is_empty() {
            WireSizeInterval::empty()
        } else {
            WireSizeInterval {
                lo: self.lo.max(WireSize::Two).half(),
                hi: self.hi.half(),
            }
        }
    }

    pub fn double(&self) -> WireSizeInterval {
        if self.is_empty() {
            WireSizeInterval::empty()
        } else if let Some(lo) = self.lo.double() {
            WireSizeInterval {
                lo,
                hi: self.hi.double().unwrap_or(WireSize::max_value()),
            }
        } else {
            WireSizeInterval::empty()
        }
    }
}

impl PartialEq for WireSizeInterval {
    fn eq(&self, other: &WireSizeInterval) -> bool {
        if self.is_empty() {
            other.is_empty()
        } else if other.is_empty() {
            false
        } else {
            self.lo == other.lo && self.hi == other.hi
        }
    }
}
impl Eq for WireSizeInterval {}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{WireSize, WireSizeInterval};
    use std::u16;

    const ALL_WIRE_SIZES: &[WireSize] = &[
        WireSize::Zero,
        WireSize::One,
        WireSize::Two,
        WireSize::Four,
        WireSize::Eight,
        WireSize::Sixteen,
    ];

    #[test]
    fn min_wire_size() {
        for &size in ALL_WIRE_SIZES {
            let mask = size.mask();
            assert!(mask <= (u16::MAX as u32));
            assert_eq!(size, WireSize::min_for_value(mask as u16));
        }
    }

    #[test]
    fn interval_is_empty() {
        assert!(WireSizeInterval::empty().is_empty());
        assert!(!WireSizeInterval::full().is_empty());
        assert!(!WireSizeInterval::exactly(WireSize::Zero).is_empty());
    }

    #[test]
    fn interval_half() {
        assert_eq!(
            WireSizeInterval::empty().half(),
            WireSizeInterval::empty()
        );
        assert_eq!(
            WireSizeInterval::exactly(WireSize::One).half(),
            WireSizeInterval::empty()
        );
        assert_eq!(
            WireSizeInterval::full().half(),
            WireSizeInterval::new(WireSize::One, WireSize::Eight)
        );
        assert_eq!(
            WireSizeInterval::new(WireSize::Four, WireSize::Sixteen).half(),
            WireSizeInterval::new(WireSize::Two, WireSize::Eight)
        );
    }

    #[test]
    fn interval_double() {
        assert_eq!(
            WireSizeInterval::empty().double(),
            WireSizeInterval::empty()
        );
        assert_eq!(
            WireSizeInterval::exactly(WireSize::Sixteen).double(),
            WireSizeInterval::empty()
        );
        assert_eq!(
            WireSizeInterval::new(WireSize::Two, WireSize::Eight).double(),
            WireSizeInterval::new(WireSize::Four, WireSize::Sixteen)
        );
        assert_eq!(
            WireSizeInterval::new(WireSize::One, WireSize::Sixteen).double(),
            WireSizeInterval::new(WireSize::Two, WireSize::Sixteen)
        );
    }
}

//===========================================================================//
