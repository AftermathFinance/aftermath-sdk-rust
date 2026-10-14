//! Aftermath signed price update format and feed identifiers.

pub use crate::aftermath_id_set::PriceFeedIdSet;
pub use crate::aftermath_writer::UnsignedPriceUpdates;
pub use crate::pyth_lazer::MarketSession;
use crate::{
    keccak::keccak256,
    reader::{Reader, decode_base64},
    sorted_set::SortedSetSlice,
};
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

/// Aftermath price feed ID.
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

/// Little-endian bytes spell "AFUP" (Aftermath feed update protocol).
pub(crate) const UPDATE_MAGIC: u32 = u32::from_le_bytes(*b"AFUP");

/// Length of the recoverable secp256k1 signature trailing an update: the 64-byte compact
/// signature `r ‖ s` followed by the recovery ID.
pub const SIGNATURE_LEN: usize = 65;

/// An Aftermath price update: a payload of feeds followed by its signature.
///
/// The last 65 bytes are the signature and everything before them is the payload, which are
/// passed separately to `af_verifier::verifier::verify_and_parse` on-chain.
///
/// Constructors check the layout, including that feed IDs are strictly increasing (sorted and
/// duplicate-free). They never verify the signature, which is left to
/// `af_verifier::verifier::verify_and_parse`.
///
/// To serialize and sign an update, see [`UnsignedPriceUpdates`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PriceUpdates(Bytes);

impl TryFrom<BytesMut> for PriceUpdates {
    type Error = Error;
    /// Copies the bytes, like [`PriceUpdates::from_slice`], so the update does not keep a
    /// larger buffer that `bytes` may share alive.
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
        Self::from_bytes(decode_base64(base64.into(), "Aftermath")?)
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
    /// The signed payload, without the trailing signature.
    pub fn payload(&self) -> &[u8] {
        self.split().0
    }
    /// The recoverable secp256k1 signature over [`Self::payload`].
    pub fn signature(&self) -> &[u8; SIGNATURE_LEN] {
        self.split().1
    }
    /// Keccak-256 of [`Self::payload`]: the prehash that the signer signed, and that
    /// `af_verifier` recovers the signer's public key from.
    pub fn digest(&self) -> [u8; 32] {
        keccak256(self.payload())
    }
    /// Timestamp of the update in microseconds.
    pub fn timestamp(&self) -> u64 {
        let mut reader = Reader(self.payload());
        // Skip magic.
        reader.skip(4).expect("checked on construction");
        reader.take_u64().expect("checked on construction")
    }
    /// Number of feeds in the update.
    pub fn feed_count(&self) -> usize {
        self.feeds().len()
    }
    /// Feeds of this update in strictly increasing feed ID order.
    ///
    /// Construction guarantees that the IDs are sorted and duplicate-free.
    pub fn feeds(&self) -> impl Clone + ExactSizeIterator<Item = Feed> {
        let mut reader = Reader(self.payload());
        // Skip magic, timestamp.
        reader.skip(4 + 8).expect("checked on construction");
        let feed_count = reader.take_vec_length().expect("checked on construction");
        (0..feed_count).map(move |_| take_feed(&mut reader).expect("checked on construction"))
    }
    /// The feed with `feed_id`, if any.
    ///
    /// Stops parsing at the first feed with a greater ID, since feed IDs are strictly increasing.
    pub fn feed(&self, feed_id: PriceFeedId) -> Option<Feed> {
        self.feeds()
            .find(|feed| feed.feed_id >= feed_id)
            .filter(|feed| feed.feed_id == feed_id)
    }
    /// Returns whether the feeds in this update exactly match `feed_ids`.
    ///
    /// Pass a reference to a [`PriceFeedIdSet`], [`SortedSet`](crate::sorted_set::SortedSet), or
    /// [`ArcSortedSet`](crate::sorted_set::ArcSortedSet); deref coercion borrows its
    /// [`SortedSetSlice`] view. Both the update and `feed_ids` are guaranteed to be sorted and
    /// duplicate-free, so this method compares them directly without allocating.
    pub fn matches_set(&self, feed_ids: &SortedSetSlice<PriceFeedId>) -> bool {
        self.feeds()
            .map(|feed| feed.feed_id)
            .eq(feed_ids.iter().copied())
    }
    fn split(&self) -> (&[u8], &[u8; SIGNATURE_LEN]) {
        self.0.split_last_chunk().expect("checked on construction")
    }
}

