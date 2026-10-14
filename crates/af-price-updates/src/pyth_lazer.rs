//! Pyth Lazer `leEcdsa` price update format and feed identifiers.

pub use crate::pyth_lazer_id_set::PriceFeedIdSet;
use crate::{
    ParseError,
    reader::{Reader, decode_base64},
    sorted_set::SortedSetSlice,
};
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

/// Pyth Lazer price feed ID, e.g. `1` for BTC/USD.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[repr(transparent)]
pub struct PriceFeedId(pub u32);

impl std::fmt::Display for PriceFeedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <u32 as std::fmt::Display>::fmt(&self.0, f)
    }
}

impl std::str::FromStr for PriceFeedId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(PriceFeedId)
    }
}

/// A property of a Pyth Lazer feed, requested when subscribing.
///
/// Serialized, displayed and parsed by its Pyth Lazer name, e.g. `bestBidPrice`. See [`Feed`] for
/// the meaning of each property.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum FeedProperty {
    /// See [`Feed::price`].
    Price,
    /// See [`Feed::best_bid_price`].
    BestBidPrice,
    /// See [`Feed::best_ask_price`].
    BestAskPrice,
    /// See [`Feed::publisher_count`].
    PublisherCount,
    /// See [`Feed::exponent`].
    Exponent,
    /// See [`Feed::confidence`].
    Confidence,
    /// See [`Feed::funding_rate`].
    FundingRate,
    /// See [`Feed::funding_timestamp`].
    FundingTimestamp,
    /// See [`Feed::funding_rate_interval`].
    FundingRateInterval,
    /// See [`Feed::market_session`].
    MarketSession,
    /// See [`Feed::ema_price`].
    EmaPrice,
    /// See [`Feed::ema_confidence`].
    EmaConfidence,
    /// See [`Feed::feed_update_timestamp`].
    FeedUpdateTimestamp,
}

impl FeedProperty {
    /// The Pyth Lazer name, as used by serde.
    const fn name(self) -> &'static str {
        match self {
            Self::Price => "price",
            Self::BestBidPrice => "bestBidPrice",
            Self::BestAskPrice => "bestAskPrice",
            Self::PublisherCount => "publisherCount",
            Self::Exponent => "exponent",
            Self::Confidence => "confidence",
            Self::FundingRate => "fundingRate",
            Self::FundingTimestamp => "fundingTimestamp",
            Self::FundingRateInterval => "fundingRateInterval",
            Self::MarketSession => "marketSession",
            Self::EmaPrice => "emaPrice",
            Self::EmaConfidence => "emaConfidence",
            Self::FeedUpdateTimestamp => "feedUpdateTimestamp",
        }
    }
}

impl std::fmt::Display for FeedProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for FeedProperty {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ALL_FEED_PROPERTIES
            .iter()
            .copied()
            .find(|property| property.name() == s)
            .ok_or_else(|| ParseError::new(format!("unknown feed property: {s}")))
    }
}

/// Every [`FeedProperty`], in the order of their Pyth Lazer property IDs.
pub const ALL_FEED_PROPERTIES: &[FeedProperty] = &[
    FeedProperty::Price,
    FeedProperty::BestBidPrice,
    FeedProperty::BestAskPrice,
    FeedProperty::PublisherCount,
    FeedProperty::Exponent,
    FeedProperty::Confidence,
    FeedProperty::FundingRate,
    FeedProperty::FundingTimestamp,
    FeedProperty::FundingRateInterval,
    FeedProperty::MarketSession,
    FeedProperty::EmaPrice,
    FeedProperty::EmaConfidence,
    FeedProperty::FeedUpdateTimestamp,
];

/// The rate at which Pyth Lazer sends updates.
///
/// Serialized, displayed and parsed by its Pyth Lazer name, e.g. `fixed_rate@200ms`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[repr(u8)]
#[non_exhaustive]
pub enum Channel {
    /// As soon as a price changes.
    #[serde(rename = "real_time")]
    RealTime = 1,
    /// Every 50 milliseconds.
    #[serde(rename = "fixed_rate@50ms")]
    FixedRate50ms = 2,
    /// Every 200 milliseconds.
    #[serde(rename = "fixed_rate@200ms")]
    FixedRate200ms = 3,
    /// Every second.
    #[serde(rename = "fixed_rate@1000ms")]
    FixedRate1000ms = 4,
}

