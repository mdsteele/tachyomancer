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

use std::fmt;
use std::ops;

//===========================================================================//

const LIMIT: i32 = 1_000_000_000;

//===========================================================================//

/// Represents a fixed-point number from -1.0 to 1.0, inclusive on both sides.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Fixed(i32);

impl Fixed {
    pub const ZERO: Fixed = Fixed(0);
    pub const ONE: Fixed = Fixed(LIMIT);

    pub const fn new(inner: i32) -> Fixed {
        if inner >= LIMIT {
            Fixed(LIMIT)
        } else if inner <= -LIMIT {
            Fixed(-LIMIT)
        } else {
            Fixed(inner)
        }
    }

    pub const fn from_ratio(
        mut numerator: i32,
        mut denominator: i32,
    ) -> Fixed {
        let mut sign: i32 = 1;
        if numerator < 0 {
            sign = -sign;
            numerator = -numerator;
        }
        if denominator < 0 {
            sign = -sign;
            denominator = -denominator;
        }
        let dividend = (numerator as i64) * (LIMIT as i64);
        let divisor = denominator as i64;
        let quotient = dividend / divisor;
        let remainder = dividend % divisor;
        let magnitude = if quotient >= (LIMIT as i64) {
            LIMIT
        } else if remainder * 2 >= divisor {
            (quotient as i32) + 1
        } else {
            quotient as i32
        };
        Fixed(sign * magnitude)
    }

    pub fn from_f64(value: f64) -> Fixed {
        Fixed::new((value.max(-1.0).min(1.0) * (LIMIT as f64)).round() as i32)
    }

    pub fn to_f64(self) -> f64 {
        (self.0 as f64) / (LIMIT as f64)
    }

    pub const fn from_encoded(encoded: u32) -> Fixed {
        Fixed::new(i32::from_ne_bytes(encoded.to_ne_bytes()))
    }

    pub const fn to_encoded(self) -> u32 {
        u32::from_ne_bytes(self.0.to_ne_bytes())
    }

    pub const fn abs(self) -> Fixed {
        Fixed(self.0.abs())
    }
}

impl fmt::Display for Fixed {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.to_f64().fmt(formatter)
    }
}

impl ops::Add for Fixed {
    type Output = Fixed;

    fn add(self, other: Fixed) -> Fixed {
        Fixed::new(self.0.saturating_add(other.0))
    }
}

impl ops::AddAssign for Fixed {
    fn add_assign(&mut self, other: Fixed) {
        *self = *self + other;
    }
}

impl ops::Mul for Fixed {
    type Output = Fixed;

    fn mul(self, other: Fixed) -> Fixed {
        let product = (self.0 as i64) * (other.0 as i64);
        Fixed((product / (LIMIT as i64)) as i32)
    }
}

impl ops::MulAssign for Fixed {
    fn mul_assign(&mut self, other: Fixed) {
        *self = *self * other;
    }
}

impl ops::Neg for Fixed {
    type Output = Fixed;

    fn neg(self) -> Fixed {
        Fixed(-self.0)
    }
}

impl ops::Sub for Fixed {
    type Output = Fixed;

    fn sub(self, other: Fixed) -> Fixed {
        Fixed::new(self.0.saturating_sub(other.0))
    }
}

impl ops::SubAssign for Fixed {
    fn sub_assign(&mut self, other: Fixed) {
        *self = *self - other;
    }
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::{Fixed, LIMIT};

    fn fixed_values() -> Vec<Fixed> {
        vec![
            Fixed::ZERO,
            Fixed::ONE,
            -Fixed::ONE,
            Fixed::from_f64(0.5),
            Fixed::from_f64(-0.5),
            Fixed::from_f64(0.25),
            Fixed::from_f64(-0.25),
            Fixed::from_f64(0.75),
            Fixed::from_f64(-0.75),
            Fixed::from_f64(0.9),
            Fixed::from_f64(std::f64::consts::FRAC_1_SQRT_2),
        ]
    }

    #[test]
    fn round_trip_encoded() {
        for value in fixed_values() {
            assert_eq!(Fixed::from_encoded(value.to_encoded()), value);
        }
    }

    #[test]
    fn round_trip_f64() {
        for value in fixed_values() {
            assert_eq!(Fixed::from_f64(value.to_f64()), value);
        }
    }

    #[test]
    fn fixed_new() {
        assert_eq!(Fixed::new(0), Fixed::ZERO);
        assert_eq!(Fixed::new(LIMIT), Fixed::ONE);
        assert_eq!(Fixed::new(-LIMIT), -Fixed::ONE);
        assert_eq!(Fixed::new(LIMIT / 2), Fixed::from_f64(0.5));
        assert_eq!(Fixed::new(-LIMIT / 4), Fixed::from_f64(-0.25));
        // Clamping:
        assert_eq!(Fixed::new(i32::MAX), Fixed::ONE);
        assert_eq!(Fixed::new(-i32::MAX), -Fixed::ONE);
    }

    #[test]
    fn from_ratio_basic() {
        assert_eq!(Fixed::from_ratio(0, 1), Fixed::ZERO);
        assert_eq!(Fixed::from_ratio(1, 1), Fixed::ONE);
        assert_eq!(Fixed::from_ratio(-1, 1), -Fixed::ONE);
        assert_eq!(Fixed::from_ratio(1, 8), Fixed::from_f64(0.125));
        assert_eq!(Fixed::from_ratio(-3, 4), Fixed::from_f64(-0.75));
        assert_eq!(
            Fixed::from_ratio(1_000_000_000, 2_000_000_000),
            Fixed::from_f64(0.5)
        );
        assert_eq!(Fixed::from_ratio(3, -4), Fixed::from_f64(-0.75));
        assert_eq!(Fixed::from_ratio(-3, -4), Fixed::from_f64(0.75));
    }

