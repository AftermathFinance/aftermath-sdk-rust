use af_sui_types::ObjectId;
use sui_framework_sdk::object::{ID, UID};

use crate::events;
use crate::oracle::PriceFeed;

/// Intended for defining how to update off-chain data with oracle event information.
///
/// This should work under the following assumptions.
///
/// # Assumptions
/// 1. The database is in sync with the on-chain protocol state at least up until the beginning of
///    the event's transaction.
pub trait Apply<T> {
    /// Use the contents of `self` to (possibly) mutate data in `target`.
    fn apply(self, target: &mut T);
}

/// General interface to help apply [`PriceFeedStorage`](crate::oracle::PriceFeedStorage)-related
/// events to a database.
pub trait PriceFeedStorage {
    fn insert_price_feed(&mut self, source_wrapper_id: ID, feed: PriceFeed);

    fn get_price_feed_mut(&mut self, source_wrapper_id: &ID) -> Option<&mut PriceFeed>;

    fn remove_price_feed(&mut self, source_wrapper_id: &ID) -> Option<PriceFeed>;
}

impl<T: PriceFeedStorage> Apply<T> for events::CreatedPriceFeed {
    fn apply(self, target: &mut T) {
        let Self {
            source_wrapper_id,
            price,
            timestamp,
            time_tolerance,
            ..
        } = self;
        let feed = PriceFeed {
            // The event doesn't have information about the object id of the feed. Thus, we
            // initialize it as zero to make it very clear to applications using this that the
            // object id here is not a real one. This shouldn't be a problem since all interactions
            // with price feeds in AfOracle are done through the `PriceFeedStorage`.
            id: UID {
                id: ID {
                    bytes: ObjectId::ZERO,
                },
            },
            price,
            timestamp,
            time_tolerance,
        };
        target.insert_price_feed(source_wrapper_id, feed);
    }
}

impl<T: PriceFeedStorage> Apply<T> for events::UpdatedPriceFeed {
    fn apply(self, target: &mut T) {
        let Self {
            source_wrapper_id,
            new_price,
            new_timestamp,
            ..
        } = self;
        // Under Assumption 1 of [`Apply`], if a price feed is not in the database at this point,
        // it's because we're applying and old event relative to the current state and the price
        // feed was removed later.
        if let Some(feed) = target.get_price_feed_mut(&source_wrapper_id) {
            feed.price = new_price;
            feed.timestamp = new_timestamp;
        }
    }
}

impl<T: PriceFeedStorage> Apply<T> for events::RemovedPriceFeed {
    fn apply(self, target: &mut T) {
        let Self {
            source_wrapper_id, ..
        } = self;
        target.remove_price_feed(&source_wrapper_id);
    }
}

impl<T: PriceFeedStorage> Apply<T> for events::UpdatedPriceFeedTimeTolerance {
    fn apply(self, target: &mut T) {
        let Self {
            source_wrapper_id,
            new_time_tolerance,
            ..
        } = self;
        // Under Assumption 1 of [`Apply`], if a price feed is not in the database at this point,
        // it's because we're applying and old event relative to the current state and the price
        // feed was removed later.
        if let Some(feed) = target.get_price_feed_mut(&source_wrapper_id) {
            feed.time_tolerance = new_time_tolerance;
        }
    }
}
