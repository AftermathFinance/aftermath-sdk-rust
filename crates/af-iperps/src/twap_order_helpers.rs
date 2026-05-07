//! Helpers for TWAP orders.

use fastcrypto::hash::{Blake2b256, HashFunction};
use serde::Serialize;

pub trait TWAPOrderTicketDetails {
    /// Pure transaction input to use when calling `create_twap_order_ticket`.
    fn encrypted_details(&self) -> Result<Vec<u8>, sui_sdk_types::bcs::Error>
    where
        Self: Serialize,
    {
        Ok(Blake2b256::digest(sui_sdk_types::bcs::ToBcs::to_bcs(&self)?).to_vec())
    }
}

/// The details to be hashed for the `encrypted_details` argument of
/// `create_twap_order_ticket`.
#[derive(Debug, Serialize)]
pub struct TWAPDetails {
    pub first_run_expire_timestamp: Option<u64>,
    pub expire_timestamp: Option<u64>,
    pub execution_gap_ms: u64,
    pub execution_time_uncertainty_ms: u64,
    pub chunks_amount: u64,
    pub small_tail_merge_threshold_bps: u64,
    pub time_for_retry_ms: u64,
    pub amount_uncertainty_bps: u64,
    pub max_one_execution_amount_bps: u64,
    pub side: bool,
    pub size: u64,
    pub max_slippage_bps: u64,
    pub reduce_only: bool,
    pub salt: Vec<u8>,
}

impl TWAPOrderTicketDetails for TWAPDetails {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_details_hashes_bcs_payload_including_salt() {
        let details = TWAPDetails {
            first_run_expire_timestamp: Some(1),
            expire_timestamp: Some(2),
            execution_gap_ms: 3,
            execution_time_uncertainty_ms: 4,
            chunks_amount: 5,
            small_tail_merge_threshold_bps: 6,
            time_for_retry_ms: 7,
            amount_uncertainty_bps: 8,
            max_one_execution_amount_bps: 9,
            side: true,
            size: 10,
            max_slippage_bps: 11,
            reduce_only: false,
            salt: vec![12; 32],
        };

        let expected = Blake2b256::digest(sui_sdk_types::bcs::ToBcs::to_bcs(&details).unwrap());
        let actual = details.encrypted_details().unwrap();

        assert_eq!(actual, expected.to_vec());
    }
}
