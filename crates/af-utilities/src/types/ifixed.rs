use std::ops::{
    Add,
    AddAssign,
    Div,
    DivAssign,
    Mul,
    MulAssign,
    Neg,
    Rem,
    RemAssign,
    Sub,
    SubAssign,
};
use std::str::FromStr;

use af_sui_types::u256::U256;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

use super::errors::Error;
use super::i256::I256;
use super::{Balance9, Fixed};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct IFixed(I256);

impl std::fmt::Debug for IFixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.debug_tuple("IFixed").field(&self.0).finish()
        } else {
            <Self as std::fmt::Display>::fmt(self, f)
        }
    }
}

// Inspired by:
// https://docs.rs/fixed-point/latest/src/fixed_point/lib.rs.html#142-177
impl std::fmt::Display for IFixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut decimal = self.udecimal();
        if Self::DECIMALS == 0 || decimal == U256::zero() {
            return write!(f, "{}.0", self.integer());
        }
        let mut length = Self::DECIMALS;
        while decimal % 10u8.into() == U256::zero() {
            decimal /= 10u8.into();
            length -= 1;
        }
        let integer = self.integer();
        if integer == I256::zero() && self.is_neg() {
            write!(f, "-0.{:0length$}", decimal, length = length as usize)
        } else {
            write!(
                f,
                "{}.{:0length$}",
                integer,
                decimal,
                length = length as usize
            )
        }
    }
}

impl Default for IFixed {
    fn default() -> Self {
        Self::zero()
    }
}

impl TryFrom<Fixed> for IFixed {
    type Error = Error;

    fn try_from(value: Fixed) -> Result<Self, Self::Error> {
        Ok(Self(value.into_inner().try_into()?))
    }
}

impl TryFrom<IFixed> for Fixed {
    type Error = Error;

    fn try_from(value: IFixed) -> Result<Self, Self::Error> {
        Ok(Self::from_inner(value.0.try_into()?))
    }
}

impl FromStr for IFixed {
    type Err = <f64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let float: f64 = s.parse()?;
        Ok(float.into())
    }
}

impl From<f64> for IFixed {
    fn from(value: f64) -> Self {
        let max_i256 = U256::max_value() >> 1;
        let unsigned_inner = Fixed::from(value.abs()).into_inner().min(max_i256);
        let unsigned_inner = I256::from_inner(unsigned_inner);
        Self(if value.is_sign_negative() {
            unsigned_inner.neg()
        } else {
            unsigned_inner
        })
    }
}

impl From<IFixed> for f64 {
    fn from(value: IFixed) -> Self {
        let result_abs = Self::from(value.uabs());
        if value.is_neg() {
            -result_abs
        } else {
            result_abs
        }
    }
}

impl From<Balance9> for IFixed {
    fn from(value: Balance9) -> Self {
        let balance_u256: U256 = value.into_inner().into();
        let scaling_factor: U256 = 1_000_000_000_u64.into();
        Self(I256::from_inner(balance_u256 * scaling_factor))
    }
}

impl TryFrom<IFixed> for Balance9 {
    type Error = Error;

    fn try_from(value: IFixed) -> Result<Self, Self::Error> {
        if value.is_neg() {
            return Err(Error::Underflow);
        }

        let scaling_factor: U256 = 1_000_000_000_u64.into();
        let inner = (value.into_inner().into_inner() / scaling_factor)
            .try_into()
            .map_err(|_| Error::Overflow)?;
        Ok(Self::from_inner(inner))
    }
}

macro_rules! impl_from_integer {
    ($($int:ty)*) => {
        $(
            impl From<$int> for IFixed {
                fn from(value: $int) -> Self {
                    Self(Self::one().0 * I256::from(value))
                }
            }
        )*
    };
}

impl_from_integer!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

macro_rules! impl_try_into_integer {
    ($($int:ty)*) => {
        $(
            impl TryFrom<IFixed> for $int {
                type Error = Error;

                fn try_from(value: IFixed) -> Result<Self, Self::Error> {
                    value.integer().try_into()
                }
            }
        )*
    };
}

impl_try_into_integer!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

impl Add for IFixed {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for IFixed {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for IFixed {
    type Output = Self;

    /// This is the '`mul_down`' equivalent
    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0) / Self::one().0)
    }
}

impl Div for IFixed {
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
///
/// # Examples
/// ```
/// # use af_utilities::types::IFixed;
/// let x: IFixed = 50.50.into();
/// let y: IFixed = 8.125.into();
/// let remainder = x - (x / y).trunc() * y;
/// assert_eq!(x % y, IFixed::from(1.75));
/// ```
impl Rem for IFixed {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        self - (self / rhs).trunc() * rhs
    }
}