impl TryFrom<u8> for Channel {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::RealTime),
            2 => Ok(Self::FixedRate50ms),
            3 => Ok(Self::FixedRate200ms),
            4 => Ok(Self::FixedRate1000ms),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Pyth Lazer unknown channel value: {value}"),
            )),
        }
    }
}

impl std::str::FromStr for Channel {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "real_time" => Ok(Self::RealTime),
            "fixed_rate@50ms" => Ok(Self::FixedRate50ms),
            "fixed_rate@200ms" => Ok(Self::FixedRate200ms),
            "fixed_rate@1000ms" => Ok(Self::FixedRate1000ms),
            _ => Err(ParseError::new(format!("unknown channel: {s}"))),
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RealTime => f.write_str("real_time"),
            Self::FixedRate50ms => f.write_str("fixed_rate@50ms"),
            Self::FixedRate200ms => f.write_str("fixed_rate@200ms"),
            Self::FixedRate1000ms => f.write_str("fixed_rate@1000ms"),
        }
    }
}

/// The trading session of a feed's market, as reported by Pyth Lazer and Aftermath.
///
/// Converts to and from its `u16` wire value. Like the on-chain `af_verifier`, parsing accepts
/// values without a named variant, such as sessions added later, as [`Self::Unknown`] instead of
/// rejecting the whole update.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MarketSession {
    /// Regular trading hours.
    #[default]
    Regular,
    /// Before regular trading hours.
    PreMarket,
    /// After regular trading hours.
    PostMarket,
    /// Overnight trading.
    OverNight,
    /// The market is closed.
    Closed,
    /// A wire value without a named variant.
    ///
    /// Converting from a `u16` never wraps a value that has a named variant, so `Unknown(4)` is
    /// not equal to [`Self::Closed`]; build sessions from wire values with `From<u16>`.
    Unknown(u16),
}

impl From<u16> for MarketSession {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Regular,
            1 => Self::PreMarket,
            2 => Self::PostMarket,
            3 => Self::OverNight,
            4 => Self::Closed,
            _ => Self::Unknown(value),
        }
    }
}

impl From<MarketSession> for u16 {
    fn from(session: MarketSession) -> Self {
        match session {
            MarketSession::Regular => 0,
            MarketSession::PreMarket => 1,
            MarketSession::PostMarket => 2,
            MarketSession::OverNight => 3,
            MarketSession::Closed => 4,
            MarketSession::Unknown(value) => value,
        }
    }
}

/// A Pyth Lazer update in the `leEcdsa` format: a signed payload of feeds.
///
/// Constructors check the layout only and never verify the signature, which is left to the
/// on-chain Pyth Lazer verifier.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PriceUpdates(Bytes);

impl TryFrom<BytesMut> for PriceUpdates {
    type Error = Error;
    /// Copies the bytes, like [`PriceUpdates::from_slice`], so the update does not keep a larger
    /// buffer that `bytes` may share alive.
    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        Self::from_slice(&bytes)
    }
}

