//! Serialization of Aftermath price updates, mirroring the parser in [`crate::aftermath`].
//!
//! The caller owns the buffer and the signing key: this module writes the payload and appends a
//! signature that the caller made over [`UnsignedPriceUpdates::digest`].

use crate::aftermath::{Feed, SIGNATURE_LEN, UPDATE_MAGIC};
use crate::keccak::keccak256;
use bytes::{BufMut, BytesMut};
use std::io::{Error, ErrorKind};
use std::ops::Range;

/// Longest feed count encoding accepted by `Reader::take_vec_length`.
const MAX_VEC_LENGTH_SIZE: usize = 5;

impl Feed {
    /// Length of the feed in a [`PriceUpdates`](crate::aftermath::PriceUpdates).
    pub const fn serialized_size(&self) -> usize {
        let property_count = self.confidence.is_some() as usize
            + self.best_bid_price.is_some() as usize
            + self.best_ask_price.is_some() as usize
            + self.ema_price.is_some() as usize
            + self.ema_confidence.is_some() as usize
            + self.funding_rate.is_some() as usize
            + self.funding_timestamp.is_some() as usize
            + self.funding_rate_interval.is_some() as usize;
        4 + 8 + 2 + 8 + 2 + 2 + 1 + property_count * (1 + 8)
    }
}

/// An unsigned Aftermath update at the end of a caller's buffer, waiting for its signature.
///
/// [`Self::new`] writes the payload. Sign [`Self::digest`] with any secp256k1 implementation, then
/// pass the signature to [`Self::append_signature`]. Dropping the value without appending a
/// signature truncates the buffer to its length before the payload, so an unsigned payload is never
/// left behind, e.g. when signing fails.
#[derive(Debug)]
#[must_use = "dropping an unsigned update removes its payload from the buffer"]
pub struct UnsignedPriceUpdates<'b> {
    buf: &'b mut BytesMut,
    /// Length of `buf` before the payload; the buffer is truncated to it on drop.
    payload_start: usize,
}

