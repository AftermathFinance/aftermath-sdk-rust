#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

//! Light-weight, read-only version of Sui's file-based keystore.

pub mod keystore;
pub mod multisig;
pub mod public_key;
pub mod utils;

pub use self::keystore::{Alias, Keystore};
pub use self::public_key::PublicKey;