impl PriceUpdates {
    /// Checks and copies an update.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        check_bytes(bytes)?;
        Ok(Self(Bytes::copy_from_slice(bytes)))
    }
    /// Checks an update and keeps `bytes` without copying.
    ///
    /// Use [`Self::from_slice`] instead to copy the update out of a larger buffer.
    pub fn from_bytes(bytes: Bytes) -> Result<Self, Error> {
        check_bytes(&bytes)?;
        Ok(Self(bytes))
    }
    /// Decodes a standard, padded base64 update and checks it.
    ///
    /// Decodes in place when `base64` is the only owner of its buffer, such as a `String`, a
    /// `Vec<u8>` or the bytes of an unshared `bytestring::ByteString`, and copies it otherwise.
    pub fn from_base64(base64: impl Into<Bytes>) -> Result<Self, Error> {
        Self::from_bytes(decode_base64(base64.into(), "Pyth Lazer")?)
    }
    /// The whole update, including its signature.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    /// The whole update, including its signature, to share without copying.
    pub const fn bytes(&self) -> &Bytes {
        &self.0
    }
    /// The whole update, including its signature, without copying.
    pub fn into_bytes(self) -> Bytes {
        self.0
    }
    /// The recoverable secp256k1 signature over [`Self::payload`].
    pub fn signature(&self) -> &[u8; SIGNATURE_LEN] {
        // Skip magic.
        self.as_bytes()[4..]
            .first_chunk()
            .expect("checked on construction")
    }
    /// The signed payload, without its length prefix.
    pub fn payload(&self) -> &[u8] {
        // Skip magic, signature, payload length.
        &self.as_bytes()[4 + SIGNATURE_LEN + 2..]
    }
    /// Timestamp of the update in microseconds.
    pub fn timestamp(&self) -> u64 {
        let mut reader = Reader(self.as_bytes());
        // Skip magic, signature, payload length, payload magic.
        reader
            .skip(4 + SIGNATURE_LEN + 2 + 4)
            .expect("checked on construction");
        reader.take_u64().expect("checked on construction")
    }
    /// The channel the update was sent on.
    pub fn channel(&self) -> Channel {
        // Skip magic, signature, payload length, payload magic, timestamp.
        self.as_bytes()[4 + SIGNATURE_LEN + 2 + 4 + 8]
            .try_into()
            .expect("checked on construction")
    }
    /// Number of feeds in the update.
    pub fn feed_count(&self) -> usize {
        // Skip magic, signature, payload length, payload magic, timestamp, channel.
        usize::from(self.as_bytes()[4 + SIGNATURE_LEN + 2 + 4 + 8 + 1])
    }
    /// Feeds of the update, in the order sent.
    pub fn feeds(&self) -> impl Clone + ExactSizeIterator<Item = Feed> {
        let mut reader = Reader(self.as_bytes());
        // Skip magic, signature, payload length, payload magic, timestamp, channel.
        reader
            .skip(4 + SIGNATURE_LEN + 2 + 4 + 8 + 1)
            .expect("checked on construction");
        let feed_count = usize::from(reader.take_u8().expect("checked on construction"));
        (0..feed_count).map(move |_| take_feed(&mut reader).expect("checked on construction"))
    }
    /// The first feed with `feed_id`, if any.
    pub fn feed(&self, feed_id: PriceFeedId) -> Option<Feed> {
        self.feeds().find(|feed| feed.feed_id == feed_id)
    }
    /// Returns whether the update contains every ID in `feed_ids` exactly once and no other IDs.
    ///
    /// Pass a reference to a [`PriceFeedIdSet`], [`SortedSet`](crate::sorted_set::SortedSet), or
    /// [`ArcSortedSet`](crate::sorted_set::ArcSortedSet); deref coercion borrows its
    /// [`SortedSetSlice`] view. Feed order does not matter. This check does not allocate: it uses
    /// the sorted indices of `feed_ids` and a fixed-size stack bitmap to detect repeated IDs.
    pub fn matches_set(&self, feed_ids: &SortedSetSlice<PriceFeedId>) -> bool {
        feed_ids.exactly_matches_bounded_iter(self.feeds().map(|feed| feed.feed_id))
    }
}

/// A parsed Pyth Lazer feed.
///
/// A field is `None` when its property was **not requested**, or when Pyth Lazer reports it as
/// **unavailable**, e.g. a price without enough publishers. Pyth Lazer encodes an unavailable
/// price, best bid or ask price, confidence, EMA price or EMA confidence as zero, so, as in the
/// on-chain Pyth Lazer parser, those fields are never `Some(0)`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Feed {
    /// ID of the feed.
    pub feed_id: PriceFeedId,
    /// Aggregate market price (mantissa). Use with exponent for actual price.
    pub price: Option<i64>,
    /// Highest bid across publishers.
    pub best_bid_price: Option<i64>,
    /// Lowest ask across publishers.
    pub best_ask_price: Option<i64>,
    /// Number of contributing publishers.
    pub publisher_count: Option<u16>,
    /// Decimal exponent: `actual_price = mantissa * 10^exponent`.
    pub exponent: Option<i16>,
    /// Price uncertainty; higher = more disagreement.
    pub confidence: Option<i64>,
    /// Funding rate (perpetual futures).
    pub funding_rate: Option<i64>,
    /// When funding rate was last calculated (μs).
    pub funding_timestamp: Option<u64>,
    /// Interval between funding updates (μs).
    pub funding_rate_interval: Option<u64>,
    /// Session: regular, preMarket, postMarket, overNight, closed.
    pub market_session: Option<MarketSession>,
    /// Exponential moving average of the price.
    pub ema_price: Option<i64>,
    /// Exponential moving average of the confidence.
    pub ema_confidence: Option<u64>,
    /// Timestamp when price was last generated for this feed (μs).
    pub feed_update_timestamp: Option<u64>,
}

