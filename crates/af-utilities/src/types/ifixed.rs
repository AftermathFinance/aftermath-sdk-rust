use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
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
    type Err = super::FromStrRadixError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        super::from_str::ifixed_from_str(s)
    }
}

impl TryFrom<f64> for IFixed {
    type Error = super::FromStrRadixError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        super::from_str::ifixed_from_str(&value.to_string())
    }
}

impl TryFrom<IFixed> for f64 {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: IFixed) -> Result<Self, Self::Error> {
        value.to_string().parse()
    }
}

// magnitude / 10^DECIMALS rounded to the nearest f64 (ties to even) with integer arithmetic only.
// The magnitude is scaled by a power of two so that the integer quotient against 10^DECIMALS has
// 54 or 55 bits: the 53 significand bits, one guard bit and, in the 55-bit case, one more bit that
// is folded into the sticky bit together with the division remainder. A 114-bit numerator
// guarantees it, because 2^113 / 10^18 > 2^53 and 2^114 / 10^18 < 2^55. Bits dropped from a
// magnitude wider than 114 bits only feed the sticky bit, which is all that rounding needs.
// Magnitudes that fit in a u128 (every value seen in practice) stay in u128 arithmetic.
fn scaled_to_f64(magnitude: U256) -> f64 {
    const NUMERATOR_BITS: u32 = 114;
    const SIGNIFICAND_BITS: u32 = f64::MANTISSA_DIGITS;
    const EXPONENT_BIAS: u32 = 1023;
    const SCALE: u128 = 10_u128.pow(IFixed::DECIMALS as u32);
    // Normalise to a 114-bit numerator; the dropped low bits only feed the sticky bit.
    let (numerator, mut sticky, bits) = match u128::try_from(magnitude) {
        Ok(0) => return 0.0,
        Ok(narrow) => {
            let bits = u128::BITS - narrow.leading_zeros();
            if bits <= NUMERATOR_BITS {
                (narrow << (NUMERATOR_BITS - bits), false, bits)
            } else {
                let dropped = bits - NUMERATOR_BITS;
                let top = narrow >> dropped;
                (top, top << dropped != narrow, bits)
            }
        }
        Err(_) => {
            let bits = 256 - magnitude.leading_zeros();
            let dropped = (bits - NUMERATOR_BITS) as u8;
            let top = magnitude >> dropped;
            (top.unchecked_as_u128(), top << dropped != magnitude, bits)
        }
    };
    let mut quotient = numerator / SCALE;
    sticky |= !numerator.is_multiple_of(SCALE);
    // magnitude / 10^DECIMALS = quotient * 2^(bits - NUMERATOR_BITS), and the significand is quotient / 2.
    let mut exponent = EXPONENT_BIAS + bits + 1 - NUMERATOR_BITS;
    if quotient >> (SIGNIFICAND_BITS + 1) != 0 {
        sticky |= quotient & 1 != 0;
        quotient >>= 1;
        exponent += 1;
    }
    let mut significand = quotient >> 1;
    if quotient & 1 != 0 && (sticky || significand & 1 != 0) {
        significand += 1;
    }
    significand as f64 * f64::from_bits(u64::from(exponent) << (SIGNIFICAND_BITS - 1))
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
/// let x: IFixed = 50.50.try_into().unwrap();
/// let y: IFixed = 8.125.try_into().unwrap();
/// let remainder = x - (x / y).trunc() * y;
/// assert_eq!(x % y, IFixed::try_from(1.75).unwrap());
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
    pub const DECIMALS: u8 = 18;

    /// The nearest `f64` (ties to even): the value `f64::try_from` gives, computed with integer
    /// arithmetic instead of a decimal string round trip.
    #[must_use]
    pub fn to_f64(self) -> f64 {
        let float = scaled_to_f64(self.0.uabs());
        if self.is_neg() { -float } else { float }
    }

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
        if other.is_neg() { -self } else { self }
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
        fn f64_matches_the_parsed_decimal_string(raw in any::<u128>(), shift in 0..128_u32, neg in any::<bool>()) {
            let value = signed(raw >> shift, neg);
            let float = value.to_f64();
            prop_assert_eq!(float.to_bits(), parsed_decimal_string(value).to_bits());
        }

        #[test]
        fn f64_matches_the_parsed_decimal_string_above_u128(
            high in 1..(1_u128 << 127),
            low in any::<u128>(),
            neg in any::<bool>(),
        ) {
            let value = signed_wide(high, low, neg);
            let float = value.to_f64();
            prop_assert_eq!(float.to_bits(), parsed_decimal_string(value).to_bits());
        }

        #[test]
        fn f64_rounds_ties_like_the_parsed_decimal_string(
            odd in (1_u128 << 52)..(1_u128 << 53),
            power in 0..32_u32,
            offset in 0..3_u128,
            neg in any::<bool>(),
        ) {
            let tie = ((odd * 2 + 1) * 5_u128.pow(18)) << power;
            let value = signed(tie + offset - 1, neg);
            let float = value.to_f64();
            prop_assert_eq!(float.to_bits(), parsed_decimal_string(value).to_bits());
        }

        #[test]
        fn trunc_is_le_to_original(x in i128::MIN..=i128::MAX, y in i128::MIN..=i128::MAX) {
            let x: IFixed = x.into();
            let y: IFixed = y.into();
            let z = x / y;
            assert!(z.trunc().abs() <= z.abs())
        }
    }

    fn signed(magnitude: u128, neg: bool) -> IFixed {
        let inner = I256::from(magnitude);
        IFixed::from_inner(if neg { -inner } else { inner })
    }

    fn signed_wide(high: u128, low: u128, neg: bool) -> IFixed {
        let inner = I256::from_inner((U256::from(high) << 128_u32) | U256::from(low));
        IFixed::from_inner(if neg { -inner } else { inner })
    }

    fn parsed_decimal_string(value: IFixed) -> f64 {
        value.to_string().parse().unwrap()
    }

    #[test]
    fn f64_matches_the_parsed_decimal_string_at_the_edges() {
        let mut edges = vec![
            IFixed::zero(),
            IFixed::from_inner(I256::from_inner(U256::max_value())),
            IFixed::from_inner(I256::from_inner(U256::one() << 255_u8)),
            IFixed::from_inner(I256::from_inner((U256::one() << 255_u8) - U256::one())),
        ];
        for magnitude in [
            1,
            u128::MAX,
            u128::MAX - 1,
            1 << 127,
            (1 << 114) - 1,
            1 << 114,
            (1 << 114) + 1,
            (1 << 53) - 1,
            1 << 53,
            (1 << 53) + 1,
            295_147_905_179_352_809_472_000_000_000_000_000_000 - 1,
            295_147_905_179_352_809_472_000_000_000_000_000_000,
            295_147_905_179_352_809_472_000_000_000_000_000_000 + 1,
            295_147_905_179_352_858_624_000_000_000_000_000_000 - 1,
            295_147_905_179_352_858_624_000_000_000_000_000_000,
            295_147_905_179_352_858_624_000_000_000_000_000_000 + 1,
        ] {
            edges.push(signed(magnitude, false));
            edges.push(signed(magnitude, true));
        }
        for power in 0..39 {
            edges.push(signed(10_u128.pow(power), false));
            edges.push(signed(10_u128.pow(power), true));
        }
        let mut power_of_ten = U256::from(10_u128.pow(38));
        for _ in 39..77 {
            power_of_ten *= U256::from(10_u8);
            edges.push(IFixed::from_inner(I256::from_inner(power_of_ten)));
            edges.push(IFixed::from_inner(-I256::from_inner(power_of_ten)));
        }
        for low in [0, 1, u128::MAX] {
            edges.push(signed_wide(1, low, false));
            edges.push(signed_wide(1, low, true));
            edges.push(signed_wide((1 << 127) - 1, low, false));
            edges.push(signed_wide((1 << 127) - 1, low, true));
        }
        for value in edges {
            let float = value.to_f64();
            assert_eq!(
                float.to_bits(),
                parsed_decimal_string(value).to_bits(),
                "{value}"
            );
        }
    }

    fn ifixed_to_float(s: &str) -> Result<f64, <f64 as FromStr>::Err> {
        IFixed::from_str(s).unwrap().try_into()
    }

    #[test]
    fn try_into_f64() {
        let mut float = ifixed_to_float("0.001").unwrap();
        insta::assert_snapshot!(float, @"0.001");

        float = ifixed_to_float("0.009").unwrap();
        insta::assert_snapshot!(float, @"0.009");

        float = ifixed_to_float("0.003").unwrap();
        insta::assert_snapshot!(float, @"0.003");

        float = ifixed_to_float("0.000000000000000001").unwrap();
        insta::assert_snapshot!(float, @"0.000000000000000001");

        float = ifixed_to_float("2.2238").unwrap();
        insta::assert_snapshot!(float, @"2.2238");

        float = ifixed_to_float("23000000000.0").unwrap();
        insta::assert_snapshot!(float, @"23000000000");

        float = ifixed_to_float("123456700000000000000.0").unwrap();
        insta::assert_snapshot!(float, @"123456700000000000000");

        float = ifixed_to_float("2.3e+10").unwrap();
        insta::assert_snapshot!(float, @"23000000000");

        float = ifixed_to_float("1.234567e+20").unwrap();
        insta::assert_snapshot!(float, @"123456700000000000000");

        float = ifixed_to_float("-2.2238").unwrap();
        insta::assert_snapshot!(float, @"-2.2238");

        float = ifixed_to_float("-1.234567e+20").unwrap();
        insta::assert_snapshot!(float, @"-123456700000000000000");

        float = ifixed_to_float(
            "57896044618658097711785492504343953926634992332820282019728.792003956564819967",
        )
        .unwrap();
        insta::assert_snapshot!(float, @"57896044618658100000000000000000000000000000000000000000000");
        insta::assert_debug_snapshot!(float, @"5.78960446186581e58");

        float = ifixed_to_float(
            "-57896044618658097711785492504343953926634992332820282019728.792003956564819967",
        )
        .unwrap();
        insta::assert_snapshot!(float, @"-57896044618658100000000000000000000000000000000000000000000");
        insta::assert_debug_snapshot!(float, @"-5.78960446186581e58");
    }
}