    #[test]
    fn from_ratio_rounding() {
        assert_eq!(Fixed::from_ratio(1, 3), Fixed::from_f64(1.0 / 3.0));
        assert_eq!(Fixed::from_ratio(2, 3), Fixed::from_f64(2.0 / 3.0));
        assert_eq!(Fixed::from_ratio(-1, 3), Fixed::from_f64(-1.0 / 3.0));
        assert_eq!(Fixed::from_ratio(-2, 3), Fixed::from_f64(-2.0 / 3.0));
        assert_eq!(Fixed::from_ratio(1, -3), Fixed::from_f64(-1.0 / 3.0));
        assert_eq!(Fixed::from_ratio(2, -3), Fixed::from_f64(-2.0 / 3.0));
    }

    #[test]
    fn from_ratio_clamping() {
        assert_eq!(Fixed::from_ratio(2, 1), Fixed::ONE);
        assert_eq!(Fixed::from_ratio(-2, 1), -Fixed::ONE);
        assert_eq!(Fixed::from_ratio(2_000_000_000, 1), Fixed::ONE);
        assert_eq!(Fixed::from_ratio(-2_000_000_000, 1), -Fixed::ONE);
    }

    #[test]
    fn fixed_display() {
        assert_eq!(format!("{}", Fixed::ZERO), "0".to_string());
        assert_eq!(format!("{}", Fixed::ONE), "1".to_string());
        assert_eq!(format!("{}", -Fixed::ONE), "-1".to_string());
        assert_eq!(format!("{}", Fixed::from_f64(0.125)), "0.125".to_string());
        assert_eq!(format!("{}", Fixed::from_f64(-0.75)), "-0.75".to_string());
    }

    #[test]
    fn fixed_add() {
        assert_eq!(
            Fixed::from_f64(0.25) + Fixed::from_f64(0.5),
            Fixed::from_f64(0.75)
        );
        assert_eq!(
            Fixed::from_f64(0.25) + Fixed::from_f64(-0.5),
            Fixed::from_f64(-0.25)
        );
        // Saturation:
        assert_eq!(
            Fixed::from_f64(0.75) + Fixed::from_f64(0.5),
            Fixed::from_f64(1.0)
        );
        assert_eq!(
            Fixed::from_f64(-0.5) + Fixed::from_f64(-0.75),
            Fixed::from_f64(-1.0)
        );
    }

    #[test]
    fn fixed_add_assign() {
        let mut value = Fixed::from_f64(0.25);
        value += Fixed::from_f64(0.5);
        assert_eq!(value, Fixed::from_f64(0.75));
    }

    #[test]
    fn fixed_mul() {
        assert_eq!(
            Fixed::from_f64(0.25) * Fixed::from_f64(0.5),
            Fixed::from_f64(0.125)
        );
        assert_eq!(
            Fixed::from_f64(0.75) * Fixed::from_f64(-1.0),
            Fixed::from_f64(-0.75)
        );
        assert_eq!(
            Fixed::from_f64(-0.5) * Fixed::from_f64(0.0),
            Fixed::from_f64(0.0)
        );
        assert_eq!(
            Fixed::from_f64(1.0) * Fixed::from_f64(1.0),
            Fixed::from_f64(1.0)
        );
        assert_eq!(
            Fixed::from_f64(-1.0) * Fixed::from_f64(-1.0),
            Fixed::from_f64(1.0)
        );
        assert_eq!(
            Fixed::from_f64(-1.0) * Fixed::from_f64(1.0),
            Fixed::from_f64(-1.0)
        );
    }

    #[test]
    fn fixed_mul_assign() {
        let mut value = Fixed::from_f64(0.25);
        value *= Fixed::from_f64(0.5);
        assert_eq!(value, Fixed::from_f64(0.125));
    }

    #[test]
    fn fixed_neg() {
        assert_eq!(-Fixed::from_f64(0.0), Fixed::from_f64(0.0));
        assert_eq!(-Fixed::from_f64(0.25), Fixed::from_f64(-0.25));
        assert_eq!(-Fixed::from_f64(-1.0), Fixed::from_f64(1.0));
    }

    #[test]
    fn fixed_sub() {
        assert_eq!(
            Fixed::from_f64(0.25) - Fixed::from_f64(0.5),
            Fixed::from_f64(-0.25)
        );
        assert_eq!(
            Fixed::from_f64(1.0) - Fixed::from_f64(0.5),
            Fixed::from_f64(0.5)
        );
        // Saturation:
        assert_eq!(
            Fixed::from_f64(0.75) - Fixed::from_f64(-0.5),
            Fixed::from_f64(1.0)
        );
        assert_eq!(
            Fixed::from_f64(-0.5) - Fixed::from_f64(0.75),
            Fixed::from_f64(-1.0)
        );
    }

    #[test]
    fn fixed_sub_assign() {
        let mut value = Fixed::from_f64(0.25);
        value -= Fixed::from_f64(0.5);
        assert_eq!(value, Fixed::from_f64(-0.25));
    }

    #[test]
    fn fixed_abs() {
        assert_eq!(Fixed::ZERO.abs(), Fixed::ZERO);
        assert_eq!(Fixed::ONE.abs(), Fixed::ONE);
        assert_eq!((-Fixed::ONE).abs(), Fixed::ONE);
        assert_eq!(Fixed::from_f64(-0.375).abs(), Fixed::from_f64(0.375));
    }
}

//===========================================================================//