impl<'b> UnsignedPriceUpdates<'b> {
    /// Length of a signed update of `feeds`: its payload plus the signature.
    ///
    /// Allocate this much spare capacity for [`Self::new`], plus any framing written before the
    /// update, to serialize and sign it without reallocating.
    pub fn signed_size(feeds: &[Feed]) -> usize {
        let feeds_size: usize = feeds.iter().map(Feed::serialized_size).sum();
        4 + 8 + vec_length_size(feeds.len()) + feeds_size + SIGNATURE_LEN
    }
    /// Validates `feeds` and appends the unsigned payload of an update to `buf`.
    ///
    /// Sign the returned [`Self::digest`] and append the signature with [`Self::append_signature`].
    /// This crate never signs: the caller keeps the key and may use any secp256k1 implementation.
    ///
    /// First reserves [`Self::signed_size`] bytes, which is a no-op when `buf` already has that
    /// much spare capacity; otherwise `buf` grows once. On error nothing is written.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::InvalidInput`] if feed IDs are not strictly increasing.
    ///
    /// # Example
    ///
    /// ```
    /// use af_price_updates::aftermath::{Feed, PriceFeedId, PriceUpdates, UnsignedPriceUpdates};
    /// use bytes::{BufMut, BytesMut};
    ///
    /// let feeds = [Feed { feed_id: PriceFeedId(1), price: 42, ..Default::default() }];
    /// let size = UnsignedPriceUpdates::signed_size(&feeds);
    /// // A length prefix followed by the signed update, in a single allocation.
    /// let mut buf = BytesMut::with_capacity(8 + size);
    /// buf.put_u64(size as u64);
    /// let unsigned = UnsignedPriceUpdates::new(&mut buf, 1_700_000_000_000_000, &feeds)?;
    /// # let sign = |_: &[u8; 32]| ([0; 64], 0);
    /// let (compact_signature, recovery_id) = sign(&unsigned.digest());
    /// let update = unsigned.append_signature(&compact_signature, recovery_id);
    ///
    /// let updates = PriceUpdates::from_bytes(buf.freeze().slice(update))?;
    /// assert!(updates.feeds().eq(feeds));
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn new(buf: &'b mut BytesMut, timestamp: u64, feeds: &[Feed]) -> Result<Self, Error> {
        if vec_length_size(feeds.len()) > MAX_VEC_LENGTH_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Aftermath feed count is longer than five ULEB128 bytes",
            ));
        }
        if !feeds.is_sorted_by(|a, b| a.feed_id < b.feed_id) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Aftermath feed IDs are not strictly increasing",
            ));
        }
        let size = Self::signed_size(feeds);
        // A no-op for a buffer sized by the caller, and otherwise the only growth: the payload and
        // the signature both fit afterwards.
        buf.reserve(size);
        let payload_start = buf.len();
        buf.put_u32_le(UPDATE_MAGIC);
        buf.put_u64_le(timestamp);
        put_vec_length(buf, feeds.len());
        for feed in feeds {
            put_feed(buf, feed);
        }
        debug_assert_eq!(buf.len() - payload_start + SIGNATURE_LEN, size);
        Ok(Self { buf, payload_start })
    }
    /// The payload that the signature must cover.
    pub fn payload(&self) -> &[u8] {
        &self.buf[self.payload_start..]
    }
    /// Keccak-256 of [`Self::payload`], the prehash to sign.
    ///
    /// Equal to [`PriceUpdates::digest`](crate::aftermath::PriceUpdates::digest) of the signed
    /// update.
    pub fn digest(&self) -> [u8; 32] {
        keccak256(self.payload())
    }
    /// Appends a recoverable secp256k1 signature over [`Self::digest`], completing the update.
    ///
    /// `compact_signature` is `r ‖ s` as two 32-byte big-endian scalars, and `recovery_id` the
    /// recovery ID byte. For example, with `k256` pass `&signature.to_bytes().into()` and
    /// `recovery_id.to_byte()`; with `secp256k1`, `RecoverableSignature::serialize_compact`
    /// returns both.
    ///
    /// Returns the range of the complete signed update within the buffer.
    pub fn append_signature(
        mut self,
        compact_signature: &[u8; 64],
        recovery_id: u8,
    ) -> Range<usize> {
        self.buf.put_slice(compact_signature);
        self.buf.put_u8(recovery_id);
        let update = self.payload_start..self.buf.len();
        // The update is complete, so dropping `self` keeps it.
        self.payload_start = self.buf.len();
        update
    }
}

impl Drop for UnsignedPriceUpdates<'_> {
    fn drop(&mut self) {
        self.buf.truncate(self.payload_start);
    }
}

/// Mirrors `take_feed` in [`crate::aftermath`].
fn put_feed(buf: &mut BytesMut, feed: &Feed) {
    buf.put_u32_le(feed.feed_id.0);
    buf.put_i64_le(feed.price);
    buf.put_i16_le(feed.exponent);
    buf.put_u64_le(feed.feed_update_timestamp);
    buf.put_u16_le(feed.publisher_count);
    buf.put_u16_le(feed.market_session.into());
    // Indexed by property ID.
    let properties: [Option<u64>; _] = [
        feed.confidence.map(i64::cast_unsigned),
        feed.best_bid_price.map(i64::cast_unsigned),
        feed.best_ask_price.map(i64::cast_unsigned),
        feed.ema_price.map(i64::cast_unsigned),
        feed.ema_confidence,
        feed.funding_rate.map(i64::cast_unsigned),
        feed.funding_timestamp,
        feed.funding_rate_interval,
    ];
    buf.put_u8(properties.iter().flatten().count() as u8);
    for (property_id, value) in properties.into_iter().enumerate() {
        if let Some(value) = value {
            buf.put_u8(property_id as u8);
            buf.put_u64_le(value);
        }
    }
}

/// Length of `length` in ULEB128.
fn vec_length_size(length: usize) -> usize {
    let bit_width = (usize::BITS - length.leading_zeros()) as usize;
    bit_width.div_ceil(7).max(1)
}

