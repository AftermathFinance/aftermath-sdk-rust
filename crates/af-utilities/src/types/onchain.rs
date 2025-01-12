//! Helpers related to the onchain storage representation of this crate's numerical types.
use af_sui_types::u256::U256;

pub fn greatest_bit() -> U256 {
    U256::one() << 255_u8
}

pub fn not_greatest_bit() -> U256 {
    greatest_bit() - U256::one()
}

/// Maximal possible (positive) value of i256 that equals 2^255 - 1.
pub fn max_i256() -> U256 {
    not_greatest_bit()
}
