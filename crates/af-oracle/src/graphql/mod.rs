use af_move_type::MoveInstance;
use af_sui_types::{Address, ObjectId, Version};
use futures::Stream;
use sui_gql_client::GraphQlClient;
use sui_gql_client::queries::Error;

pub use crate::graphql::price_feed_for_source::Error as PfForSourceError;
use crate::oracle::PriceFeed;

pub(crate) mod price_feed_for_source;
pub(crate) mod price_feeds;

/// Extension trait to [`GraphQlClient`] collecting all defined queries in one place.
pub trait GraphQlClientExt: GraphQlClient + Sized {
    /// Snapshot of price feeds under the [`PriceFeedStorage`].
    /// Returns tuples representing (`source_wrapper_id`, `price_feed`)
    ///
    /// [`PriceFeedStorage`]: crate::oracle::PriceFeedStorage
    fn price_feeds(
        &self,
        pfs: ObjectId,
        version: Option<Version>,
    ) -> impl Stream<Item = Result<(ObjectId, MoveInstance<PriceFeed>), Error<Self::Error>>> + '_
    {
        price_feeds::query(self, pfs, version)
    }

    /// Get a price feed under the [`PriceFeedStorage`], given a specific `source_wrapper_id`.
    ///
    /// [`PriceFeedStorage`]: crate::oracle::PriceFeedStorage
    fn price_feed_for_source(
        &self,
        af_oracle_pkg: Address,
        pfs: ObjectId,
        source_wrapper_id: ObjectId,
    ) -> impl Future<Output = Result<Option<MoveInstance<PriceFeed>>, PfForSourceError<Self::Error>>>
    + Send {
        price_feed_for_source::query(self, af_oracle_pkg, pfs, source_wrapper_id)
    }
}

impl<T: GraphQlClient> GraphQlClientExt for T {}
