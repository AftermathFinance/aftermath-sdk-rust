use std::ops::Mul;

use num_traits::One;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Balance9(u64);

impl Mul for Balance9 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let (Self(lhs), Self(rhs)) = (self, rhs);
        Self(lhs * rhs / Self::one().into_inner())
    }
}

impl From<f64> for Balance9 {
    fn from(value: f64) -> Self {
        let Self(one) = Self::one();
        let one_f64 = one as f64;
        Self((one_f64 * value) as u64)
    }
}

impl One for Balance9 {
    fn one() -> Self {
        Self(1_000_000_000)
    }
}

impl Balance9 {
    pub const fn into_inner(self) -> u64 {
        self.0
    }

    pub const fn from_inner(int: u64) -> Self {
        Self(int)
    }
}