/// A parsed Aftermath feed.
///
/// Optional properties are `None` when they were **not reported** in this update, which is
/// semantically distinct from a value of zero.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Feed {
    /// ID of the feed.
    pub feed_id: PriceFeedId,
    /// Aggregate market price (mantissa). Use with exponent for actual price.
    pub price: i64,
    /// Decimal exponent: `actual_price = mantissa * 10^exponent`.
    pub exponent: i16,
    /// Timestamp when price was last generated for this feed (μs).
    pub feed_update_timestamp: u64,
    /// Number of contributing publishers.
    pub publisher_count: u16,
    /// Session: regular, preMarket, postMarket, overNight, closed.
    pub market_session: MarketSession,
    /// Price uncertainty; higher = more disagreement.
    pub confidence: Option<i64>,
    /// Highest bid across publishers.
    pub best_bid_price: Option<i64>,
    /// Lowest ask across publishers.
    pub best_ask_price: Option<i64>,
    /// Exponential moving average of the price.
    pub ema_price: Option<i64>,
    /// Exponential moving average of the confidence.
    pub ema_confidence: Option<u64>,
    /// Funding rate (perpetual futures).
    pub funding_rate: Option<i64>,
    /// When funding rate was last calculated (μs).
    pub funding_timestamp: Option<u64>,
    /// Interval between funding updates (μs).
    pub funding_rate_interval: Option<u64>,
}

impl Feed {
    /// Returns `true` when the market session is reported as [`MarketSession::Closed`].
    pub fn is_market_closed(&self) -> bool {
        self.market_session == MarketSession::Closed
    }
}

/// Mirrors `af_verifier::feed::parse_from_cursor`.
fn take_feed(reader: &mut Reader<'_>) -> Result<Feed, Error> {
    let mut feed = Feed {
        feed_id: PriceFeedId(reader.take_u32()?),
        price: reader.take_i64()?,
        exponent: reader.take_i16()?,
        feed_update_timestamp: reader.take_u64()?,
        publisher_count: reader.take_u16()?,
        market_session: MarketSession::from(reader.take_u16()?),
        ..Default::default()
    };
    let property_count = reader.take_u8()?;
    for _ in 0..property_count {
        let property_id = reader.take_u8()?;
        match property_id {
            0 => feed.confidence = Some(reader.take_i64()?),
            1 => feed.best_bid_price = Some(reader.take_i64()?),
            2 => feed.best_ask_price = Some(reader.take_i64()?),
            3 => feed.ema_price = Some(reader.take_i64()?),
            4 => feed.ema_confidence = Some(reader.take_u64()?),
            5 => feed.funding_rate = Some(reader.take_i64()?),
            6 => feed.funding_timestamp = Some(reader.take_u64()?),
            7 => feed.funding_rate_interval = Some(reader.take_u64()?),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Aftermath invalid property ID: {property_id}"),
                ));
            }
        }
    }
    Ok(feed)
}

