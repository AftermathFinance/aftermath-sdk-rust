use std::num::ParseFloatError;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};
use std::str::FromStr;

use af_sui_types::u256::U256;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::IFixed;
use crate::types::errors::Error;

const ONE_FIXED_F64: f64 = 1_000_000_000_000_000_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Fixed(U256);

// Inspired by:
// https://docs.rs/fixed-point/latest/src/fixed_point/lib.rs.html#142-177
impl std::fmt::Display for Fixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut decimal = self.decimal();
        if Self::DECIMALS == 0 || decimal == U256::zero() {
            return write!(f, "{}.0", self.integer());
        }
        let mut length = Self::DECIMALS;
        while decimal % 10u8.into() == U256::zero() {
            decimal /= 10u8.into();
            length -= 1;
        }
        let integer = self.integer();
        write!(
            f,
            "{}.{:0length$}",
            integer,
            decimal,
            length = length as usize
        )
    }
}

#[derive(Debug, Clone, Error)]
pub enum FromStrErr {
    #[error("Handling af-utilities types")]
    AfUtils(#[from] Error),
    #[error("Parsing f64")]
    Fromf64(#[from] ParseFloatError),
}

impl FromStr for Fixed {
    type Err = FromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let signed: IFixed = s.parse()?;
        Ok(signed.try_into()?)
    }
}

impl Add for Fixed {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Fixed {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for Fixed {
    type Output = Self;

    /// This is the '`mul_down`' equivalent
    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0) / Self::one().0)
    }
}

impl Div for Fixed {
    type Output = Self;

    /// This is the '`div_down`' equivalent
    fn div(self, rhs: Self) -> Self::Output {
        Self((self.0 * Self::one().0) / rhs.0)
    }
}

/// The remainder from the division of two fixed, inspired by the primitive floats implementations.
///
/// The remainder has the same sign as the dividend and is computed as:
/// `x - (x / y).trunc() * y`.
impl Rem for Fixed {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        self - (self / rhs).trunc() * rhs
    }
}

super::reuse_op_for_assign!(Fixed {
    AddAssign add_assign +,
    SubAssign sub_assign -,
    MulAssign mul_assign *,
    DivAssign div_assign /,
    RemAssign rem_assign %,
});

impl One for Fixed {
    fn one() -> Self {
        Self::one()
    }
}

impl Zero for Fixed {
    fn zero() -> Self {
        Self::zero()
    }

    fn is_zero(&self) -> bool {
        self.0 == U256::zero()
    }
}

macro_rules! impl_from_integer {
    ($($int:ty)*) => {
        $(
            impl From<$int> for Fixed {
                fn from(value: $int) -> Self {
                    Self(Self::one().0 * U256::from(value))
                }
            }
        )*
    };
}

impl_from_integer!(u8 u16 u32 u64 u128);

macro_rules! impl_try_into_integer {
    ($($int:ty)*) => {
        $(
            impl TryFrom<Fixed> for $int {
                type Error = Error;

                fn try_from(value: Fixed) -> Result<Self, Self::Error> {
                    value.integer().try_into().map_err(|_| Error::Overflow)
                }
            }
        )*
    };
}

impl_try_into_integer!(u8 u16 u32 u64 u128);

impl From<f64> for Fixed {
    fn from(value: f64) -> Self {
        Self(U256::from_f64_lossy(value * ONE_FIXED_F64))
    }
}

impl From<Fixed> for f64 {
    fn from(value: Fixed) -> Self {
        value.0.to_f64_lossy() / ONE_FIXED_F64
    }
}

impl Fixed {
    const DECIMALS: u8 = 18;

    /// Round this number up to an integer
    pub fn ceil(self) -> Self {
        if self.decimal() > U256::zero() {
            self.trunc() + Self::one()
        } else {
            self
        }
    }

    /// Truncate the decimal part of this number.
    pub fn trunc(self) -> Self {
        Self(self.integer() * Self::one().0)
    }

    fn decimal(&self) -> U256 {
        self.0 % Self::one().0
    }

    fn integer(&self) -> U256 {
        self.0 / Self::one().0
    }

    pub fn one() -> Self {
        Self(1_000_000_000_000_000_000_u64.into())
    }

    pub const fn zero() -> Self {
        Self(U256::zero())
    }

    pub const fn into_inner(self) -> U256 {
        self.0
    }

    pub const fn from_inner(value: U256) -> Self {
        Self(value)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn from_max_u128_doesnt_panic() {
        let _: Fixed = u128::MAX.into();
    }

    proptest! {
        #[test]
        fn uint_conversions_are_preserving(x in 0..=u128::MAX) {
            let x_: u128 = Fixed::from(x).try_into().unwrap();
            assert_eq!(x, x_)
        }

        #[test]
        fn can_recover_from_decimal_and_integer(x in 0..=u128::MAX, y in 1..=u128::MAX) {
            let x: Fixed = x.into();
            let y: Fixed = y.into();
            let z = x / y;
            assert_eq!(z, Fixed::from_inner(z.integer() * Fixed::one().into_inner() + z.decimal()))
        }

        #[test]
        fn trunc_is_le_to_original(x in 0..=u128::MAX, y in 1..=u128::MAX) {
            let x: Fixed = x.into();
            let y: Fixed = y.into();
            let z = x / y;
            assert!(z.trunc() <= z)
        }

        #[test]
        fn ceil_is_ge_to_original(x in 0..=u128::MAX, y in 1..=u128::MAX) {
            let x: Fixed = x.into();
            let y: Fixed = y.into();
            let z = x / y;
            assert!(z.ceil() >= z)
        }
    }
}
