use crate::order_helpers::Side;

/// Return the side of the order
pub const fn order_side(order_id: u128) -> Side {
    if order_id < 0x8000_0000_0000_0000_0000_0000_0000_0000 {
        Side::Ask
    } else {
        Side::Bid
    }
}

/// Return order ID for `price` and `counter` on given `side`
pub const fn order_id(price: u64, counter: u64, side: Side) -> u128 {
    // Return corresponding order ID type based on side
    match side {
        Side::Ask => order_id_ask(price, counter),
        Side::Bid => order_id_bid(price, counter),
    }
}

/// Return order ID for ask with `price` and `counter`
pub const fn order_id_ask(price: u64, counter: u64) -> u128 {
    (price as u128) << 64 | (counter as u128)
}

/// Return order ID for bid with `price` and `counter`
pub const fn order_id_bid(price: u64, counter: u64) -> u128 {
    ((price ^ 0xffff_ffff_ffff_ffff) as u128) << 64 | (counter as u128)
}

/// Returns price of a given `order_id`.
pub const fn price(order_id: u128) -> u64 {
    match order_side(order_id) {
        Side::Ask => price_ask(order_id),
        Side::Bid => price_bid(order_id),
    }
}

/// Returns price of a given ask `order_id`.
pub const fn price_ask(order_id: u128) -> u64 {
    (order_id >> 64) as u64
}

/// Returns price of a given bid `order_id`.
pub const fn price_bid(order_id: u128) -> u64 {
    ((order_id >> 64) as u64) ^ 0xffff_ffff_ffff_ffff
}

/// Returns counter of a given `order_id`.
pub const fn counter(order_id: u128) -> u64 {
    (order_id & 0x0000_0000_0000_0000_ffff_ffff_ffff_ffff) as u64
}
