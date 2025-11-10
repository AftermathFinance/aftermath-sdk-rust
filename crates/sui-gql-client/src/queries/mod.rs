use af_sui_types::{
    Address as SuiAddress,
    Address,
    Object,
    ObjectRef,
    StructTag,
    Transaction,
    Version,
};
// For `object_args!` macro only
#[doc(hidden)]
use futures::Stream;

mod coin_metadata;
mod current_epoch;
mod events_backward;
mod full_objects;
mod latest_checkpoint;
mod latest_full_objects;
pub mod model;
mod object_df_by_name;
mod object_dfs;
mod object_dof_by_name;
mod object_type;
mod owner_gas_coins;
mod packages;
pub(crate) mod stream;
use crate::queries::model::fragments::{EventEdge, EventFilter};
// mod transaction_blocks_status;
mod transactions_by_digests;
use crate::queries::model::outputs::{DynamicField, RawMoveValue};
use crate::{GraphQlClient, GraphQlErrors};

/// Standard query result type to aid in adding new queries.
type Result<T, C> = std::result::Result<T, Error<<C as GraphQlClient>::Error>>;

/// Extension trait to [`GraphQlClient`] collecting all defined queries in one place.
#[trait_variant::make(Send)]
pub trait GraphQlClientExt: GraphQlClient + Sized {
    // NOTE: `.await` is not used in the implementations below because `trait_variant` de-sugars the
    // method definitions removing their `async` prefixes

    /// The latest epoch id and reference gas price
    async fn current_epoch(&self) -> Result<(u64, u64), Self> {
        current_epoch::query(self)
    }

    /// Return a single page of events + cursors and a flag indicating if there's a previous page.
    ///
    /// If `page_size` is left `None`, the server decides the size of the page.
    ///
    /// The edges are returned in reverse order of which they where returned by the server
    async fn events_backward(
        &self,
        filter: Option<EventFilter>,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> Result<(Vec<EventEdge>, bool), Self> {
        events_backward::query(self, filter, cursor, page_size)
    }

    /// The full [`Object`] contents at specific versions.
    ///
    /// Duplicate object keys are automatically discarded.
    async fn full_objects(
        &self,
        keys: impl IntoIterator<Item = (Address, Option<Version>)> + Send,
        at_checkpoint: Option<u64>,
    ) -> Result<Vec<Object>, Self> {
        self::full_objects::query(self, keys, at_checkpoint)
    }

    /// Latest checkpoint number.
    async fn latest_checkpoint(&self) -> Result<u64, Self> {
        latest_checkpoint::query(self)
    }

    /// The full [`Object`] contents at their latest versions.
    ///
    /// Fails if any requested object id is not in the final map.
    ///
    /// # Note
    ///
    /// The check for returned object ids is just so that the caller can safely do `map[object_id]`
    /// on the returned map. Keep in mind that the result if an object id is repeated in `objects`
    /// is undefined. Avoid doing so.
    fn latest_full_objects(
        &self,
        owner: Option<SuiAddress>,
        type_: Option<String>,
        page_size: Option<u32>,
    ) -> impl Stream<Item = Result<Object, Self>> + '_ {
        latest_full_objects::query(self, owner, type_, page_size)
    }

    /// Get the raw Move value of a dynamic field's value.
    async fn object_df_by_name(
        &self,
        address: Address,
        raw_move_value: RawMoveValue,
        at_checkpoint: Option<u64>,
    ) -> Result<DynamicField, Self> {
        object_df_by_name::query(self, address, raw_move_value, at_checkpoint)
    }

    /// Get the raw Move value of a dynamic object field's value.
    async fn object_dof_by_name(
        &self,
        address: Address,
        raw_move_value: RawMoveValue,
        at_checkpoint: Option<u64>,
    ) -> Result<DynamicField, Self> {
        object_dof_by_name::query(self, address, raw_move_value, at_checkpoint)
    }

    /// **Streamed** map of all keys to dynamic field and dynamic object field values
    /// [`RawMoveValue`] -> [`DynamicField`].
    async fn object_dfs(
        &self,
        address: Address,
        at_checkpoint: Option<u64>,
        page_size: Option<i32>,
    ) -> impl Stream<Item = Result<(RawMoveValue, DynamicField), Self>> + '_ {
        object_dfs::query(self, address, at_checkpoint, page_size)
    }

    /// The full [`Object`] contents at their latest versions.
    ///
    /// Fails if any requested object id is not in the final map.
    ///
    /// # Note
    ///
    /// The check for returned object ids is just so that the caller can safely do `map[object_id]`
    /// on the returned map. Keep in mind that the result if an object id is repeated in `objects`
    /// is undefined. Avoid doing so.
    fn owner_gas_coins(
        &self,
        owner: SuiAddress,
        type_: Option<String>,
        page_size: Option<u32>,
    ) -> impl Stream<Item = Result<(u64, ObjectRef, u64), Self>> + '_ {
        owner_gas_coins::query(self, owner, type_, page_size)
    }

    /// Get all the package ids and versions given either the original package id or
    /// an upgraded package id.
    async fn packages(&self, package_id: Address) -> Result<Vec<(Address, u64)>, Self> {
        packages::query(self, package_id)
    }

    /// Get transactions given their digests
    async fn transactions(
        &self,
        transaction_digests: Vec<String>,
    ) -> Result<Vec<Transaction>, Self> {
        transactions_by_digests::query(self, transaction_digests)
    }

    // /// Get execution status for the input transaction digests
    // async fn transaction_blocks_status(
    //     &self,
    //     transaction_digests: Vec<String>,
    // ) -> Result<impl Iterator<Item = crate::extract::Result<(String, bool)>>, Self> {
    //     transaction_blocks_status::query(self, transaction_digests)
    // }

    /// Struct type of an object given its ID.
    async fn object_type(&self, id: Address) -> Result<StructTag, Self> {
        object_type::query(self, id)
    }

    /// Fetches metadata for the given coin type
    ///
    /// Returns a tuple containing (decimals, name, symbol)
    async fn coin_metadata(
        &self,
        type_: &str,
    ) -> Result<(Option<u8>, Option<String>, Option<String>), Self> {
        coin_metadata::query(self, type_)
    }
}

impl<T: GraphQlClient> GraphQlClientExt for T {}

/// Generic error type for queries.
#[derive(thiserror::Error, Clone, Debug)]
pub enum Error<C: std::error::Error> {
    #[error("Client error: {0:?}")]
    Client(C),
    #[error("In server response: {0}")]
    Server(#[from] GraphQlErrors),
    #[error("Missing data in response: {0}")]
    MissingData(String),
}

#[expect(deprecated, reason = "Internal module deprecation")]
impl<C: std::error::Error> From<crate::extract::Error> for Error<C> {
    fn from(value: crate::extract::Error) -> Self {
        Self::MissingData(value.0)
    }
}

impl<C: std::error::Error> From<&'static str> for Error<C> {
    fn from(value: &'static str) -> Self {
        Self::MissingData(value.into())
    }
}

/// Helper to generate [`Error::MissingData`].
///
/// Works very much like an `anyhow!`/`eyre!` macro, but intended for the case when trying to
/// extract some data from the query.
#[macro_export]
macro_rules! missing_data {
    ($($msg:tt)*) => {
        $crate::queries::Error::MissingData(format!($($msg)*))
    };
}
