#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Move types for Aftermath's `Perpetuals` package

use af_move_type::otw::Otw;
use af_sui_pkg_sdk::sui_pkg_sdk;
use af_sui_types::{hex_address_bytes, ObjectId};
use af_utilities::types::ifixed::IFixed;
use sui_framework_sdk::balance::Balance;
use sui_framework_sdk::object::{ID, UID};

pub mod errors;
pub mod event_ext;
pub mod event_instance;
#[cfg(feature = "graphql")]
pub mod graphql;
pub mod order_helpers;
pub mod order_id;
pub mod slo;

// Convenient aliases since these types will never exist onchain with a type argument other than an
// OTW.
pub type Account = account::Account<Otw>;
pub type AccountTypeTag = account::AccountTypeTag<Otw>;
pub type ClearingHouse = clearing_house::ClearingHouse<Otw>;
pub type ClearingHouseTypeTag = clearing_house::ClearingHouseTypeTag<Otw>;
pub type Vault = clearing_house::Vault<Otw>;
pub type VaultTypeTag = clearing_house::VaultTypeTag<Otw>;

/// Package IDs of the perpetuals contract versions published on testnet, in order of its versions.
pub const TESTNET_PACKAGE_VERSIONS: &[ObjectId] = &[ObjectId::new(hex_address_bytes(
    b"0x9725155a70cf2d2241b8cc2fa8376809689312cabb4acaa5ca5ba47eaf4d611f",
))];

