use std::cmp::max;

use af_move_type::MoveType;
use af_utilities::{Balance9, IFixed};
use num_traits::Zero as _;

use crate::clearing_house::ClearingHouse;
use crate::{MarketParams, MarketState, Position};

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Overflow when converting types")]
    Overflow,
    #[error("Not enough precision to represent price")]
    Precision,
    #[error("Division by zero")]
    DivisionByZero,
}

/// Convenience trait to convert to/from units used in the orderbook.
pub trait OrderBookUnits {
    /// Price in the orderbook to fixed-point number.
    ///
    /// # Panics
    ///
    /// If `self.lot_size() == 0`
    fn price_to_ifixed(&self, price: u64) -> IFixed {
        let price_ifixed = IFixed::from(price);
        let lot_size_ifixed = IFixed::from(self.lot_size());
        let tick_size_ifixed = IFixed::from(self.tick_size());
        price_ifixed * tick_size_ifixed / lot_size_ifixed
    }

    /// The price in ticks/lot closest to the desired value.
    ///
    /// Note that this:
    /// - rounds the equivalent ticks/lot **down** to the nearest integer.
    /// - errors if the equivalent ticks/lot < 1, signaling not enough precision.
    fn ifixed_to_price(&self, ifixed: IFixed) -> Result<u64, Error> {
        if ifixed.is_zero() {
            return Ok(0);
        }
        // ifixed = (price_ifixed * tick_size_ifixed) / lot_size_ifixed
        // (ifixed * lot_size_ifixed) / tick_size_ifixed = price_ifixed
        if self.tick_size() == 0 {
            return Err(Error::DivisionByZero);
        }
        // safety: we checked agains division by zero above.
        let price_ifixed =
            (ifixed * IFixed::from(self.lot_size())) / IFixed::from(self.tick_size());
        let price: u64 = price_ifixed
            .integer()
            .uabs()
            .try_into()
            .map_err(|_| Error::Overflow)?;
        if price == 0 {
            return Err(Error::Precision);
        }
        Ok(price)
    }

    fn lots_to_ifixed(&self, lots: u64) -> IFixed {
        let ifixed_lots: IFixed = lots.into();
        let ifixed_lot_size: IFixed = Balance9::from_inner(self.lot_size()).into();
        ifixed_lots * ifixed_lot_size
    }

    fn ifixed_to_lots(&self, ifixed: IFixed) -> Result<u64, Error> {
        let balance: Balance9 = ifixed.try_into().map_err(|_| Error::Overflow)?;
        balance
            .into_inner()
            .checked_div(self.lot_size())
            .ok_or(Error::DivisionByZero)
    }

    // NOTE: these could be updated to return NonZeroU64 ensuring division by zero errors are
    // impossible.
    fn lot_size(&self) -> u64;
    fn tick_size(&self) -> u64;
}

impl OrderBookUnits for MarketParams {
    fn lot_size(&self) -> u64 {
        self.lot_size
    }

    fn tick_size(&self) -> u64 {
        self.tick_size
    }
}

impl<T: MoveType> OrderBookUnits for ClearingHouse<T> {
    fn lot_size(&self) -> u64 {
        self.market_params.lot_size
    }

    fn tick_size(&self) -> u64 {
        self.market_params.tick_size
    }
}

impl<T: MoveType> ClearingHouse<T> {
    /// Convenience method for computing a position's liquidation price.
    ///
    /// Forwards to [`Position::liquidation_price`].
    pub fn liquidation_price(&self, pos: &Position, coll_price: IFixed) -> Option<IFixed> {
        pos.liquidation_price(
            coll_price,
            self.market_state.cum_funding_rate_long,
            self.market_state.cum_funding_rate_short,
            self.market_params.margin_ratio_maintenance,
        )
    }
}

impl MarketParams {
    /// The initial and maintenance margin requirements for a certain notional, in the same units.
    pub fn margin_requirements(&self, notional: IFixed) -> (IFixed, IFixed) {
        let min_margin = notional * self.margin_ratio_initial;
        let liq_margin = notional * self.margin_ratio_maintenance;
        (min_margin, liq_margin)
    }
}

impl MarketState {
    /// Convenience method for computing a position's unrealized funding.
    ///
    /// Forwards to [`Position::unrealized_funding`].
    pub fn unrealized_funding(&self, pos: &Position) -> IFixed {
        pos.unrealized_funding(self.cum_funding_rate_long, self.cum_funding_rate_short)
    }
}

