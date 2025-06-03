//! Helpers for stop orders.

use af_utilities::IFixed;
use fastcrypto::hash::{Blake2b256, HashFunction};
use serde::{Deserialize, Serialize};
use sui_framework_sdk::object::ID;

use crate::order_helpers::{OrderType, Side};

pub trait StopOrderTicketDetails {
    /// Pure transaction input to use when calling `create_stop_order_ticket`.
    fn encrypted_details(&self, salt: Vec<u8>) -> bcs::Result<Vec<u8>>
    where
        Self: serde::Serialize,
    {
        let mut bytes = bcs::to_bytes(self)?;
        bytes.extend(salt);
        Ok(Blake2b256::digest(bytes).to_vec())
    }
}

/// The details to be hashed for the `encrypted_details` argument of `create_stop_order_ticket`.
#[derive(Debug, serde::Serialize)]
pub struct SLTPDetails {
    pub clearing_house_id: ID,
    /// The `Clock` value after (>=) which the order isn't valid anymore
    pub expire_timestamp: Option<u64>,
    /// `true` if limit order, `false` if market order
    pub is_limit_order: bool,
    pub stop_index_price: IFixed,
    /// `true` if stop loss order, `false` if take profit order
    pub is_stop_loss: bool,
    /// - position_is_ask: `true` if position is short, `false` if position is long
    pub position_is_ask: bool,
    pub size: u64,
    /// Can be set at random value if `is_limit_order` is false
    pub price: u64,
    /// Can be set at random value if `is_limit_order` is false
    pub order_type: OrderType,
}

impl StopOrderTicketDetails for SLTPDetails {}

/// The details to be hashed for the `encrypted_details` argument of `create_stop_order_ticket`.
#[derive(Debug, serde::Serialize)]
pub struct StandaloneDetails {
    pub clearing_house_id: ID,
    /// The `Clock` value after (>=) which the order isn't valid anymore
    pub expire_timestamp: Option<u64>,
    /// `true` if limit order, `false` if market order
    pub is_limit_order: bool,
    pub stop_index_price: IFixed,
    /// `true` means the order can be placed when oracle index price is >= than chosen
    /// `stop_index_price`
    pub ge_stop_index_price: bool,
    pub side: Side,
    pub size: u64,
    /// Can be set at random value if `is_limit_order` is false
    pub price: u64,
    /// Can be set at random value if `is_limit_order` is false
    pub order_type: OrderType,
    pub reduce_only: bool,
}

impl StopOrderTicketDetails for StandaloneDetails {}

#[derive(Clone, Copy, Debug, clap::ValueEnum, Serialize, Deserialize)]
#[serde(into = "u64")]
pub enum StopOrderType {
    /// Stop-Loss / Take-Profit type, aimed to reduce position
    SLTP,
    /// Standard stop order, without restrictions
    Standalone,
}

impl From<StopOrderType> for u64 {
    fn from(value: StopOrderType) -> Self {
        match value {
            StopOrderType::SLTP => 0,
            StopOrderType::Standalone => 1,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid stop order type value")]
pub struct InvalidStopOrderTypeValue;

impl TryFrom<u64> for StopOrderType {
    type Error = InvalidStopOrderTypeValue;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::SLTP),
            1 => Ok(Self::Standalone),
            _ => Err(InvalidStopOrderTypeValue),
        }
    }
}