/// Writes `length` in ULEB128, mirroring `Reader::take_vec_length`.
fn put_vec_length(buf: &mut BytesMut, mut length: usize) {
    while length >= 0x80 {
        buf.put_u8((length & 0x7f) as u8 | 0x80);
        length >>= 7;
    }
    buf.put_u8(length as u8);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aftermath::{
        MarketSession, PriceFeedId, PriceUpdates,
        tests::{SIGNATURE, TIMESTAMP, feed_with_all_properties, mandatory_feed, signed_update},
    };
    use bytes::Bytes;

    /// Appends the placeholder [`SIGNATURE`] of the parser tests.
    fn append_placeholder_signature(unsigned: UnsignedPriceUpdates<'_>) -> Range<usize> {
        let (compact_signature, recovery_id) = SIGNATURE.split_first_chunk().unwrap();
        unsigned.append_signature(compact_signature, recovery_id[0])
    }

    /// Writes a signed update of `feeds` into an exactly sized buffer.
    fn write_update(feeds: &[Feed]) -> BytesMut {
        let size = UnsignedPriceUpdates::signed_size(feeds);
        let mut buf = BytesMut::with_capacity(size);
        let unsigned = UnsignedPriceUpdates::new(&mut buf, TIMESTAMP, feeds).unwrap();

        assert_eq!(append_placeholder_signature(unsigned), 0..size);
        buf
    }

    /// Mirrors `mandatory_feed(feed_id, 1, 0, 0)`.
    fn feed(feed_id: u32) -> Feed {
        Feed {
            feed_id: PriceFeedId(feed_id),
            price: 1,
            feed_update_timestamp: TIMESTAMP,
            publisher_count: 42,
            ..Default::default()
        }
    }

    /// A buffer holding `framing`, with exactly enough spare capacity for a signed update of
    /// `feeds`.
    fn framed_buffer(framing: &[u8], feeds: &[Feed]) -> BytesMut {
        let mut buf =
            BytesMut::with_capacity(framing.len() + UnsignedPriceUpdates::signed_size(feeds));
        buf.put_slice(framing);
        buf
    }

    #[test]
    fn write_mandatory_fields() {
        let feed = Feed {
            feed_id: PriceFeedId(0x7856_3412),
            price: -10,
            exponent: -8,
            market_session: MarketSession::Closed,
            ..feed(0)
        };

        assert_eq!(
            write_update(&[feed]),
            signed_update(&[1], &mandatory_feed(0x7856_3412, -10, -8, 4))
        );
    }

    #[test]
    fn write_all_properties() {
        let feed = Feed {
            confidence: Some(100),
            best_bid_price: Some(101),
            best_ask_price: Some(102),
            ema_price: Some(103),
            ema_confidence: Some(104),
            funding_rate: Some(105),
            funding_timestamp: Some(106),
            funding_rate_interval: Some(107),
            ..feed(1)
        };
        let expected_feed = feed_with_all_properties();

        assert_eq!(feed.serialized_size(), expected_feed.len());
        assert_eq!(write_update(&[feed]), signed_update(&[1], &expected_feed));
    }

    #[test]
    fn write_uleb128_feed_count() {
        let feeds: Vec<Feed> = (0..128).map(feed).collect();
        let expected_feeds: Vec<u8> = (0..128)
            .flat_map(|feed_id| mandatory_feed(feed_id, 1, 0, 0))
            .collect();

        assert_eq!(
            write_update(&feeds),
            signed_update(&[0x80, 0x01], &expected_feeds)
        );
    }

    #[test]
    fn write_no_feeds() {
        assert_eq!(write_update(&[]), signed_update(&[0], &[]));
    }

    #[test]
    fn write_round_trips() {
        let feeds = [
            Feed {
                best_ask_price: Some(i64::MAX),
                funding_rate: Some(i64::MIN),
                ..Feed::default()
            },
            Feed {
                feed_id: PriceFeedId(7),
                ..Default::default()
            },
            Feed {
                feed_id: PriceFeedId(u32::MAX),
                price: i64::MIN,
                exponent: i16::MIN,
                feed_update_timestamp: u64::MAX,
                publisher_count: u16::MAX,
                market_session: MarketSession::OverNight,
                confidence: Some(-1),
                best_bid_price: Some(-2),
                best_ask_price: Some(-3),
                ema_price: Some(-4),
                ema_confidence: Some(u64::MAX),
                funding_rate: Some(-5),
                funding_timestamp: Some(6),
                funding_rate_interval: Some(7),
            },
        ];
        let updates = PriceUpdates::from_bytes(write_update(&feeds).freeze()).unwrap();

        assert_eq!(updates.timestamp(), TIMESTAMP);
        assert!(updates.feeds().eq(feeds));
        assert_eq!(updates.signature(), &SIGNATURE);
    }

    #[test]
    fn write_rejects_unsorted_feed_ids_without_writing() {
        for feed_ids in [[1, 1], [5, 1]] {
            let feeds = feed_ids.map(feed);
            let mut buf = framed_buffer(b"framing", &feeds);
            let err = UnsignedPriceUpdates::new(&mut buf, TIMESTAMP, &feeds).unwrap_err();

            assert_eq!(err.kind(), ErrorKind::InvalidInput, "{feed_ids:?}");
            assert_eq!(buf, &b"framing"[..], "{feed_ids:?}");
        }
    }

    #[test]
    fn write_after_framing_without_reallocating() {
        let feeds = [feed(1), feed(5)];
        let mut buf = framed_buffer(&[0; 8], &feeds);
        let (capacity, ptr) = (buf.capacity(), buf.as_ptr());
        let unsigned = UnsignedPriceUpdates::new(&mut buf, TIMESTAMP, &feeds).unwrap();
        let update = append_placeholder_signature(unsigned);

        assert_eq!(update, 8..buf.len());
        assert_eq!(buf.len(), capacity);
        assert_eq!(buf.capacity(), capacity);
        assert_eq!(buf.as_ptr(), ptr);
        assert_eq!(buf[..8], [0u8; 8]);
        let updates = PriceUpdates::from_bytes(buf.freeze().slice(update)).unwrap();
        assert!(updates.feeds().eq(feeds));
    }

    #[test]
    fn write_grows_undersized_buffer() {
        let feeds = [feed(1), feed(5)];
        let mut buf = BytesMut::from(&b"framing"[..]);
        let unsigned = UnsignedPriceUpdates::new(&mut buf, TIMESTAMP, &feeds).unwrap();
        let update = append_placeholder_signature(unsigned);

        assert_eq!(update.len(), UnsignedPriceUpdates::signed_size(&feeds));
        assert_eq!(buf[..7], *b"framing");
        assert_eq!(buf[update], write_update(&feeds));
    }

    #[test]
    fn dropping_unsigned_update_removes_payload() {
        let feeds = [feed(1)];
        let mut buf = framed_buffer(b"framing", &feeds);
        let unsigned = UnsignedPriceUpdates::new(&mut buf, TIMESTAMP, &feeds).unwrap();
        assert!(!unsigned.payload().is_empty());
        drop(unsigned);

        assert_eq!(buf, &b"framing"[..]);
    }

    #[test]
    fn digest_covers_payload() {
        let feeds = [feed(1), feed(5)];
        let mut buf = framed_buffer(b"framing", &feeds);
        let unsigned = UnsignedPriceUpdates::new(&mut buf, TIMESTAMP, &feeds).unwrap();
        let digest = unsigned.digest();

        assert_eq!(digest, keccak256(unsigned.payload()));
        let update = append_placeholder_signature(unsigned);
        let updates = PriceUpdates::from_bytes(Bytes::copy_from_slice(&buf[update])).unwrap();
        assert_eq!(updates.digest(), digest);
        assert_eq!(updates.payload(), &buf[7..buf.len() - SIGNATURE_LEN]);
    }

    #[test]
    fn vec_length_size_matches_written_length() {
        for length in [0, 1, 0x7f, 0x80, 0x3fff, 0x4000, u32::MAX as usize] {
            let mut buf = BytesMut::new();
            put_vec_length(&mut buf, length);

            assert_eq!(buf.len(), vec_length_size(length), "{length}");
        }
    }
}
