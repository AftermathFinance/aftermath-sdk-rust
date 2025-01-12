#![expect(non_upper_case_globals, reason = "Copied from Move")]

// ClearingHouse ---------------------------------------------------------------

/// Cannot deposit/withdraw zero coins to/from the account's collateral.
pub const DepositOrWithdrawAmountZero: u64 = 0;
/// Orderbook size or price are invalid values
pub const InvalidSizeOrPrice: u64 = 1;
/// Index price returned from oracle is 0 or invalid value
pub const BadIndexPrice: u64 = 2;
/// Order value in USD is too low
pub const OrderUsdValueTooLow: u64 = 4;
/// Passed a vector of invalid order ids to perform force cancellation
/// during liquidation
pub const InvalidForceCancelIds: u64 = 5;
/// Liquidate must be the first operation of the session, if performed.
pub const LiquidateNotFirstOperation: u64 = 6;
/// Passed a vector of invalid order ids to cancel
pub const InvalidCancelOrderIds: u64 = 7;
/// Ticket has already passed `expire_timestamp` and can only be cancelled
pub const StopOrderTicketExpired: u64 = 8;
/// Index price is not at correct value to satisfy stop order conditions
pub const StopOrderConditionsViolated: u64 = 9;
/// Index price is not at correct value to satisfy stop order conditions
pub const WrongOrderDetails: u64 = 10;
/// Invalid base price feed storage for the clearing house
pub const InvalidBasePriceFeedStorage: u64 = 11;
/// Same liquidator and liqee account ids
pub const SelfLiquidation: u64 = 12;
/// User trying to access the subaccount is not the one specified by parent
pub const InvalidSubAccountUser: u64 = 13;
/// The parent `Account` trying to delete the subaccount is not the correct one.
pub const WrongParentForSubAccount: u64 = 14;
/// Raised when trying to delete a subaccount still containing collateral.
pub const SubAccountContainsCollateral: u64 = 15;
/// Raised when trying to call a function with the wrong package's version
pub const WrongVersion: u64 = 16;
/// Raised when trying to have a session composed by only `start_session` and `end_session`
pub const EmptySession: u64 = 17;
/// Market already registered in the registry
pub const MarketAlreadyRegistered: u64 = 18;
/// Collateral is not registered in the registry
pub const CollateralIsNotRegistered: u64 = 19;
/// Market is not registered in the registry
pub const MarketIsNotRegistered: u64 = 20;
/// Invalid collateral price feed storage for the clearing house
pub const InvalidCollateralPriceFeedStorage: u64 = 21;

// Market ---------------------------------------------------------------

/// While creating ordered map with invalid parameters,
/// or changing them improperly for an existent map.
pub const InvalidMarketParameters: u64 = 1000;
/// Tried to call `update_funding` before enough time has passed since the
/// last update.
pub const UpdatingFundingTooEarly: u64 = 1001;
/// Margin ratio update proposal already exists for market
pub const ProposalAlreadyExists: u64 = 1002;
/// Margin ratio update proposal cannot be commited too early
pub const PrematureProposal: u64 = 1003;
/// Margin ratio update proposal delay is outside the valid range
pub const InvalidProposalDelay: u64 = 1004;
/// Margin ratio update proposal does not exist for market
pub const ProposalDoesNotExist: u64 = 1005;
/// Exchange has no available fees to withdraw
pub const NoFeesAccrued: u64 = 1006;
/// Tried to withdraw more insurance funds than the allowed amount
pub const InsufficientInsuranceSurplus: u64 = 1007;
/// Cannot create a market for which a price feed does not exist
pub const NoPriceFeedForMarket: u64 = 1008;
/// Cannot delete a proposal that already matured. It can only be committed.
pub const ProposalAlreadyMatured: u64 = 1009;

// Position  ---------------------------------------------------------------

/// Tried placing a new pending order when the position already has the maximum
/// allowed number of pending orders.
pub const MaxPendingOrdersExceeded: u64 = 2000;
/// Used for checking both liqee and liqor positions during liquidation
pub const PositionBelowIMR: u64 = 2001;
/// When leaving liqee's position with a margin ratio above tolerance,
/// meaning that liqor has overbought position
pub const PositionAboveTolerance: u64 = 2002;
/// An operation brought an account below initial margin requirements.
pub const InitialMarginRequirementViolated: u64 = 2003;
/// Position is above MMR, so can't be liquidated.
pub const PositionAboveMMR: u64 = 2004;
/// Cannot realize bad debt via means other than calling 'liquidate'.
pub const PositionBadDebt: u64 = 2005;
/// Cannot withdraw more than the account's free collateral.
pub const InsufficientFreeCollateral: u64 = 2006;
/// Cannot have more than 1 position in a market.
pub const PositionAlreadyExists: u64 = 2007;
/// Cannot compute deallocate amount for a target MR < IMR.
pub const DeallocateTargetMrTooLow: u64 = 2008;

// Orderbook & OrderedMap -------------------------------------------------------

/// While creating ordered map with wrong parameters.
pub const InvalidMapParameters: u64 = 3000;
/// While searching for a key, but it doesn't exist.
pub const KeyNotExist: u64 = 3001;
/// While inserting already existing key.
pub const KeyAlreadyExists: u64 = 3002;
/// When attempting to destroy a non-empty map
pub const DestroyNotEmpty: u64 = 3003;
/// Invalid user tries to modify an order
pub const InvalidUserForOrder: u64 = 3004;
/// Orderbook flag requirements violated
pub const FlagRequirementsViolated: u64 = 3005;
/// Minimum size matched not reached
pub const NotEnoughLiquidity: u64 = 3006;
/// When trying to change a map configuration, but the map has
/// length less than 4
pub const MapTooSmall: u64 = 3007;
/// When taker matches its own order
pub const SelfTrading: u64 = 3008;
