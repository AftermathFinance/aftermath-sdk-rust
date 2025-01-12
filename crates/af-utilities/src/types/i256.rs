use std::cmp::Ordering;
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

use af_sui_types::u256::U256;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

use super::errors::Error;
use super::onchain;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct I256(U256);

impl std::fmt::Display for I256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_neg() {
            write!(f, "-{}", self.uabs())
        } else {
            write!(f, "{}", self.0)
        }
    }
}

macro_rules! impl_from_uint {
    ($($int:ty)*) => {
        $(
            impl From<$int> for I256 {
                fn from(value: $int) -> Self {
                    Self(U256::from(value))
                }
            }
        )*
    };
}

impl_from_uint!(u8 u16 u32 u64 u128);

macro_rules! impl_from_int {
    ($($int:ty)*) => {
        $(
            impl From<$int> for I256 {
                fn from(value: $int) -> Self {
                    let is_neg = value.is_negative();
                    let abs = Self::from(value.unsigned_abs());
                    match is_neg {
                        true => abs.neg(),
                        false => abs,
                    }
                }
            }
        )*
    };
}

impl_from_int!(i8 i16 i32 i64 i128);

macro_rules! impl_try_into_int {
    ($($bridge:ty => $int:ty),*) => {
        $(
            impl TryFrom<I256> for $int {
                type Error = Error;

                fn try_from(value: I256) -> Result<Self, Self::Error> {
                    let is_neg = value.is_neg();
                    let bridge: $bridge = value.uabs().try_into().map_err(|_| Error::Overflow)?;
                    let self_: Self = bridge.try_into().map_err(|_| Error::Overflow)?;
                    Ok(match is_neg {
                        true => -self_,
                        false => self_,
                    })
                }
            }
        )*
    };
}

impl_try_into_int!(u8 => i8, u16 => i16, u32 => i32, u64 => i64, u128 => i128);

macro_rules! impl_try_into_uint {
    ($($int:ty)*) => {
        $(
            impl TryFrom<I256> for $int {
                type Error = Error;

                fn try_from(value: I256) -> Result<Self, Self::Error> {
                    if value.is_neg() {
                        return Err(Error::Underflow);
                    }
                    value.uabs().try_into().map_err(|_| Error::Overflow)
                }
            }
        )*
    };
}

impl_try_into_uint!(u8 u16 u32 u64 u128);

impl TryFrom<U256> for I256 {
    type Error = Error;

    fn try_from(value: U256) -> Result<Self, Self::Error> {
        if value <= onchain::max_i256() {
            Ok(Self(value))
        } else {
            Err(Error::Overflow)
        }
    }
}

impl TryFrom<I256> for U256 {
    type Error = Error;

    fn try_from(value: I256) -> Result<Self, Self::Error> {
        if value.is_neg() {
            return Err(Error::Underflow);
        }
        Ok(value.0)
    }
}

impl Add for I256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self(x) = self;
        let Self(y) = rhs;
        let greatest_bit = Self::greatest_bit();
        let not_greatest_bit = Self::not_greatest_bit();

        // First, compute sum of x and y except the greatest bit.
        let w = (x & not_greatest_bit) + (y & not_greatest_bit);
        Self(if x ^ y < greatest_bit {
            // The signs of x and y are the same, so the result sign must also be the same
            // for no overflow.
            // assert!(x ^ w < greatest_bit, overflow_error);
            w
        } else {
            // Overflow cannot happen if the signs are different because sum will be closer
            // to 0 than an input.
            w ^ greatest_bit
        })
    }
}

impl Sub for I256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let Self(x) = self;
        let Self(y) = rhs;
        // First, compute wrapping difference of x and y.
        let w = if x >= y {
            x - y
        } else {
            ((y - x) ^ Self::neg_one().0) + U256::one()
        };
        // assert!(x ^ y < GREATEST_BIT || x ^ w < GREATEST_BIT, OVERFLOW_ERROR);
        Self(w)
    }
}

impl Mul for I256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let z = self.uabs() * rhs.uabs();
        let (Self(x), Self(y)) = (self, rhs);
        let greatest_bit = Self::greatest_bit();

        Self(if x ^ y < greatest_bit {
            // assert!(z < greatest_bit, overflow_error);
            z
        } else {
            // assert!(z <= greatest_bit, overflow_error);
            (greatest_bit - z) ^ greatest_bit
        })
    }
}

impl Div for I256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let z = self.uabs() / rhs.uabs();
        let (Self(x), Self(y)) = (self, rhs);
        let greatest_bit = Self::greatest_bit();

        Self(if x ^ y < greatest_bit {
            // assert!(z < greatest_bit, overflow_error);
            z
        } else {
            (greatest_bit - z) ^ greatest_bit
        })
    }
}

impl Rem for I256 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        let is_neg = self.is_neg();
        let abs_rem = Self(self.uabs() % rhs.uabs());
        match is_neg {
            true => abs_rem.neg(),
            false => abs_rem,
        }
    }
}

super::reuse_op_for_assign!(I256 {
    AddAssign add_assign +,
    SubAssign sub_assign -,
    MulAssign mul_assign *,
    DivAssign div_assign /,
    RemAssign rem_assign %,
});

impl PartialOrd for I256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for I256 {
    fn cmp(&self, other: &Self) -> Ordering {
        let Self(x) = self;
        let Self(y) = other;

        if x == y {
            Ordering::Equal
        } else if *x ^ Self::greatest_bit() < *y ^ Self::greatest_bit() {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl Neg for I256 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(((self.0 ^ Self::not_greatest_bit()) + U256::one()) ^ Self::greatest_bit())
    }
}

impl One for I256 {
    fn one() -> Self {
        Self::one()
    }
}

impl Zero for I256 {
    fn zero() -> Self {
        Self(U256::zero())
    }

    fn is_zero(&self) -> bool {
        self.0 == U256::zero()
    }
}

impl I256 {
    fn greatest_bit() -> U256 {
        U256::one() << 255_u8
    }

    fn not_greatest_bit() -> U256 {
        (U256::one() << 255_u8) - U256::one()
    }

    pub const fn neg_one() -> Self {
        Self(U256::max_value())
    }

    pub const fn into_inner(self) -> U256 {
        self.0
    }

    pub const fn from_inner(inner: U256) -> Self {
        Self(inner)
    }

    pub const fn one() -> Self {
        Self(U256::one())
    }

    pub const fn zero() -> Self {
        Self(U256::zero())
    }

    pub fn is_neg(&self) -> bool {
        self.0 >= Self::greatest_bit()
    }

    /// Absolute value of a number.
    /// Can be thought as function from i256 to u256, so doesn't abort.
    pub fn uabs(&self) -> U256 {
        let Self(x) = self;
        if *x >= Self::greatest_bit() {
            (*x ^ Self::neg_one().0) + U256::one()
        } else {
            *x
        }
    }

    pub fn abs(self) -> Self {
        let Self(x) = self;
        let greatest_bit = Self::greatest_bit();
        let not_greatest_bit = Self::not_greatest_bit();
        Self(if x >= greatest_bit {
            ((x ^ not_greatest_bit) + U256::one()) ^ greatest_bit
        } else {
            x
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn from_u128_max_doesnt_overflow() {
        assert!(!I256::from(u128::MAX).is_neg())
    }

    #[test]
    fn from_i128_min_doesnt_underflow() {
        assert!(I256::from(i128::MIN).is_neg())
    }
}