/// Mirrors `af_verifier::update::parse`.
fn check_bytes(bytes: &[u8]) -> Result<(), Error> {
    let (update, _signature) = bytes.split_last_chunk::<SIGNATURE_LEN>().ok_or_else(|| {
        Error::new(
            ErrorKind::UnexpectedEof,
            "Aftermath signed price updates are shorter than a signature",
        )
    })?;
    let mut reader = Reader(update);
    if reader.take_u32()? != UPDATE_MAGIC {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Aftermath invalid update magic number",
        ));
    }
    let _timestamp = reader.take_u64()?;
    let feed_count = reader.take_vec_length()?;
    let mut prev_feed_id = None;
    for _ in 0..feed_count {
        let feed = take_feed(&mut reader)?;
        // Strict increase makes the feeds a sorted set and supports early-exit lookup by ID.
        if prev_feed_id.is_some_and(|prev_feed_id| feed.feed_id <= prev_feed_id) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Aftermath feed IDs are not strictly increasing",
            ));
        }
        prev_feed_id = Some(feed.feed_id);
    }
    reader.assert_eof()?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::sorted_set::{ArcSortedSet, SortedSet};
    use base64ct::{Base64, Encoding as _};

    pub(crate) const TIMESTAMP: u64 = 1_700_000_000_123_456;
    pub(crate) const SIGNATURE: [u8; SIGNATURE_LEN] = [0xab; SIGNATURE_LEN];

    /// Serializes a signed update, taking the ULEB128 feed count as raw bytes.
    pub(crate) fn signed_update(feed_count: &[u8], feeds: &[u8]) -> Vec<u8> {
        let mut bytes = b"AFUP".to_vec();
        bytes.extend_from_slice(&TIMESTAMP.to_le_bytes());
        bytes.extend_from_slice(feed_count);
        bytes.extend_from_slice(feeds);
        bytes.extend_from_slice(&SIGNATURE);
        bytes
    }

    /// Serializes a feed without optional properties.
    pub(crate) fn mandatory_feed(
        feed_id: u32,
        price: i64,
        exponent: i16,
        market_session: u16,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&feed_id.to_le_bytes());
        bytes.extend_from_slice(&price.to_le_bytes());
        bytes.extend_from_slice(&exponent.to_le_bytes());
        bytes.extend_from_slice(&TIMESTAMP.to_le_bytes());
        bytes.extend_from_slice(&42u16.to_le_bytes());
        bytes.extend_from_slice(&market_session.to_le_bytes());
        bytes.push(0);
        bytes
    }

    /// Serializes a feed with every optional property, valued `100 + property ID`.
    pub(crate) fn feed_with_all_properties() -> Vec<u8> {
        let mut bytes = mandatory_feed(1, 1, 0, 0);
        *bytes.last_mut().unwrap() = 8;
        for property_id in 0..8u8 {
            bytes.push(property_id);
            bytes.extend_from_slice(&(u64::from(property_id) + 100).to_le_bytes());
        }
        bytes
    }

    fn signed_update_with_feed_ids(feed_ids: &[u32]) -> Vec<u8> {
        let feeds: Vec<u8> = feed_ids
            .iter()
            .flat_map(|&feed_id| mandatory_feed(feed_id, 1, 0, 0))
            .collect();
        signed_update(&[feed_ids.len() as u8], &feeds)
    }

    #[test]
    fn parse_mandatory_fields() {
        let bytes = signed_update(&[1], &mandatory_feed(0x7856_3412, -10, -8, 4));
        let updates = PriceUpdates::from_slice(&bytes).unwrap();

        assert_eq!(updates.timestamp(), TIMESTAMP);
        assert_eq!(updates.feed_count(), 1);
        let feed = updates.feeds().next().unwrap();
        assert_eq!(
            feed,
            Feed {
                feed_id: PriceFeedId(0x7856_3412),
                price: -10,
                exponent: -8,
                feed_update_timestamp: TIMESTAMP,
                publisher_count: 42,
                market_session: MarketSession::Closed,
                ..Default::default()
            },
        );
        assert!(feed.is_market_closed());
    }

    #[test]
    fn parse_optional_properties() {
        let bytes = signed_update(&[1], &feed_with_all_properties());
        let feed = PriceUpdates::from_slice(&bytes)
            .unwrap()
            .feeds()
            .next()
            .unwrap();

        assert_eq!(feed.confidence, Some(100));
        assert_eq!(feed.best_bid_price, Some(101));
        assert_eq!(feed.best_ask_price, Some(102));
        assert_eq!(feed.ema_price, Some(103));
        assert_eq!(feed.ema_confidence, Some(104));
        assert_eq!(feed.funding_rate, Some(105));
        assert_eq!(feed.funding_timestamp, Some(106));
        assert_eq!(feed.funding_rate_interval, Some(107));
    }

    #[test]
    fn parse_uleb128_feed_count() {
        let feeds: Vec<u8> = (0..128)
            .flat_map(|feed_id| mandatory_feed(feed_id, 1, 0, 0))
            .collect();
        // 128 encoded as ULEB128, proving the count is not a single u8.
        let bytes = signed_update(&[0x80, 0x01], &feeds);
        let updates = PriceUpdates::from_slice(&bytes).unwrap();

        assert_eq!(updates.feed_count(), 128);
        assert!(updates.matches_set(&(0..128).map(PriceFeedId).collect::<SortedSet<_>>()));
        assert!(!updates.matches_set(&(0..127).map(PriceFeedId).collect::<SortedSet<_>>()));
        assert!(!updates.matches_set(&(0..129).map(PriceFeedId).collect::<SortedSet<_>>()));
    }

    #[test]
    fn update_bytes_and_signature() {
        let bytes = signed_update_with_feed_ids(&[1, 5]);
        let updates = PriceUpdates::from_bytes(Bytes::from(bytes.clone())).unwrap();

        assert_eq!(updates.as_bytes(), bytes);
        assert_eq!(updates.payload(), &bytes[..bytes.len() - SIGNATURE_LEN]);
        assert_eq!(updates.signature(), &SIGNATURE);
        assert!(updates.matches_set(&PriceFeedIdSet::from([PriceFeedId(5), PriceFeedId(1)])));
    }

    #[test]
    fn bytes_are_shared() {
        let bytes = Bytes::from(signed_update_with_feed_ids(&[1, 5]));
        let updates = PriceUpdates::from_bytes(bytes.clone()).unwrap();

        assert_eq!(updates.bytes().as_ptr(), bytes.as_ptr());
        assert_eq!(updates.into_bytes(), bytes);
    }

    #[test]
    fn feed_by_id() {
        let updates = PriceUpdates::from_slice(&signed_update_with_feed_ids(&[1, 5, 9])).unwrap();

        assert_eq!(
            updates.feed(PriceFeedId(5)).unwrap().feed_id,
            PriceFeedId(5)
        );
        assert_eq!(
            updates.feed(PriceFeedId(9)).unwrap().feed_id,
            PriceFeedId(9)
        );
        for missing in [0, 4, 6, 10] {
            assert_eq!(updates.feed(PriceFeedId(missing)), None, "{missing}");
        }
    }

    #[test]
    fn parse_unknown_market_session() {
        let bytes = signed_update(&[1], &mandatory_feed(1, 1, 0, 5));
        let feed = PriceUpdates::from_slice(&bytes)
            .unwrap()
            .feeds()
            .next()
            .unwrap();

        assert_eq!(feed.market_session, MarketSession::Unknown(5));
        assert!(!feed.is_market_closed());
    }

    #[test]
    fn from_base64() {
        let bytes = signed_update_with_feed_ids(&[1, 5]);
        let base64 = Base64::encode_string(&bytes);
        let updates = PriceUpdates::from_base64(base64).unwrap();

        assert_eq!(updates, PriceUpdates::from_slice(&bytes).unwrap());
    }

    #[test]
    fn reject_duplicate_feed_ids() {
        let bytes = signed_update_with_feed_ids(&[1, 5, 5]);
        let err = PriceUpdates::from_slice(&bytes).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert_eq!(
            err.to_string(),
            "Aftermath feed IDs are not strictly increasing"
        );
    }

    #[test]
    fn matches_set_with_sorted_set_and_checked_slice() {
        let updates = PriceUpdates::from_slice(&signed_update_with_feed_ids(&[1, 5])).unwrap();
        let requested = SortedSet::from([PriceFeedId(5), PriceFeedId(1)]);

        assert!(updates.matches_set(&requested));
        assert!(updates.matches_set(&ArcSortedSet::from([PriceFeedId(5), PriceFeedId(1)])));
        let ids = [PriceFeedId(1), PriceFeedId(5)];
        assert!(updates.matches_set(SortedSetSlice::try_from_slice(&ids).unwrap()));
    }

    #[test]
    fn reject_invalid_updates() {
        let valid = signed_update_with_feed_ids(&[1, 5]);
        let update_len = valid.len() - SIGNATURE_LEN;

        let mut invalid_magic = valid.clone();
        invalid_magic[0] = b'X';

        let mut trailing_data = valid[..update_len].to_vec();
        trailing_data.push(0);
        trailing_data.extend_from_slice(&SIGNATURE);

        let mut feed_count_too_large = valid.clone();
        feed_count_too_large[12] = 3;

        let mut invalid_property = mandatory_feed(1, 1, 0, 0);
        *invalid_property.last_mut().unwrap() = 1;
        invalid_property.push(8);
        invalid_property.extend_from_slice(&0u64.to_le_bytes());

        let cases = [
            ("unsorted", signed_update_with_feed_ids(&[1, 5, 4])),
            ("invalid magic", invalid_magic),
            ("trailing data", trailing_data),
            ("truncated", valid[..valid.len() - 1].to_vec()),
            ("no signature", valid[..SIGNATURE_LEN - 1].to_vec()),
            ("feed count too large", feed_count_too_large),
            ("invalid property", signed_update(&[1], &invalid_property)),
        ];
        for (name, bytes) in cases {
            assert!(PriceUpdates::from_slice(&bytes).is_err(), "{name}");
        }
    }
}
