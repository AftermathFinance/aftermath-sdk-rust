#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Move types for the `utilities` package and off-chain numerical types for reproducing
//! calculations.

pub mod types;

pub use af_sui_types::U256;
pub use types::{Balance9, IFixed, I256};
