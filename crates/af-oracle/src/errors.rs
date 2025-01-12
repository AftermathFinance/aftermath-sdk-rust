//! Error codes for Move aborts.
#![expect(non_upper_case_globals, reason = "Copied from Move")]

pub const PriceFeedAlreadyExists: u64 = 1;
pub const PriceFeedDoesNotExist: u64 = 2;
pub const InvalidSourceObjectForFeed: u64 = 3;
pub const InvalidPriceValue: u64 = 5;
pub const SourceAlreadyAuthorized: u64 = 6;
pub const SourceNotAuthorized: u64 = 7;
pub const SourceObjectIsNotRegistered: u64 = 8;
pub const SymbolDoesNotExists: u64 = 9;
pub const PriceIsTooHigh: u64 = 10;
pub const SourceTimestampGreaterThanCurrentClockTimestamp: u64 = 11;
pub const SourceTimestampOlderThanCurrentFeedTimestamp: u64 = 12;
pub const SourceTimestampOlderThanTolerance: u64 = 13;
