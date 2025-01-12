//! Helpers for updating off-chain data with event content
use af_move_type::otw::Otw;
use af_utilities::types::IFixed;

use crate::account::Account;
use crate::clearing_house::{ClearingHouse, Vault};
use crate::events;
use crate::orderbook::Order;
use crate::position::Position;

/// Intended for defining how to update off-chain data with event information.
pub trait Apply<T> {
    /// Use the contents of `self` to (possibly) mutate data in `target`.
    fn apply(&self, target: &mut T);
}

/// Convenience tool for implementing [`Apply`] for events that contain the values of attributes to
/// set in an object.
///
/// The implementations here don't handle how to fetch each object, that is up to the application.
///
/// For instance, one database might store only a single [`Position`] and ignore the `ch_id` and
/// `account_id` fields of the relevant events because those are already filtered for in the
/// WebSocket event subscriber.
///
/// On the other hand, another database might store several positions in a `HashMap` and use the
/// `account_id` field of the relevant events to fetch the [`Position`] from the map before passing
/// it to `.apply()`.
macro_rules! set_fields {
    (
        $(
            $event_type:ty {
                $(
                    $object_type:ty {
                        $(
                            $event_field:ident => $($object_field:ident).+
                        ),+ $(,)?
                    }
                )+
            }
        )*
    ) => {
        $(
            $(
                impl Apply<$object_type> for $event_type {
                    fn apply(&self, target: &mut $object_type) {
                        let Self {
                            $($event_field,)+
                            ..
                        } = self;
                        $(
                            target.$($object_field).+ = $event_field.clone();
                        )+
                    }
                }
            )+
        )*
    };
}