impl Feed {
    /// Returns `true` when the market session is reported as [`MarketSession::Closed`].
    pub fn is_market_closed(&self) -> bool {
        self.market_session == Some(MarketSession::Closed)
    }
}

// Binary WebSocket frame magic (outer wrapper around the leEcdsa message).
const BINARY_FRAME_MAGIC: u32 = 461_928_307;

/// Parses a binary WebSocket frame received from Pyth Lazer when using
/// `formats: ["leEcdsa"]` + `deliveryFormat: "binary"`.
///
/// The frame layout is:
/// - 4 bytes LE: `BINARY_FRAME_MAGIC`
/// - 8 bytes BE: `subscription_id`
/// - 2 bytes BE: message length
/// - message bytes: leEcdsa-signed `PriceUpdates`
///
/// Returns the subscription ID and the update, which shares `data` without copying.
pub fn parse_binary_answer(data: Bytes) -> Result<(u64, PriceUpdates), Error> {
    let mut reader = Reader(&data);
    if reader.take_u32()? != BINARY_FRAME_MAGIC {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Pyth Lazer invalid binary frame magic",
        ));
    }
    let subscription_id = u64::from_be_bytes(reader.take_chunk()?);
    // Message length is 2 bytes BE.
    let msg_len = u16::from_be_bytes(reader.take_chunk()?) as usize;
    let msg_start = data.len() - reader.0.len();
    reader.skip(msg_len)?;
    reader.assert_eof()?;
    let updates = PriceUpdates::from_bytes(data.slice(msg_start..msg_start + msg_len))?;
    Ok((subscription_id, updates))
}

const LE_ECDSA_FORMAT_MAGIC: u32 = 1_296_547_300;
/// Length of the recoverable secp256k1 signature after the format magic.
const SIGNATURE_LEN: usize = 65;
const PAYLOAD_FORMAT_MAGIC: u32 = 2_479_346_549;

fn take_feed(reader: &mut Reader<'_>) -> Result<Feed, Error> {
    let feed_id = PriceFeedId(reader.take_u32()?);
    let mut feed = Feed {
        feed_id,
        ..Default::default()
    };
    let property_count = reader.take_u8()?;
    for _ in 0..property_count {
        let property_id = reader.take_u8()?;
        match property_id {
            0 => feed.price = reader.take_nonzero_i64()?,
            1 => feed.best_bid_price = reader.take_nonzero_i64()?,
            2 => feed.best_ask_price = reader.take_nonzero_i64()?,
            3 => feed.publisher_count = Some(reader.take_u16()?),
            4 => feed.exponent = Some(reader.take_i16()?),
            5 => feed.confidence = reader.take_nonzero_i64()?,
            6 => feed.funding_rate = reader.take_option_u64()?.map(u64::cast_signed),
            7 => feed.funding_timestamp = reader.take_option_u64()?,
            8 => feed.funding_rate_interval = reader.take_option_u64()?,
            9 => feed.market_session = Some(MarketSession::from(reader.take_u16()?)),
            10 => feed.ema_price = reader.take_nonzero_i64()?,
            11 => feed.ema_confidence = reader.take_nonzero_u64()?,
            12 => feed.feed_update_timestamp = reader.take_option_u64()?,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Pyth Lazer invalid property ID: {property_id}"),
                ));
            }
        }
    }
    Ok(feed)
}