sui_pkg_sdk!(perpetuals {
    module account {
        /// The Account object saves the collateral available to be used in clearing houses.
        struct Account<!phantom T> has key, store {
            id: UID,
            /// Numerical value associated to the account
            account_id: u64,
            /// Balance available to be allocated to markets.
            collateral: Balance<T>,
        }

        /// Object that allows to place one order on behalf of the user, used to
        /// offer stop limit or market orders. A stop order is an order that is placed
        /// only if the index price respects certain conditions, like being above or
        /// below a certain price.
        ///
        /// Only the `Account` owner can mint this object and can decide who is
        /// going to be the recipient of the ticket. This allows users to run their
        /// own stop orders bots eventually, but it's mainly used to allow 3rd parties
        /// to offer such a service (the user is required to trust such 3rd party).
        /// The object is intended to be sent to a multisig wallet owned by
        /// both the 3rd party and the user. The object is not transferable, stopping
        /// the 3rd party from transferring it away, and can be destroyed in any moment
        /// only by the user. The user needs to trust the 3rd party for liveness of the
        /// service offered.
        ///
        /// The order details are encrypted offchain and the result is stored in the ticket.
        /// The user needs to share such details with the 3rd party only.
        struct StopOrderTicket<!phantom T> has key {
            id: UID,
            /// Save user address. This allow only the user to cancel the ticket eventually.
            user_address: address,
            /// Timestamp after which the order cannot be placed anymore
            expire_timestamp: u64,
            /// Vector containing the blake2b hash obtained by the offchain
            /// application of blake2b on the following parameters:
            /// - clearing_house_id: ID
            /// - account_id: u64
            /// - is_limit_order: `true` if limit order, `false` if market order
            /// - stop_index_price: u256
            /// - ge_stop_index_price: `true` means the order can be placed when
            /// oracle index price is >= than chosen `stop_index_price`
            /// - side: bool
            /// - size: u64
            /// - price: u64 (can be set at random value if `is_limit_order` is false)
            /// - order_type: u64 (can be set at random value if `is_limit_order` is false)
            /// - salt: vector<u8>
            encrypted_details: vector<u8>
        }
    }

    module admin {
        /// Capability object required to perform admin functions.
        ///
        /// Minted once when the module is published and transfered to its creator.
        struct AdminCapability has key, store {
            id: UID
        }
    }

    module clearing_house {
        /// Used to dynamically load market objects as needed.
        /// Used to dynamically load traders' position objects as needed.
        struct ClearingHouse<!phantom T> has key {
            id: UID,
            version: u64,
            market_params: market::MarketParams,
            market_state: market::MarketState
        }

        /// Stores all deposits from traders for collateral T.
        /// Stores the funds reserved for covering bad debt from untimely
        /// liquidations.
        ///
        /// The Clearing House keeps track of who owns each share of the vault.
        struct Vault<!phantom T> has store {
            collateral_balance: Balance<T>,
            insurance_fund_balance: Balance<T>,
            scaling_factor: IFixed
        }

        /// Stores the proposed parameters for updating margin ratios
        struct MarginRatioProposal has store {
            /// Target timestamp at which to apply the proposed updates
            maturity: u64,
            /// Proposed IMR
            margin_ratio_initial: IFixed,
            /// Proposed MMR
            margin_ratio_maintenance: IFixed,
        }

        /// Stores the proposed parameters for a position's custom fees
        struct PositionFeesProposal has store {
            /// Proposed IMR
            maker_fee: IFixed,
            /// Proposed MMR
            taker_fee: IFixed,
        }

        /// Used by clearing house to check margin when placing an order
        struct SessionHotPotato<!phantom T> {
            clearing_house: ClearingHouse<T>,
            account_id: u64,
            timestamp_ms: u64,
            collateral_price: IFixed,
            index_price: IFixed,
            book_price: IFixed,
            margin_before: IFixed,
            min_margin_before: IFixed,
            fills: vector<orderbook::FillReceipt>,
            post: orderbook::PostReceipt,
            liquidation_receipt: move_stdlib_sdk::option::Option<LiquidationReceipt>
        }

        struct LiquidationReceipt has drop, store {
            liqee_account_id: u64,
            size_to_liquidate: u64,
            base_ask_cancel: u64,
            base_bid_cancel: u64,
            pending_orders: u64
        }

        struct SessionSummary has drop {
            base_filled_ask: IFixed,
            base_filled_bid: IFixed,
            quote_filled_ask: IFixed,
            quote_filled_bid: IFixed,
            base_posted_ask: IFixed,
            base_posted_bid: IFixed,
            /// This would be the `mark_price` used in the eventuality the session contains a liquidation.
            /// Set at 0 in case there is no liquidation in the session.
            mark_price: IFixed,
            bad_debt: IFixed
        }
    }

    module subaccount {
        /// The SubAccount object represents an `Account` object with limited access to
        /// protocol's features. Being a shared object, it can only be used by the address
        /// specified in the `user` field.
        struct SubAccount<!phantom T> has key, store {
            id: UID,
            /// Address able to make calls using this `SubAccount`
            user: address,
            /// Numerical value associated to the parent account
            account_id: u64,
            /// Balance available to be allocated to markets.
            collateral: Balance<T>,
        }

    }

    module events {
        struct CreatedAccount<!phantom T> has copy, drop {
            user: address,
            account_id: u64
        }

        struct DepositedCollateral<!phantom T> has copy, drop {
            account_id: u64,
            collateral: u64,
            account_collateral_after: u64
        }

        struct AllocatedCollateral has copy, drop {
            ch_id: ID,
            account_id: u64,
            collateral: u64,
            account_collateral_after: u64,
            position_collateral_after: IFixed,
            vault_balance_after: u64
        }

        struct WithdrewCollateral<!phantom T> has copy, drop {
            account_id: u64,
            collateral: u64,
            account_collateral_after: u64
        }

        struct DeallocatedCollateral has copy, drop {
            ch_id: ID,
            account_id: u64,
            collateral: u64,
            account_collateral_after: u64,
            position_collateral_after: IFixed,
            vault_balance_after: u64
        }

        struct CreatedOrderbook has copy, drop {
            branch_min: u64,
            branches_merge_max: u64,
            branch_max: u64,
            leaf_min: u64,
            leaves_merge_max: u64,
            leaf_max: u64
        }

        struct CreatedClearingHouse has copy, drop {
            ch_id: ID,
            collateral: String,
            coin_decimals: u64,
            margin_ratio_initial: IFixed,
            margin_ratio_maintenance: IFixed,
            base_oracle_id: ID,
            collateral_oracle_id: ID,
            funding_frequency_ms: u64,
            funding_period_ms: u64,
            premium_twap_frequency_ms: u64,
            premium_twap_period_ms: u64,
            spread_twap_frequency_ms: u64,
            spread_twap_period_ms: u64,
            maker_fee: IFixed,
            taker_fee: IFixed,
            liquidation_fee: IFixed,
            force_cancel_fee: IFixed,
            insurance_fund_fee: IFixed,
            lot_size: u64,
            tick_size: u64,
        }

        struct RegisteredMarketInfo<!phantom T> has copy, drop {
            ch_id: ID,
            base_pfs_id: ID,
            collateral_pfs_id: ID,
            lot_size: u64,
            tick_size: u64,
            scaling_factor: IFixed
        }

        struct RemovedRegisteredMarketInfo<!phantom T> has copy, drop {
            ch_id: ID,
        }

        struct RegisteredCollateralInfo<!phantom T> has copy, drop {
            ch_id: ID,
            collateral_pfs_id: ID,
            scaling_factor: IFixed
        }

        struct UpdatedClearingHouseVersion has copy, drop {
            ch_id: ID,
            version: u64
        }

        struct UpdatedPremiumTwap has copy, drop {
            ch_id: ID,
            book_price: IFixed,
            index_price: IFixed,
            premium_twap: IFixed,
            premium_twap_last_upd_ms: u64,
        }

        struct UpdatedSpreadTwap has copy, drop {
            ch_id: ID,
            book_price: IFixed,
            index_price: IFixed,
            spread_twap: IFixed,
            spread_twap_last_upd_ms: u64,
        }

        struct UpdatedFunding has copy, drop {
            ch_id: ID,
            cum_funding_rate_long: IFixed,
            cum_funding_rate_short: IFixed,
            funding_last_upd_ms: u64,
        }

        struct SettledFunding has copy, drop {
            ch_id: ID,
            account_id: u64,
            collateral_change_usd: IFixed,
            collateral_after: IFixed,
            mkt_funding_rate_long: IFixed,
            mkt_funding_rate_short: IFixed
        }

        struct FilledMakerOrder has copy, drop {
            ch_id: ID,
            maker_account_id: u64,
            maker_collateral: IFixed,
            collateral_change_usd: IFixed,
            order_id: u128,
            maker_size: u64,
            maker_final_size: u64,
            maker_base_amount: IFixed,
            maker_quote_amount: IFixed,
            maker_pending_asks_quantity: IFixed,
            maker_pending_bids_quantity: IFixed,
        }

        struct FilledTakerOrder has copy, drop {
            ch_id: ID,
            taker_account_id: u64,
            taker_collateral: IFixed,
            collateral_change_usd: IFixed,
            base_asset_delta_ask: IFixed,
            quote_asset_delta_ask: IFixed,
            base_asset_delta_bid: IFixed,
            quote_asset_delta_bid: IFixed,
            taker_base_amount: IFixed,
            taker_quote_amount: IFixed,
            liquidated_volume: IFixed,
        }

        struct OrderbookPostReceipt has copy, drop {
            ch_id: ID,
            account_id: u64,
            order_id: u128,
            order_size: u64,
        }

        struct PostedOrder has copy, drop {
            ch_id: ID,
            account_id: u64,
            posted_base_ask: u64,
            posted_base_bid: u64,
            pending_asks: IFixed,
            pending_bids: IFixed,
            pending_orders: u64,
        }

        struct CanceledOrder has copy, drop {
            ch_id: ID,
            account_id: u64,
            size: u64,
            order_id: u128,
        }

        struct CanceledOrders has copy, drop {
            ch_id: ID,
            account_id: u64,
            asks_quantity: IFixed,
            bids_quantity: IFixed,
            pending_orders: u64,
        }

        struct LiquidatedPosition has copy, drop {
            ch_id: ID,
            liqee_account_id: u64,
            liqor_account_id: u64,
            is_liqee_long: bool,
            size_liquidated: u64,
            mark_price: IFixed,
            liqee_collateral_change_usd: IFixed,
            liqee_collateral: IFixed,
            liqee_base_amount: IFixed,
            liqee_quote_amount: IFixed,
            bad_debt: IFixed
        }

        struct UpdatedCumFundings has copy, drop {
            ch_id: ID,
            cum_funding_rate_long: IFixed,
            cum_funding_rate_short: IFixed,
        }

        struct CreatedPosition has copy, drop {
            ch_id: ID,
            account_id: u64,
            mkt_funding_rate_long: IFixed,
            mkt_funding_rate_short: IFixed,
        }

        struct CreatedStopOrderTicket has copy, drop {
            account_id: u64,
            recipient: address,
            encrypted_details: vector<u8>
        }

        struct DeletedStopOrderTicket has copy, drop {
            id: ID,
            user_address: address,
            processed: bool
        }

        struct CreatedMarginRatiosProposal has copy, drop {
            ch_id: ID,
            margin_ratio_initial: IFixed,
            margin_ratio_maintenance: IFixed,
        }

        struct UpdatedMarginRatios has copy, drop {
            ch_id: ID,
            margin_ratio_initial: IFixed,
            margin_ratio_maintenance: IFixed,
        }

        struct DeletedMarginRatiosProposal has copy, drop {
            ch_id: ID,
            margin_ratio_initial: IFixed,
            margin_ratio_maintenance: IFixed,
        }

        struct CreatedPositionFeesProposal has copy, drop {
            ch_id: ID,
            account_id: u64,
            maker_fee: IFixed,
            taker_fee: IFixed,
        }

        struct DeletedPositionFeesProposal has copy, drop {
            ch_id: ID,
            account_id: u64,
            maker_fee: IFixed,
            taker_fee: IFixed,
        }

        struct AcceptedPositionFeesProposal has copy, drop {
            ch_id: ID,
            account_id: u64,
            maker_fee: IFixed,
            taker_fee: IFixed,
        }

        struct RejectedPositionFeesProposal has copy, drop {
            ch_id: ID,
            account_id: u64,
            maker_fee: IFixed,
            taker_fee: IFixed,
        }

        struct ResettedPositionFees has copy, drop {
            ch_id: ID,
            account_id: u64,
        }

        struct UpdatedFees has copy, drop {
            ch_id: ID,
            maker_fee: IFixed,
            taker_fee: IFixed,
            liquidation_fee: IFixed,
            force_cancel_fee: IFixed,
            insurance_fund_fee: IFixed,
        }

        struct UpdatedFundingParameters has copy, drop {
            ch_id: ID,
            funding_frequency_ms: u64,
            funding_period_ms: u64,
            premium_twap_frequency_ms: u64,
            premium_twap_period_ms: u64,
        }

        struct UpdatedSpreadTwapParameters has copy, drop {
            ch_id: ID,
            spread_twap_frequency_ms: u64,
            spread_twap_period_ms: u64
        }

        struct UpdatedMinOrderUsdValue has copy, drop {
            ch_id: ID,
            min_order_usd_value: IFixed,
        }

        struct UpdatedLiquidationTolerance has copy, drop {
            ch_id: ID,
            liquidation_tolerance: u64,
        }

        struct UpdatedBaseOracleTolerance has copy, drop {
            ch_id: ID,
            oracle_tolerance: u64,
        }

        struct UpdatedCollateralOracleTolerance has copy, drop {
            ch_id: ID,
            oracle_tolerance: u64,
        }

        struct UpdatedMaxPendingOrders has copy, drop {
            ch_id: ID,
            max_pending_orders: u64
        }

        struct DonatedToInsuranceFund has copy, drop {
            sender: address,
            ch_id: ID,
            new_balance: u64,
        }

        struct WithdrewFees has copy, drop {
            sender: address,
            ch_id: ID,
            amount: u64,
            vault_balance_after: u64,
        }

        struct WithdrewInsuranceFund has copy, drop {
            sender: address,
            ch_id: ID,
            amount: u64,
            insurance_fund_balance_after: u64,
        }

        struct UpdatedOpenInterestAndFeesAccrued has copy, drop {
            ch_id: ID,
            open_interest: IFixed,
            fees_accrued: IFixed
        }

        struct CreatedSubAccount has copy, drop {
            subaccount_id: ID,
            user: address,
            account_id: u64
        }

        struct SetSubAccountUser has copy, drop {
            subaccount_id: ID,
            user: address,
            account_id: u64
        }

        struct DeletedSubAccount has copy, drop {
            subaccount_id: ID,
            account_id: u64
        }

        struct DepositedCollateralSubAccount has copy, drop {
            subaccount_id: ID,
            account_id: u64,
            collateral: u64,
            subaccount_collateral_after: u64
        }

        struct WithdrewCollateralSubAccount has copy, drop {
            subaccount_id: ID,
            account_id: u64,
            collateral: u64,
            subaccount_collateral_after: u64
        }

        struct AllocatedCollateralSubAccount has copy, drop {
            ch_id: ID,
            subaccount_id: ID,
            account_id: u64,
            collateral: u64,
            subaccount_collateral_after: u64,
            position_collateral_after: IFixed,
            vault_balance_after: u64
        }

        struct DeallocatedCollateralSubAccount has copy, drop {
            ch_id: ID,
            subaccount_id: ID,
            account_id: u64,
            collateral: u64,
            subaccount_collateral_after: u64,
            position_collateral_after: IFixed,
            vault_balance_after: u64
        }
    }

    module keys {
        /// Key type for accessing a `MarketInfo` saved in registry.
        struct RegistryMarketInfo has copy, drop, store {
            ch_id: ID
        }

        /// Key type for accessing a `CollateralInfo` saved in registry.
        struct RegistryCollateralInfo<!phantom T> has copy, drop, store {}

        /// Key type for accessing market params in clearing house.
        struct Orderbook has copy, drop, store {}

        /// Key type for accessing vault in clearing house.
        struct MarketVault has copy, drop, store {}

        /// Key type for accessing trader position in clearing house.
        struct Position has copy, drop, store {
            account_id: u64,
        }

        /// Key type for accessing market margin parameters change proposal in clearing house.
        struct MarginRatioProposal has copy, drop, store {}

        /// Key type for accessing custom fees parameters change proposal for an account
        struct PositionFeesProposal has copy, drop, store {
            account_id: u64
        }

        /// Key type for accessing asks map in the orderbook
        struct AsksMap has copy, drop, store {}

        /// Key type for accessing asks map in the orderbook
        struct BidsMap has copy, drop, store {}
    }

    module market {
        /// Static attributes of a perpetuals market.
        struct MarketParams has copy, drop, store {
            /// Minimum margin ratio for opening a new position.
            margin_ratio_initial: IFixed,
            /// Margin ratio below which full liquidations can occur.
            margin_ratio_maintenance: IFixed,
            /// Identifier of the base asset's price feed storage.
            base_pfs_id: ID,
            /// Identifier of the collateral asset's price feed storage.
            collateral_pfs_id: ID,
            /// The time span between each funding rate update.
            funding_frequency_ms: u64,
            /// Period of time over which funding (the difference between book and
            /// index prices) gets paid.
            ///
            /// Setting the funding period too long may cause the perpetual to start
            /// trading at a very dislocated price to the index because there's less
            /// of an incentive for basis arbitrageurs to push the prices back in
            /// line since they would have to carry the basis risk for a longer
            /// period of time.
            ///
            /// Setting the funding period too short may cause nobody to trade the
            /// perpetual because there's too punitive of a price to pay in the case
            /// the funding rate flips sign.
            funding_period_ms: u64,
            /// The time span between each funding TWAP (both index price and orderbook price) update.
            premium_twap_frequency_ms: u64,
            /// The reference time span used for weighting the TWAP (both index price and orderbook price)
            /// updates for funding rates estimation
            premium_twap_period_ms: u64,
            /// The time span between each spread TWAP updates (used for liquidations).
            spread_twap_frequency_ms: u64,
            /// The reference time span used for weighting the TWAP updates for spread.
            spread_twap_period_ms: u64,
            /// Proportion of volume charged as fees from makers upon processing
            /// fill events.
            maker_fee: IFixed,
            /// Proportion of volume charged as fees from takers after processing
            /// fill events.
            taker_fee: IFixed,
            /// Proportion of volume charged as fees from liquidatees
            liquidation_fee: IFixed,
            /// Proportion of volume charged as fees from liquidatees after forced cancelling
            /// of pending orders during liquidation.
            force_cancel_fee: IFixed,
            /// Proportion of volume charged as fees from liquidatees to deposit into insurance fund
            insurance_fund_fee: IFixed,
            /// Minimum USD value an order is required to be worth to be placed
            min_order_usd_value: IFixed,
            /// Number of base units exchanged per lot
            lot_size: u64,
            /// Number of quote units exchanged per tick
            tick_size: u64,
            /// Number of lots in a position that a liquidator may buy in excess of what would be
            /// strictly required to bring the liqee's account back to IMR.
            liquidation_tolerance: u64,
            /// Maximum number of pending orders that a position can have.
            max_pending_orders: u64,
            /// Timestamp tolerance for base oracle price
            base_oracle_tolerance: u64,
            /// Timestamp tolerance for collateral oracle price
            collateral_oracle_tolerance: u64
        }

        /// The state of a perpetuals market.
        struct MarketState has store {
            /// The latest cumulative funding premium in this market for longs. Must be updated
            /// periodically.
            cum_funding_rate_long: IFixed,
            /// The latest cumulative funding premium in this market for shorts. Must be updated
            /// periodically.
            cum_funding_rate_short: IFixed,
            /// The timestamp (millisec) of the latest cumulative funding premium update
            /// (both longs and shorts).
            funding_last_upd_ms: u64,
            /// The last calculated funding premium TWAP (used for funding settlement).
            premium_twap: IFixed,
            /// The timestamp (millisec) of the last update of `premium_twap`.
            premium_twap_last_upd_ms: u64,
            /// The last calculated spread TWAP (used for liquidations).
            /// Spread is (book - index).
            spread_twap: IFixed,
            /// The timestamp (millisec) of `spread_twap` last update.
            spread_twap_last_upd_ms: u64,
            /// Open interest (in base tokens) as a fixed-point number. Counts the
            /// total size of contracts as the sum of all long positions.
            open_interest: IFixed,
            /// Total amount of fees accrued by this market (in T's units)
            /// Only admin can withdraw these fees.
            fees_accrued: IFixed,
        }
    }

    module orderbook {
        /// An order on the orderbook
        struct Order has copy, drop, store {
            /// User's account id
            account_id: u64,
            /// Amount of lots to be filled
            size: u64
        }

        /// The orderbook doesn't know the types of tokens traded, it assumes a correct
        /// management by the clearing house
        struct Orderbook has key, store {
            id: UID,
            /// Number of limit orders placed on book, monotonically increases
            counter: u64,
        }

        // -----------------------------------------------------------------------------
        //        Result Structures
        // -----------------------------------------------------------------------------

        struct FillReceipt has drop, store {
            account_id: u64,
            order_id: u128,
            size: u64,
            final_size: u64,
        }

        struct PostReceipt has drop, store {
            base_ask: u64,
            base_bid: u64,
            pending_orders: u64
        }

        /// Order info data structure that is returned by `inspect_orders` function.
        struct OrderInfo has copy, drop, store {
            price: u64,
            size: u64,
        }
    }

    module ordered_map {
        /// Ordered map with `u128` type as a key and `V` type as a value.
        struct Map<!phantom V: copy + drop + store> has key, store {
            /// Object UID for adding dynamic fields that are used as pointers to nodes.
            id: UID,
            /// Number of key-value pairs in the map.
            size: u64,
            /// Counter for creating another node as a dynamic field.
            counter: u64,
            /// Pointer to the root node, which is a branch or a leaf.
            root: u64,
            /// Pointer to first leaf.
            first: u64,
            /// Minimal number of kids in a non-root branch;
            /// must satisfy 2 <= branch_min <= branch_max / 2.
            branch_min: u64,
            /// Maximal number of kids in a branch, which is merge of two branches;
            /// must satisfy 2 * branch_min <= branches_merge_max <= branch_max.
            branches_merge_max: u64,
            /// Maximal number of kids in a branch.
            branch_max: u64,
            /// Minimal number of elements in a non-root leaf;
            /// must satisfy 2 <= leaf_min <= (leaf_max + 1) / 2.
            leaf_min: u64,
            /// Maximal number of elements in a leaf, which is merge of two leaves;
            /// must satisfy 2 * leaf_min - 1 <= leaves_merge_max <= leaf_max.
            leaves_merge_max: u64,
            /// Maximal number of elements in a leaf.
            leaf_max: u64,
        }

        /// Branch node with kids and ordered separating keys.
        struct Branch has drop, store {
            /// Separating keys for kids sorted in ascending order.
            keys: vector<u128>,
            /// Kids of the node.
            kids: vector<u64>,
        }

        /// Key-value pair.
        struct Pair<V: copy + drop + store> has copy, drop, store {
            key: u128,
            val: V,
        }

        /// Leaf node with ordered key-value pairs.
        struct Leaf<V: copy + drop + store> has drop, store {
            /// Keys sorted in ascending order together with values.
            keys_vals: vector<Pair<V>>,
            /// Pointer to next leaf.
            next: u64,
        }
    }

    module position {
        /// Stores information about an open position
        struct Position has store {
            /// Amount of allocated tokens (e.g., USD stables) backing this account's position.
            collateral: IFixed,
            /// The perpetual contract size, controlling the amount of exposure to
            /// the underlying asset. Positive implies long position and negative,
            /// short. Represented as a signed fixed-point number.
            base_asset_amount: IFixed,
            /// The entry value for this position, including leverage. Represented
            /// as a signed fixed-point number.
            quote_asset_notional_amount: IFixed,
            /// Last long cumulative funding rate used to update this position. The
            /// market's latest long cumulative funding rate minus this gives the funding
            /// rate this position must pay. This rate multiplied by this position's
            /// value (base asset amount * market price) gives the total funding
            /// owed, which is deducted from the trader account's margin. This debt
            /// is accounted for in margin ratio calculations, which may lead to
            /// liquidation. Represented as a signed fixed-point number.
            cum_funding_rate_long: IFixed,
            /// Last short cumulative funding rate used to update this position. The
            /// market's latest short cumulative funding rate minus this gives the funding
            /// rate this position must pay. This rate multiplied by this position's
            /// value (base asset amount * market price) gives the total funding
            /// owed, which is deducted from the trader account's margin. This debt
            /// is accounted for in margin ratio calculations, which may lead to
            /// liquidation. Represented as a signed fixed-point number.
            cum_funding_rate_short: IFixed,
            /// Base asset amount resting in ask orders in the orderbook.
            /// Represented as a signed fixed-point number.
            asks_quantity: IFixed,
            /// Base asset amount resting in bid orders in the orderbook.
            /// Represented as a signed fixed-point number.
            bids_quantity: IFixed,
            /// Number of pending orders in this position.
            pending_orders: u64,
            /// Custom maker fee for this position, set at default value of 100%
            maker_fee: IFixed,
            /// Custom taker fee for this position, set at default value of 100%
            taker_fee: IFixed,
        }
    }

    module registry {
        /// Registry object that maintains:
        /// - A mapping between a clearing house id and `MarketInfo`
        /// - A mapping between a collateral type `T` and `CollateralInfo`
        /// It also maintains the global counter for account creation.
        /// Minted and shared when the module is published.
        struct Registry has key {
            id: UID,
            next_account_id: u64
        }

        /// Struct containing all the immutable info about a registered market
        struct MarketInfo<!phantom T> has store {
            base_pfs_id: ID,
            collateral_pfs_id: ID,
            lot_size: u64,
            tick_size: u64,
            scaling_factor: IFixed
        }

        /// Struct containing all the immutable info about the collateral
        /// used in one or more markets
        struct CollateralInfo<!phantom T> has store {
            collateral_pfs_id: ID,
            scaling_factor: IFixed
        }
    }
});

impl<T: af_move_type::MoveType> clearing_house::ClearingHouse<T> {
    /// The ID of the package that governs this clearing house's logic.
    ///
    /// This may be different than the package defining the clearing house's type because a package
    /// upgrade + `interface::upgrade_clearing_house_version` call can change
    /// [`ClearingHouse::version`] so that the upgraded package is the one that is allowed to make
    /// changes to it.
    ///
    /// Attempting to make a PTB Move call that mutates this clearing house but is not defined in
    /// this package version will fail.
    pub const fn governing_package_testnet(&self) -> ObjectId {
        // NOTE: we published the most recent testnet contracts starting with `VERSION = 1`
        TESTNET_PACKAGE_VERSIONS[self.version as usize - 1]
    }
}
