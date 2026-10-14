//! Keccak-256, the hash that Aftermath and Pyth Lazer signatures are made over and Stork asset
//! IDs are encoded with.

/// Keccak-256 of `bytes`, as used by Ethereum, Sui's `ecdsa_k1` and Stork asset IDs.
pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    let mut digest = [0; 32];
    hasher.finalize(&mut digest);
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keccak256_of_empty_input() {
        // Keccak-256, not the SHA3-256 standardized later.
        assert_eq!(
            const_hex::encode(keccak256(b"")),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }
}