fn check_bytes(bytes: &[u8]) -> Result<(), Error> {
    let mut reader = Reader(bytes);
    if reader.take_u32()? != LE_ECDSA_FORMAT_MAGIC {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Pyth Lazer invalid format magic number",
        ));
    }
    let _signature = reader.take(SIGNATURE_LEN)?;
    let payload_len = reader.take_u16()?;
    reader.assert_remain(payload_len as usize)?;

    if reader.take_u32()? != PAYLOAD_FORMAT_MAGIC {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Pyth Lazer invalid payload magic number",
        ));
    }
    let _timestamp = reader.take_u64()?;
    let _channel = Channel::try_from(reader.take_u8()?)?;
    let feed_count = reader.take_u8()?;
    for _ in 0..feed_count {
        let _feed = take_feed(&mut reader)?;
    }
    reader.assert_eof()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BTC_USD: PriceFeedId = PriceFeedId(1);
    const ETH_USD: PriceFeedId = PriceFeedId(2);
    const USDC_USD: PriceFeedId = PriceFeedId(7);

    fn price_updates_with_feed_ids(ids: &[PriceFeedId]) -> PriceUpdates {
        let mut payload = Vec::new();
        payload.extend_from_slice(&PAYLOAD_FORMAT_MAGIC.to_le_bytes());
        payload.extend_from_slice(&0_u64.to_le_bytes());
        payload.push(Channel::RealTime as u8);
        payload.push(ids.len().try_into().unwrap());
        for id in ids {
            payload.extend_from_slice(&id.0.to_le_bytes());
            payload.push(0); // No properties.
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&LE_ECDSA_FORMAT_MAGIC.to_le_bytes());
        bytes.extend_from_slice(&[0; 65]);
        bytes.extend_from_slice(&u16::try_from(payload.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(&payload);
        PriceUpdates::from_slice(&bytes).unwrap()
    }

    const PYTH_LAZER_PRICE_UPDATES_BASE64: &str = "5L1HTSKo39Jq5IVP8R18EHxhJW/6aUm+sw06RHxZTx8WOrd8bIALgZi8Sk7GAzaHeP6UiN3Ew0cSVjHDPbGP3XE5hAgBtgB108eTAG0ZPF5MBgADAgEAAAANAGGlUjI5BgAAAYwFYS45BgAAAnoXaDk5BgAAAxIABPj/Ba04NSQAAAAABgAHAAgACQAACiA4I408BgAAC8COzYc8BgAADAEAbRk8XkwGAAcAAAANAADh9QUAAAAAATvX9QUAAAAAAkz+9QUAAAAAAwsABPj/BfYqAAAAAAAABgAHAAgACQAACovf9QUAAAAAC+ba9QUAAAAADAEAbRk8XkwGAA==";

    /// Updates of the default feed properties, in the `leEcdsa` format.
    const PYTH_LAZER_UPDATES: [&[u8]; 3] = [
        &[
            228, 189, 71, 77, 155, 77, 83, 126, 124, 37, 209, 194, 89, 114, 165, 72, 144, 13, 138,
            152, 104, 189, 145, 58, 21, 106, 249, 217, 179, 135, 250, 6, 55, 213, 135, 87, 62, 10,
            68, 204, 226, 52, 39, 178, 68, 42, 197, 79, 14, 105, 118, 26, 254, 95, 31, 109, 215,
            106, 195, 181, 56, 11, 165, 82, 30, 94, 37, 64, 0, 53, 0, 117, 211, 199, 147, 64, 195,
            236, 88, 4, 92, 6, 0, 3, 1, 1, 0, 0, 0, 5, 0, 128, 174, 218, 98, 232, 7, 0, 0, 4, 248,
            255, 5, 82, 20, 217, 58, 0, 0, 0, 0, 9, 0, 0, 12, 1, 64, 195, 236, 88, 4, 92, 6, 0,
        ],
        &[
            228, 189, 71, 77, 156, 67, 250, 57, 25, 123, 108, 85, 70, 172, 127, 78, 11, 106, 174,
            99, 192, 165, 5, 199, 201, 233, 239, 14, 70, 183, 125, 192, 30, 143, 172, 68, 35, 83,
            234, 247, 42, 117, 98, 219, 182, 174, 135, 223, 36, 221, 89, 178, 225, 91, 0, 70, 19,
            179, 86, 183, 65, 162, 207, 225, 131, 133, 173, 175, 0, 53, 0, 117, 211, 199, 147, 0,
            235, 245, 88, 4, 92, 6, 0, 3, 1, 2, 0, 0, 0, 5, 0, 49, 64, 201, 190, 64, 0, 0, 0, 4,
            248, 255, 5, 51, 209, 65, 1, 0, 0, 0, 0, 9, 0, 0, 12, 1, 0, 235, 245, 88, 4, 92, 6, 0,
        ],
        &[
            228, 189, 71, 77, 236, 8, 218, 33, 37, 205, 94, 170, 12, 46, 198, 24, 83, 197, 162, 46,
            111, 236, 41, 123, 6, 204, 168, 37, 138, 187, 248, 41, 164, 110, 240, 243, 107, 102,
            152, 246, 239, 139, 110, 98, 153, 129, 208, 228, 75, 224, 38, 27, 248, 96, 71, 242, 58,
            244, 36, 97, 169, 209, 169, 112, 149, 109, 240, 83, 0, 92, 0, 117, 211, 199, 147, 192,
            18, 255, 88, 4, 92, 6, 0, 3, 2, 1, 0, 0, 0, 5, 0, 248, 46, 41, 103, 232, 7, 0, 0, 4,
            248, 255, 5, 202, 148, 39, 63, 0, 0, 0, 0, 9, 0, 0, 12, 1, 192, 18, 255, 88, 4, 92, 6,
            0, 2, 0, 0, 0, 5, 0, 157, 195, 93, 191, 64, 0, 0, 0, 4, 248, 255, 5, 157, 218, 68, 1,
            0, 0, 0, 0, 9, 0, 0, 12, 1, 192, 18, 255, 88, 4, 92, 6, 0,
        ],
    ];

    const PYTH_LAZER_BINARY_ANSWER: &[u8] = &[
        115, 119, 136, 27, 0, 0, 0, 0, 0, 0, 0, 1, 0, 253, 228, 189, 71, 77, 89, 217, 69, 65, 163,
        37, 208, 162, 81, 207, 184, 53, 65, 210, 179, 94, 179, 237, 207, 222, 153, 60, 159, 208,
        159, 56, 208, 6, 142, 51, 206, 197, 34, 185, 85, 169, 127, 172, 153, 15, 25, 166, 242, 168,
        74, 241, 9, 209, 166, 104, 97, 96, 111, 206, 19, 50, 50, 38, 59, 157, 76, 49, 86, 146, 1,
        182, 0, 117, 211, 199, 147, 64, 101, 36, 153, 94, 76, 6, 0, 3, 2, 1, 0, 0, 0, 13, 0, 20,
        27, 209, 92, 42, 6, 0, 0, 1, 118, 101, 169, 92, 42, 6, 0, 0, 2, 162, 190, 23, 95, 42, 6, 0,
        0, 3, 18, 0, 4, 248, 255, 5, 37, 151, 250, 18, 0, 0, 0, 0, 6, 0, 7, 0, 8, 0, 9, 0, 0, 10,
        96, 225, 236, 86, 56, 6, 0, 0, 11, 192, 145, 134, 77, 56, 6, 0, 0, 12, 1, 64, 101, 36, 153,
        94, 76, 6, 0, 7, 0, 0, 0, 13, 0, 255, 224, 245, 5, 0, 0, 0, 0, 1, 247, 206, 245, 5, 0, 0,
        0, 0, 2, 171, 238, 245, 5, 0, 0, 0, 0, 3, 11, 0, 4, 248, 255, 5, 234, 44, 0, 0, 0, 0, 0, 0,
        6, 0, 7, 0, 8, 0, 9, 0, 0, 10, 111, 222, 245, 5, 0, 0, 0, 0, 11, 149, 217, 245, 5, 0, 0, 0,
        0, 12, 1, 64, 101, 36, 153, 94, 76, 6, 0,
    ];
    #[test]
    fn pyth_lazer_price_updates() {
        let updates = PriceUpdates::from_base64(PYTH_LAZER_PRICE_UPDATES_BASE64).unwrap();
        assert_eq!(updates.feed_count(), 2);
        assert!(updates.matches_set(&PriceFeedIdSet::from([USDC_USD, BTC_USD])));
        assert_eq!(updates.timestamp(), 1_772_817_479_200_000);
        assert_eq!(updates.channel(), Channel::FixedRate200ms);
        assert_eq!(
            updates.feeds().collect::<Vec<_>>(),
            [
                Feed {
                    feed_id: BTC_USD,
                    price: Some(6_842_727_179_617),
                    best_bid_price: Some(6_842_661_012_876),
                    best_ask_price: Some(6_842_846_025_594),
                    publisher_count: Some(18),
                    exponent: Some(-8),
                    confidence: Some(607_467_693),
                    // Requested but not reported.
                    funding_rate: None,
                    funding_timestamp: None,
                    funding_rate_interval: None,
                    market_session: Some(MarketSession::Regular),
                    ema_price: Some(6_857_135_700_000),
                    ema_confidence: Some(6_857_046_200_000),
                    feed_update_timestamp: Some(1_772_817_479_200_000),
                },
                Feed {
                    feed_id: USDC_USD,
                    price: Some(100_000_000),
                    best_bid_price: Some(99_997_499),
                    best_ask_price: Some(100_007_500),
                    publisher_count: Some(11),
                    exponent: Some(-8),
                    confidence: Some(10_998),
                    funding_rate: None,
                    funding_timestamp: None,
                    funding_rate_interval: None,
                    market_session: Some(MarketSession::Regular),
                    ema_price: Some(99_999_627),
                    ema_confidence: Some(99_998_438),
                    feed_update_timestamp: Some(1_772_817_479_200_000),
                },
            ],
        );
    }

    #[test]
    fn matches_set_rejects_repeated_ids() {
        let expected = PriceFeedIdSet::from([BTC_USD, USDC_USD]);

        let reversed = price_updates_with_feed_ids(&[USDC_USD, BTC_USD]);
        assert!(reversed.matches_set(&expected));

        let repeated = price_updates_with_feed_ids(&[BTC_USD, BTC_USD]);
        assert!(!repeated.matches_set(&expected));
    }

    #[test]
    fn parse_default_properties() {
        let expected = [
            (
                1_790_023_601_800_000,
                &[(BTC_USD, 8_694_672_305_792, 987_305_042)][..],
            ),
            (
                1_790_023_602_400_000,
                &[(ETH_USD, 278_078_767_153, 21_090_611)],
            ),
            (
                1_790_023_603_000_000,
                &[
                    (BTC_USD, 8_694_744_559_352, 1_059_558_602),
                    (ETH_USD, 278_088_500_125, 21_289_629),
                ],
            ),
        ];
        for (bytes, (timestamp, feeds)) in PYTH_LAZER_UPDATES.into_iter().zip(expected) {
            let updates = PriceUpdates::from_slice(bytes).unwrap();
            let feeds: Vec<_> = feeds
                .iter()
                .map(|&(feed_id, price, confidence)| Feed {
                    feed_id,
                    price: Some(price),
                    exponent: Some(-8),
                    confidence: Some(confidence),
                    market_session: Some(MarketSession::Regular),
                    feed_update_timestamp: Some(timestamp),
                    ..Default::default()
                })
                .collect();

            assert_eq!(updates.timestamp(), timestamp);
            assert_eq!(updates.channel(), Channel::FixedRate200ms);
            assert_eq!(updates.feeds().collect::<Vec<_>>(), feeds);
        }
    }

    #[test]
    fn zero_prices_are_unavailable() {
        // After the magic, signature, payload length, payload magic, timestamp, channel, feed
        // count, feed ID, property count and the price property ID.
        let price = 4 + 65 + 2 + 4 + 8 + 1 + 1 + 4 + 1 + 1;
        // After the price, the exponent and the confidence property ID.
        let confidence = price + 8 + (1 + 2) + 1;
        let mut bytes = PYTH_LAZER_UPDATES[0].to_vec();
        bytes[price..price + 8].fill(0);

        let feed = PriceUpdates::from_slice(&bytes)
            .unwrap()
            .feeds()
            .next()
            .unwrap();
        assert_eq!(feed.price, None);
        assert_eq!(feed.exponent, Some(-8));
        assert_eq!(feed.confidence, Some(987_305_042));

        bytes[confidence..confidence + 8].fill(0);
        let feed = PriceUpdates::from_slice(&bytes)
            .unwrap()
            .feeds()
            .next()
            .unwrap();
        assert_eq!(feed.confidence, None);
    }

    #[test]
    fn reject_invalid_updates() {
        let valid = PYTH_LAZER_UPDATES[0];
        let modified = |index: usize, byte: u8| {
            let mut bytes = valid.to_vec();
            bytes[index] = byte;
            bytes
        };
        let mut trailing_data = valid.to_vec();
        trailing_data.push(0);
        // After the magic, signature and payload length.
        let payload_start = 4 + 65 + 2;
        // After the payload magic, timestamp and channel.
        let feed_count = payload_start + 4 + 8 + 1;
        // After the feed count, feed ID and property count.
        let first_property = feed_count + 1 + 4 + 1;
        // After price, exponent and confidence.
        let market_session = first_property + (1 + 8) + (1 + 2) + (1 + 8);
        // After the market session.
        let feed_update_timestamp = market_session + 1 + 2;

        let cases = [
            ("invalid magic", modified(0, 0)),
            ("invalid payload magic", modified(payload_start, 0)),
            ("unknown channel", modified(feed_count - 1, 5)),
            ("feed count too large", modified(feed_count, 2)),
            ("invalid property", modified(first_property, 13)),
            (
                "invalid option flag",
                modified(feed_update_timestamp + 1, 2),
            ),
            ("trailing data", trailing_data),
            ("truncated", valid[..valid.len() - 1].to_vec()),
        ];
        for (name, bytes) in cases {
            assert!(PriceUpdates::from_slice(&bytes).is_err(), "{name}");
        }

        let unknown_session = PriceUpdates::from_slice(&modified(market_session + 1, 5)).unwrap();
        assert_eq!(
            unknown_session.feeds().next().unwrap().market_session,
            Some(MarketSession::Unknown(5)),
        );
    }

    #[test]
    fn market_session_round_trips_through_u16() {
        for value in 0..=6_u16 {
            assert_eq!(u16::from(MarketSession::from(value)), value);
        }
        assert_eq!(MarketSession::from(0), MarketSession::Regular);
        assert_eq!(MarketSession::from(4), MarketSession::Closed);
        assert_eq!(MarketSession::from(5), MarketSession::Unknown(5));
        assert_eq!(u16::from(MarketSession::Unknown(u16::MAX)), u16::MAX);
    }

    #[test]
    fn signature_payload_and_feed_by_id() {
        let bytes = Bytes::from_static(PYTH_LAZER_UPDATES[0]);
        let updates = PriceUpdates::from_bytes(bytes.clone()).unwrap();

        assert_eq!(updates.signature(), &bytes[4..4 + 65]);
        assert_eq!(updates.payload(), &bytes[4 + 65 + 2..]);
        assert_eq!(updates.payload()[..4], PAYLOAD_FORMAT_MAGIC.to_le_bytes());
        assert_eq!(updates.bytes().as_ptr(), bytes.as_ptr());

        assert_eq!(updates.feed(BTC_USD).unwrap().feed_id, BTC_USD);
        assert_eq!(updates.feed(USDC_USD), None);
        assert_eq!(updates.into_bytes(), bytes);
    }

    #[test]
    fn parse_binary_answer_with_subscription_id() {
        let (subscription_id, updates) =
            parse_binary_answer(Bytes::from_static(PYTH_LAZER_BINARY_ANSWER)).unwrap();
        assert_eq!(subscription_id, 1);
        assert_eq!(updates.feed_count(), 2);
        let feeds: Vec<_> = updates.feeds().collect();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[0].feed_id, BTC_USD);
        assert_eq!(feeds[0].price, Some(6_779_015_600_916));
        assert_eq!(feeds[1].feed_id, USDC_USD);
        assert_eq!(feeds[1].price, Some(99_999_999));
        assert_eq!(updates.timestamp(), 1_772_819_040_200_000);
    }

    #[test]
    fn reject_invalid_binary_answers() {
        let mut invalid_magic = PYTH_LAZER_BINARY_ANSWER.to_vec();
        invalid_magic[0] = 0;
        let mut trailing_data = PYTH_LAZER_BINARY_ANSWER.to_vec();
        trailing_data.push(0);
        let truncated = PYTH_LAZER_BINARY_ANSWER[..PYTH_LAZER_BINARY_ANSWER.len() - 1].to_vec();

        for (name, bytes) in [
            ("invalid magic", invalid_magic),
            ("trailing data", trailing_data),
            ("truncated", truncated),
        ] {
            assert!(parse_binary_answer(bytes.into()).is_err(), "{name}");
        }
    }

    #[test]
    fn feed_properties_display_parse_and_serialize_by_name() {
        for &property in ALL_FEED_PROPERTIES {
            let name = property.to_string();
            assert_eq!(name.parse(), Ok(property));
            assert_eq!(
                serde_json::to_string(&property).unwrap(),
                format!("\"{name}\"")
            );
        }
        assert_eq!(
            "bestBidPrice".parse::<FeedProperty>(),
            Ok(FeedProperty::BestBidPrice)
        );
        assert!("best_bid_price".parse::<FeedProperty>().is_err());
    }

    #[test]
    fn channels_display_parse_and_serialize_by_name() {
        for channel in [
            Channel::RealTime,
            Channel::FixedRate50ms,
            Channel::FixedRate200ms,
            Channel::FixedRate1000ms,
        ] {
            let name = channel.to_string();
            assert_eq!(name.parse(), Ok(channel));
            assert_eq!(
                serde_json::to_string(&channel).unwrap(),
                format!("\"{name}\"")
            );
        }
        assert_eq!(
            "fixed_rate@10ms".parse::<Channel>(),
            Err(ParseError::new("unknown channel: fixed_rate@10ms"))
        );
    }
}
