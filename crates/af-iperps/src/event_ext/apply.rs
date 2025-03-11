//! Helpers for updating off-chain orderbook data with event content

use crate::events;
use crate::orderbook::Order;

/// Intended for defining how to update off-chain data with event information.
pub trait Apply<T> {
    /// Use the contents of `self` to (possibly) mutate data in `target`.
    fn apply(&self, target: &mut T);
}

/// General interface to help apply orderbook-related events to a database.
///
/// The methods here don't have to be used as an interface for other applications, they're only
/// defined for conveniently calling `event.apply(&mut database)` on whatever the `database` type
/// is.
pub trait Orderbook {
    /// To use with `OrderbookPostReceipt` event.
    fn insert_order(&mut self, order_id: u128, order: Order);

    /// To use with `CanceledOrder` or `FilledMakerOrder`
    /// in which maker order was fully filled.
    fn remove_order(&mut self, order_id: u128);

    /// To use with `FilledMakerOrder` in which
    /// maker order was not fully filled.
    fn reduce_order_size(&mut self, order_id: u128, size_to_sub: u64);
}

impl<T: Orderbook> Apply<T> for events::OrderbookPostReceipt {
    fn apply(&self, target: &mut T) {
        let Self {
            order_id,
            order_size: size,
            account_id,
            ..
        } = *self;
        let order = Order { account_id, size };
        target.insert_order(order_id, order);
    }
}

impl<T: Orderbook> Apply<T> for events::CanceledOrder {
    fn apply(&self, target: &mut T) {
        let Self { order_id, .. } = *self;
        target.remove_order(order_id);
    }
}

impl<T: Orderbook> Apply<T> for events::FilledMakerOrder {
    fn apply(&self, target: &mut T) {
        let Self {
            order_id,
            filled_size,
            remaining_size,
            ..
        } = *self;
        if remaining_size > 0 {
            target.reduce_order_size(order_id, filled_size)
        } else {
            target.remove_order(order_id);
        }
    }
}