impl Position {
    /// At which index price the position should be (partially) liquidated, assuming all the input
    /// variables stay the same.
    pub fn liquidation_price(
        &self,
        coll_price: IFixed,
        cum_funding_rate_long: IFixed,
        cum_funding_rate_short: IFixed,
        maintenance_margin_ratio: IFixed,
    ) -> Option<IFixed> {
        let coll = self.collateral * coll_price;
        let ufunding = self.unrealized_funding(cum_funding_rate_long, cum_funding_rate_short);
        let quote = self.quote_asset_notional_amount;

        let size = self.base_asset_amount;
        let bids_net_abs = (size + self.bids_quantity).abs();
        let asks_net_abs = (size - self.asks_quantity).abs();
        let max_abs_net_base = max(bids_net_abs, asks_net_abs);

        let denominator = max_abs_net_base * maintenance_margin_ratio - size;
        if denominator.is_zero() {
            None
        } else {
            Some((coll + ufunding - quote) / denominator)
        }
    }

    /// Entry price of the position's contracts; in the same units as the oracle index price.
    ///
    /// This function returns `None` if the position has no open contracts, i.e., if
    /// [`Self::base_asset_amount`] is zero.
    pub fn entry_price(&self) -> Option<IFixed> {
        if self.base_asset_amount.is_zero() {
            return None;
        }
        Some(self.quote_asset_notional_amount / self.base_asset_amount)
    }

    /// The funding yet to be settled in this position given the market's current cumulative
    /// fundings.
    ///
    /// The return value is in the same quote currency that the index price uses. E.g., if the
    /// index price is USD/BTC, then the unrealized funding is in USD units.
    pub fn unrealized_funding(
        &self,
        cum_funding_rate_long: IFixed,
        cum_funding_rate_short: IFixed,
    ) -> IFixed {
        if self.base_asset_amount.is_neg() {
            unrealized_funding(
                cum_funding_rate_short,
                self.cum_funding_rate_short,
                self.base_asset_amount,
            )
        } else {
            unrealized_funding(
                cum_funding_rate_long,
                self.cum_funding_rate_long,
                self.base_asset_amount,
            )
        }
    }

    /// Unrealized PnL given an index price.
    ///
    /// The returned value is in the same currency as what the index price is quoted at. E.g., if
    /// the index price is a ratio of BTC/ETH, then the PnL is in BTC units.
    pub fn unrealized_pnl(&self, price: IFixed) -> IFixed {
        (self.base_asset_amount * price) - self.quote_asset_notional_amount
    }

    /// Total position value used for risk calculations.
    ///
    /// The returned value is in the same currency as what the index price is quoted at. E.g., if
    /// the index price is a ratio of BTC/ETH, then the PnL is in BTC units.
    pub fn notional(&self, price: IFixed) -> IFixed {
        let size = self.base_asset_amount;
        let bids_net_abs = (size + self.bids_quantity).abs();
        let asks_net_abs = (size - self.asks_quantity).abs();
        let max_abs_net_base = max(bids_net_abs, asks_net_abs);
        max_abs_net_base * price
    }
}

