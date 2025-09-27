#[derive(
    serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
)]
pub struct UID {
    id: ID,
}
#[derive(
    serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
)]
pub struct ID {
    bytes: af_sui_types::ObjectId,
}
#[derive(
    serde::Deserialize, serde::Serialize, Clone, Debug, derive_more::Display, PartialEq, Eq, Hash,
)]
#[display("Balance")]
pub struct Balance<T> {
    _phantom: std::marker::PhantomData<T>,
}
use af_sui_pkg_sdk::sui_pkg_sdk;

sui_pkg_sdk!(package {
    module clearing_house {
        /// Used to dynamically load market objects as needed.
        /// Used to dynamically load traders' position objects as needed.
        struct ClearingHouse<!phantom T> has key {
            id: UID,
            // ...
        }

        /// Stores all deposits from traders for collateral T.
        /// Stores the funds reserved for covering bad debt from untimely
        /// liquidations.
        ///
        /// The Clearing House keeps track of who owns each share of the vault.
        struct Vault<!phantom T> has key, store {
            id: UID,
            collateral_balance: Balance<T>,
            insurance_fund_balance: Balance<T>,
            scaling_factor: u64
        }
    }

    module keys {
        /// Key type for accessing trader position in clearing house.
        struct Position has copy, drop, store {
            account_id: u64,
        }
    }
});

fn main() {}
