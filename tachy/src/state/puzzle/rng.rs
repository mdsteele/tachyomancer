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

use std::u32;

//===========================================================================//

/// A simple, not-very-good pseudo-random number generator.  We use this
/// instead of the rand crate because (1) we don't need a very good RNG, but
/// (2) we do need the random sequence to be deterministic and guaranteed
/// stable across compiles and crate versions.
pub struct SimpleRng {
    z: u32,
    w: u32,
}

impl SimpleRng {
    pub fn new(seed: u64) -> SimpleRng {
        SimpleRng {
            z: 0x159a55e5 ^ ((seed & 0xffffffff) as u32),
            w: 0x1f123bb5 ^ ((seed >> 32) as u32),
        }
    }

    pub fn rand_u32(&mut self) -> u32 {
        // This RNG is based on the MWC algorithm from George Marsaglia's post
        // to sci.stat.math on 12 Jan 1999, which can be found here:
        //   https://groups.google.com/forum/#!topic/sci.stat.math/5yb0jwf1stw
        self.z = 36969 * (self.z & 0xffff) + (self.z >> 16);
        self.w = 18000 * (self.w & 0xffff) + (self.w >> 16);
        (self.z << 16) | (self.w & 0xffff)
    }

    pub fn rand_u4(&mut self) -> u32 {
        self.rand_u32() & 0xf
    }

    /// Returns a random value between lower and upper, inclusive.
    #[allow(dead_code)]
    pub fn rand_int(&mut self, lower: u32, upper: u32) -> u32 {
        assert!(lower <= upper);
        if lower == u32::MIN && upper == u32::MAX {
            return self.rand_u32();
        }
        let range_size = upper - lower + 1;
        if range_size.is_power_of_two() {
            return lower + (self.rand_u32() & (range_size - 1));
        }
        let limit = u32::MAX - (u32::MAX % range_size);
        debug_assert_eq!(0, limit % range_size);
        let mut value = limit;
        while value >= limit {
            value = self.rand_u32();
        }
        return lower + (value % range_size);
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::SimpleRng;
    use std::u32;

    #[test]
    fn golden_sequence() {
        let mut rng = SimpleRng::new(1234567890987654321);
        assert_eq!(0xf86a5880, rng.rand_u32());
        assert_eq!(0xacd7b3e1, rng.rand_u32());
        assert_eq!(0x6050d49e, rng.rand_u32());
        assert_eq!(0xe250d6c7, rng.rand_u32());
        assert_eq!(0x0924c295, rng.rand_u32());
        assert_eq!(0x7f6ec78d, rng.rand_u32());
    }

    #[test]
    fn rand_int_same_as_rand_u32() {
        let seed: u64 = 0xb6d11e8ba7cb5dee;
        let mut rng1 = SimpleRng::new(seed);
        let seq1: Vec<u32> = (0..10).map(|_| rng1.rand_u32()).collect();
        let mut rng2 = SimpleRng::new(seed);
        let seq2: Vec<u32> =
            (0..10).map(|_| rng2.rand_int(u32::MIN, u32::MAX)).collect();
        assert_eq!(seq1, seq2);
    }

    #[test]
    fn rand_int_same_as_rand_u4() {
        let seed: u64 = 0xbdd0cb3c08c5e7f8;
        let mut rng1 = SimpleRng::new(seed);
        let seq1: Vec<u32> = (0..10).map(|_| rng1.rand_u4()).collect();
        let mut rng2 = SimpleRng::new(seed);
        let seq2: Vec<u32> = (0..10).map(|_| rng2.rand_int(0, 15)).collect();
        assert_eq!(seq1, seq2);
    }

    #[test]
    fn rand_int_stays_in_range() {
        let seed: u64 = 0xf324772645c911e0;
        let mut rng = SimpleRng::new(seed);
        let mut hits = [false; 13];
        // Generate a bunch of random values; they should all be within the
        // specified range.
        for _ in 0..100 {
            let value = rng.rand_int(7, 19);
            assert!(value >= 7 && value <= 19);
            hits[(value - 7) as usize] = true;
        }
        // After so many random values, we ought to have hit every number in
        // the range at least once (note that since SimpleRng is deterministic,
        // this test is not flaky).
        assert_eq!(hits, [true; 13]);
    }
}

//===========================================================================//