fn unrealized_funding(
    cum_funding_rate_now: IFixed,
    cum_funding_rate_before: IFixed,
    size: IFixed,
) -> IFixed {
    if cum_funding_rate_now == cum_funding_rate_before {
        return IFixed::zero();
    };

    (cum_funding_rate_now - cum_funding_rate_before) * (-size)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use proptest::prelude::*;
    use test_strategy::{Arbitrary, proptest};

    use super::*;

    impl OrderBookUnits for (u64, u64) {
        fn lot_size(&self) -> u64 {
            self.0
        }

        fn tick_size(&self) -> u64 {
            self.1
        }
    }

    #[test]
    fn orderbook_units() {
        let mut units = (10_000_000, 1_000_000);
        let mut ifixed: IFixed;

        ifixed = u64::MAX.into();
        ifixed += IFixed::from_inner(1.into());
        insta::assert_snapshot!(ifixed, @"18446744073709551615.000000000000000001");
        let err = units.ifixed_to_lots(ifixed).unwrap_err();
        insta::assert_snapshot!(err, @"Overflow when converting types");

        // Values smaller than 1 balance9 get cast to 0
        ifixed = IFixed::from_inner(1.into());
        insta::assert_snapshot!(ifixed, @"0.000000000000000001");
        let ok = units.ifixed_to_lots(ifixed).unwrap();
        assert_eq!(ok, 0);

        ifixed = 0.001.try_into().unwrap();
        insta::assert_snapshot!(ifixed, @"0.001");
        let err = units.ifixed_to_price(ifixed).unwrap_err();
        insta::assert_snapshot!(err, @"Not enough precision to represent price");

        ifixed = 0.0.try_into().unwrap();
        insta::assert_snapshot!(ifixed, @"0.0");
        let ok = units.ifixed_to_price(ifixed).unwrap();
        assert_eq!(ok, 0);

        ifixed = 0.1.try_into().unwrap();
        insta::assert_snapshot!(ifixed, @"0.1");
        let ok = units.ifixed_to_price(ifixed).unwrap();
        assert_eq!(ok, 1);

        // `ifixed_to_price` truncates
        ifixed = 0.15.try_into().unwrap();
        insta::assert_snapshot!(ifixed, @"0.15");
        let ok = units.ifixed_to_price(ifixed).unwrap();
        assert_eq!(ok, 1);

        ifixed = units.price_to_ifixed(0);
        insta::assert_snapshot!(ifixed, @"0.0");

        // Can handle an absurdly large price no problem
        units = (1, u64::MAX);
        let ok = units.price_to_ifixed(u64::MAX);
        insta::assert_snapshot!(ok, @"340282366920938463426481119284349108225.0");

        // Can handle an absurdly large lot size no problem
        units = (u64::MAX, 1);
        let ok = units.lots_to_ifixed(u64::MAX);
        insta::assert_snapshot!(ok, @"340282366920938463426481119284.349108225");

        units = (100000, 1000);
        let min_amount = units.lots_to_ifixed(1);
        insta::assert_snapshot!(min_amount, @"0.0001");
        let price_precision = units.price_to_ifixed(1);
        insta::assert_snapshot!(price_precision, @"0.01");
    }

    #[test]
    #[should_panic]
    fn zero_lot_and_tick() {
        (0u64, 0u64).price_to_ifixed(1);
    }

    #[test]
    #[should_panic]
    fn zero_lot() {
        (0u64, 1u64).price_to_ifixed(1);
    }

    #[test]
    fn zero_tick() {
        assert_eq!((1u64, 0u64).price_to_ifixed(1), IFixed::zero());
    }

    #[test]
    #[should_panic]
    fn ifixed_to_price() {
        (1u64, 0u64).ifixed_to_price(IFixed::one()).expect("Panics");
    }

    #[derive(Arbitrary, Debug)]
    struct Contracts {
        lots: NonZeroU64,
        ticks: NonZeroU64,
        short: bool,
    }

    impl Position {
        fn from_contracts(
            collateral: IFixed,
            contracts: Contracts,
            params: &impl OrderBookUnits,
        ) -> Self {
            let mut base = params.lots_to_ifixed(contracts.lots.into());
            if contracts.short {
                base = -base;
            }
            let mut quote = params.lots_to_ifixed(contracts.ticks.into());
            if contracts.short {
                quote = -quote;
            }
            Self {
                collateral,
                base_asset_amount: base,
                quote_asset_notional_amount: quote,
                cum_funding_rate_long: 0.into(),
                cum_funding_rate_short: 0.into(),
                asks_quantity: 0.into(),
                bids_quantity: 0.into(),
                pending_orders: 0,
                maker_fee: 1.into(),
                taker_fee: 1.into(),
                initial_margin_ratio: 1.into(),
            }
        }

        fn empty(collateral: IFixed) -> Self {
            Self {
                collateral,
                base_asset_amount: 0.into(),
                quote_asset_notional_amount: 0.into(),
                cum_funding_rate_long: 0.into(),
                cum_funding_rate_short: 0.into(),
                asks_quantity: 0.into(),
                bids_quantity: 0.into(),
                pending_orders: 0,
                maker_fee: 1.into(),
                taker_fee: 1.into(),
                initial_margin_ratio: 1.into(),
            }
        }
    }

    #[proptest]
    fn liquidation_price_is_positive(
        contracts: Contracts,
        #[strategy(0.0001..=1e12)] coll_price: f64,
        #[strategy(0.0001..=0.5)] maintenance_margin_ratio: f64,
        #[strategy(1..=1_000_000_000_u64)] lot_size: u64,
        #[strategy(1..=#lot_size)] tick_size: u64,
    ) {
        let position = Position::from_contracts(1.into(), contracts, &(lot_size, tick_size));
        let liq_price = position
            .liquidation_price(
                coll_price.try_into().unwrap(),
                IFixed::zero(),
                IFixed::zero(),
                maintenance_margin_ratio.try_into().unwrap(),
            )
            .unwrap();
        dbg!(liq_price.to_string());
        assert!(liq_price > IFixed::zero());
    }

    #[proptest]
    fn liquidation_price_none(
        #[strategy(any::<NonZeroU64>())]
        #[map(|x: NonZeroU64| Balance9::from_inner(x.get()))]
        collateral: Balance9,
        #[strategy(0.0001..=1e12)] coll_price: f64,
    ) {
        let position = Position::empty(collateral.into());
        let liq_price = position.liquidation_price(
            coll_price.try_into().unwrap(),
            IFixed::zero(),
            IFixed::zero(),
            0.001.try_into().unwrap(),
        );
        assert_eq!(liq_price, None);
    }
}