super::reuse_op_for_assign!(IFixed {
    AddAssign add_assign +,
    SubAssign sub_assign -,
    MulAssign mul_assign *,
    DivAssign div_assign /,
    RemAssign rem_assign %,
});

impl Neg for IFixed {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl One for IFixed {
    fn one() -> Self {
        Self::one()
    }
}

impl Zero for IFixed {
    fn zero() -> Self {
        Self::zero()
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

// TODO: add IFixed::from_pyth_repr(i64, i32)
// https://docs.rs/pyth-sdk/0.8.0/pyth_sdk/struct.Price.html
impl IFixed {
    const DECIMALS: u8 = 18;

    /// Create an `u64` from a `IFixed` applying the specified scaling factor.
    pub fn try_into_balance_with_scaling(self, scaling_factor: U256) -> Result<u64, Error> {
        if self.is_neg() {
            return Err(Error::Underflow);
        }

        let inner = (self.into_inner().into_inner() / scaling_factor)
            .try_into()
            .map_err(|_| Error::Overflow)?;
        Ok(inner)
    }

    /// Create an `IFixed` from a `u64` applying the specified scaling factor.
    pub fn from_balance_with_scaling(balance: u64, scaling_factor: U256) -> Self {
        let balance_u256: U256 = balance.into();
        Self(I256::from_inner(balance_u256 * scaling_factor))
    }

    /// Create an `IFixed` from a `str` containing the
    /// ifixed internal representation.
    ///
    /// Example: the `str` containing "134850000000000000000" is
    /// converted to the value 134.85 in IFixed.
    pub fn from_raw_str(ifixed_string: &str) -> Result<Self, Error> {
        let Ok(u256_val) = ifixed_string.parse::<U256>() else {
            return Err(Error::ParseStringToU256(ifixed_string.to_string()));
        };
        Ok(Self::from_inner(I256::from_inner(u256_val)))
    }

    /// Create an `IFixed` using its internal representation
    pub const fn from_inner(inner: I256) -> Self {
        Self(inner)
    }

    /// Truncate the decimal part of this number.
    pub fn trunc(self) -> Self {
        Self(self.integer() * Self::one().0)
    }

    /// The integer part of this number.
    pub fn integer(self) -> I256 {
        self.0 / Self::one().0
    }

    /// The decimal part of this number.
    pub fn decimal(self) -> I256 {
        self.0 % Self::one().0
    }

    pub fn round_to_decimals(self, decimals: u32, round_up: bool) -> Self {
        let scaling_factor: I256 = 10_u64.pow(decimals).into();
        let rounding: I256 = 1_u64.into();
        let partial = self.into_inner() / scaling_factor;
        if round_up {
            Self((partial + rounding) * scaling_factor)
        } else {
            Self((partial - rounding) * scaling_factor)
        }
    }

    /// The unsigned decimal part of this number.
    pub fn udecimal(self) -> U256 {
        self.0.uabs() % Self::one().0.uabs()
    }

    pub const fn into_inner(self) -> I256 {
        self.0
    }

    pub fn is_neg(&self) -> bool {
        self.0.is_neg()
    }

    pub fn one() -> Self {
        Self(1_000_000_000_000_000_000_u64.into())
    }

    pub const fn zero() -> Self {
        Self(I256::zero())
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    pub fn uabs(self) -> Fixed {
        Fixed::from_inner(self.0.uabs())
    }

    pub fn copy_sign(self, other: &Self) -> Self {
        if other.is_neg() {
            -self
        } else {
            self
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn from_u128_max_doesnt_overflow() {
        assert!(!IFixed::from(u128::MAX).is_neg())
    }

    #[test]
    fn from_i128_min_doesnt_underflow() {
        assert!(IFixed::from(i128::MIN).is_neg())
    }

    proptest! {
        #[test]
        fn int_conversions_are_preserving(x in i128::MIN..=i128::MAX) {
            let x_: i128 = IFixed::from(x).try_into().unwrap();
            assert_eq!(x, x_)
        }

        #[test]
        fn uint_conversions_are_preserving(x in 0..=u128::MAX) {
            let x_: u128 = IFixed::from(x).try_into().unwrap();
            assert_eq!(x, x_)
        }

        #[test]
        fn trunc_is_le_to_original(x in i128::MIN..=i128::MAX, y in i128::MIN..=i128::MAX) {
            let x: IFixed = x.into();
            let y: IFixed = y.into();
            let z = x / y;
            assert!(z.trunc().abs() <= z.abs())
        }
    }
}