set_fields! {
    events::DepositedCollateral<Otw> {
        Account<Otw> {
            account_collateral_after => collateral.value
        }
    }

    events::AllocatedCollateral {
        Position {
            position_collateral_after => collateral
        }
        Account<Otw> {
            account_collateral_after => collateral.value
        }
        Vault<Otw> {
            vault_balance_after => collateral_balance.value
        }
    }

    events::WithdrewCollateral<Otw> {
        Account<Otw> {
            account_collateral_after => collateral.value
        }
    }

    events::DeallocatedCollateral {
        Account<Otw> {
            account_collateral_after => collateral.value
        }
        Position {
            position_collateral_after => collateral
        }
        Vault<Otw> {
            vault_balance_after => collateral_balance.value
        }
    }

    events::UpdatedPremiumTwap {
        ClearingHouse<Otw> {
            premium_twap => market_state.premium_twap,
            premium_twap_last_upd_ms => market_state.premium_twap_last_upd_ms,
        }
    }

    events::UpdatedSpreadTwap {
        ClearingHouse<Otw> {
            spread_twap => market_state.spread_twap,
            spread_twap_last_upd_ms => market_state.spread_twap_last_upd_ms,
        }
    }

    events::UpdatedFunding {
        ClearingHouse<Otw> {
            cum_funding_rate_long => market_state.cum_funding_rate_long,
            cum_funding_rate_short => market_state.cum_funding_rate_short,
            funding_last_upd_ms => market_state.funding_last_upd_ms,
        }
    }

    events::UpdatedOpenInterestAndFeesAccrued {
        ClearingHouse<Otw> {
            open_interest => market_state.open_interest,
            fees_accrued => market_state.fees_accrued,
        }
    }

    events::SettledFunding {
        Position {
            collateral_after => collateral,
            mkt_funding_rate_long => cum_funding_rate_long,
            mkt_funding_rate_short => cum_funding_rate_short,
        }
    }

    events::FilledMakerOrder {
        Position {
            maker_collateral => collateral,
            maker_base_amount => base_asset_amount,
            maker_quote_amount => quote_asset_notional_amount,
            maker_pending_asks_quantity => asks_quantity,
            maker_pending_bids_quantity => bids_quantity,
        }
    }

    events::FilledTakerOrder {
        Position {
            taker_collateral => collateral,
            taker_base_amount => base_asset_amount,
            taker_quote_amount => quote_asset_notional_amount,
        }
    }

    events::PostedOrder {
        Position {
            pending_asks => asks_quantity,
            pending_bids => bids_quantity,
            pending_orders => pending_orders,
        }
    }

    events::CanceledOrders {
        Position {
            asks_quantity => asks_quantity,
            bids_quantity => bids_quantity,
            pending_orders => pending_orders,
        }
    }

    events::UpdatedCumFundings {
        ClearingHouse<Otw> {
            cum_funding_rate_long => market_state.cum_funding_rate_long,
            cum_funding_rate_short => market_state.cum_funding_rate_short,
        }
    }

    events::UpdatedMarginRatios {
        ClearingHouse<Otw> {
            margin_ratio_initial => market_params.margin_ratio_initial,
            margin_ratio_maintenance => market_params.margin_ratio_maintenance,
        }
    }

    events::AcceptedPositionFeesProposal {
        Position {
            maker_fee => maker_fee,
            taker_fee => taker_fee,
        }
    }

    events::UpdatedFees {
        ClearingHouse<Otw> {
            maker_fee => market_params.maker_fee,
            taker_fee => market_params.taker_fee,
            liquidation_fee => market_params.liquidation_fee,
            force_cancel_fee => market_params.force_cancel_fee,
            insurance_fund_fee => market_params.insurance_fund_fee,
        }
    }

    events::UpdatedFundingParameters {
        ClearingHouse<Otw> {
            funding_frequency_ms => market_params.funding_frequency_ms,
            funding_period_ms => market_params.funding_period_ms,
            premium_twap_frequency_ms => market_params.premium_twap_frequency_ms,
            premium_twap_period_ms => market_params.premium_twap_period_ms,
        }
    }

    events::UpdatedSpreadTwapParameters {
        ClearingHouse<Otw> {
            spread_twap_frequency_ms => market_params.spread_twap_frequency_ms,
            spread_twap_period_ms => market_params.spread_twap_period_ms,
        }
    }

    events::UpdatedMinOrderUsdValue {
        ClearingHouse<Otw> {
            min_order_usd_value => market_params.min_order_usd_value,
        }
    }

    events::UpdatedLiquidationTolerance {
        ClearingHouse<Otw> {
            liquidation_tolerance => market_params.liquidation_tolerance,
        }
    }

    events::UpdatedBaseOracleTolerance {
        ClearingHouse<Otw> {
            oracle_tolerance => market_params.base_oracle_tolerance,
        }
    }

    events::UpdatedCollateralOracleTolerance {
        ClearingHouse<Otw> {
            oracle_tolerance => market_params.collateral_oracle_tolerance,
        }
    }

    events::UpdatedMaxPendingOrders {
        ClearingHouse<Otw> {
            max_pending_orders => market_params.max_pending_orders,
        }
    }

    events::DonatedToInsuranceFund {
        Vault<Otw> {
            new_balance => insurance_fund_balance.value,
        }
    }

    events::WithdrewFees {
        Vault<Otw> {
            vault_balance_after => collateral_balance.value,
        }
    }

    events::WithdrewInsuranceFund {
        Vault<Otw> {
            insurance_fund_balance_after => insurance_fund_balance.value,
        }
    }
}

impl Apply<ClearingHouse<Otw>> for events::WithdrewFees {
    fn apply(&self, target: &mut ClearingHouse<Otw>) {
        target.market_state.fees_accrued = IFixed::zero();
    }
}

// When you reset fees you set at a constant value which is used as a "null" value. It's in the
// constants module of the contracts.
impl Apply<Position> for events::ResettedPositionFees {
    fn apply(&self, target: &mut Position) {
        target.maker_fee = 1.into();
        target.taker_fee = 1.into();
    }
}

impl Apply<Position> for events::LiquidatedPosition {
    fn apply(&self, target: &mut Position) {
        let Self {
            liqee_collateral,
            liqee_base_amount,
            liqee_quote_amount,
            ..
        } = *self;
        target.collateral = liqee_collateral;
        target.base_asset_amount = liqee_base_amount;
        target.quote_asset_notional_amount = liqee_quote_amount;
        // All pending orders are force canceled upon liquidation
        target.asks_quantity = IFixed::zero();
        target.bids_quantity = IFixed::zero();
        target.pending_orders = 0;
    }
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
            maker_size,
            maker_final_size,
            ..
        } = *self;
        if maker_final_size > 0 {
            target.reduce_order_size(order_id, maker_size)
        } else {
            target.remove_order(order_id);
        }
    }
}
