//! Helpers for stop-loss orders.

use af_utilities::types::Fixed;
use fastcrypto::hash::{Blake2b256, HashFunction};
use sui_framework_sdk::object::ID;

use crate::order_helpers::{OrderType, Side};

/// The details to be hashed for the `encrypted_details` argument of `create_stop_order_ticket`.
#[derive(Debug, serde::Serialize)]
pub struct StopOrderTicketDetails {
    pub clearing_house_id: ID,
    /// The `Clock` value after (>=) which the order isn't valid anymore
    pub expire_timestamp: u64,
    /// `true` if limit order, `false` if market order
    pub is_limit_order: bool,
    pub stop_index_price: Fixed,
    /// `true` means the order can be placed when oracle index price is >= than chosen
    /// `stop_index_price`
    pub ge_stop_index_price: bool,
    pub side: Side,
    pub size: u64,
    /// Can be set at random value if `is_limit_order` is false
    pub price: u64,
    /// Can be set at random value if `is_limit_order` is false
    pub order_type: OrderType,
}

impl StopOrderTicketDetails {
    /// Pure transaction input to use when calling `create_stop_order_ticket`.
    pub fn encrypted_details(&self, salt: Vec<u8>) -> bcs::Result<Vec<u8>> {
        let mut bytes = bcs::to_bytes(self)?;
        bytes.extend(salt);
        Ok(Blake2b256::digest(bytes).to_vec())
    }
}
