use std::ops::Not;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OrderDetails {
    pub account_id: u64,
    pub price: u64,
    pub size: u64,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum, Serialize, Deserialize, Eq, PartialEq)]
#[serde(into = "bool")]
pub enum Side {
    Bid,
    Ask,
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Bid => Self::Ask,
            Self::Ask => Self::Bid,
        }
    }
}

impl From<Side> for bool {
    fn from(value: Side) -> Self {
        match value {
            Side::Bid => false,
            Side::Ask => true,
        }
    }
}

impl From<bool> for Side {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Bid,
            true => Self::Ask,
        }
    }
}

#[derive(Clone, Copy, Debug, clap::ValueEnum, Serialize, Deserialize)]
#[serde(into = "u64")]
pub enum OrderType {
    Standard,
    /// Mandates that the entire order size be filled in the current transaction. Otherwise, the
    /// order is canceled.
    FillOrKill,
    /// Mandates that the entire order not be filled at all in the current transaction. Otherwise,
    /// cancel the order.
    PostOnly,
    /// Mandates that maximal possible part of an order will be filled in the current transaction.
    /// The rest of the order canceled.
    ImmediateOrCancel,
}

impl From<OrderType> for u64 {
    fn from(value: OrderType) -> Self {
        match value {
            OrderType::Standard => 0,
            OrderType::FillOrKill => 1,
            OrderType::PostOnly => 2,
            OrderType::ImmediateOrCancel => 3,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid order type value")]
pub struct InvalidOrderTypeValue;

impl TryFrom<u64> for OrderType {
    type Error = InvalidOrderTypeValue;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Standard),
            1 => Ok(Self::FillOrKill),
            2 => Ok(Self::PostOnly),
            3 => Ok(Self::ImmediateOrCancel),
            _ => Err(InvalidOrderTypeValue),
        }
    }
}
