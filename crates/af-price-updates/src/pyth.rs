//! Pyth accumulator (`PNAU`) price update format and feed identifiers.

pub use crate::pyth_id_set::PriceFeedIdSet;
use crate::reader::{Reader, decode_base64};
use crate::{ParseError, U256, sorted_set::SortedSetSlice};
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

/// Pyth price feed ID, displayed and parsed as `0x`-prefixed hex.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[repr(transparent)]
pub struct PriceFeedId(pub U256);

/// A parsed Pyth feed.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Feed {
    /// ID of the feed.
    pub feed_id: PriceFeedId,
    /// The price.
    pub price: Price,
    /// Exponential moving average of the price.
    pub ema_price: Price,
}

/// A Pyth price with its confidence interval.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Price {
    /// Price mantissa; the price is `price * 10^exponent`.
    pub price: i64,
    /// Confidence interval around the price, with the same exponent.
    pub confidence: u64,
    /// Decimal exponent of `price` and `confidence`.
    pub exponent: i32,
    /// Publish time in seconds since the Unix epoch.
    pub timestamp: u64,
}

impl std::fmt::Display for PriceFeedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        <U256 as std::fmt::Display>::fmt(&self.0, f)
    }
}

impl std::str::FromStr for PriceFeedId {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<U256>().map(PriceFeedId)
    }
}

impl From<[u8; 32]> for PriceFeedId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(U256::from(bytes))
    }
}

impl AsRef<[u8]> for PriceFeedId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

const MAGIC: u32 = u32::from_be_bytes(*b"PNAU");
const MAJOR_VERSION: u8 = 1;
const PROOF_TYPE: u8 = 0;
const MESSAGE_LEN: u16 = 85;
const MESSAGE_TYPE: u8 = 0;

/// A Pyth accumulator (`PNAU`) update with exactly one feed.
///
/// Constructors check the layout only and never verify the VAA or Merkle proofs, which is left to
/// the on-chain Pyth contract.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PriceUpdate(Bytes);

impl TryFrom<BytesMut> for PriceUpdate {
    type Error = Error;
    /// Copies the bytes, like [`PriceUpdate::from_slice`], so the update does not keep a larger
    /// buffer that `bytes` may share alive.
    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        Self::from_slice(&bytes)
    }
}

impl PriceUpdate {
    /// Checks and copies an update with exactly one feed.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        check_bytes_with_one_feed(bytes)?;
        Ok(Self(Bytes::copy_from_slice(bytes)))
    }
    /// Checks an update with exactly one feed and keeps `bytes` without copying.
    ///
    /// Use [`Self::from_slice`] instead to copy the update out of a larger buffer.
    pub fn from_bytes(bytes: Bytes) -> Result<Self, Error> {
        check_bytes_with_one_feed(&bytes)?;
        Ok(Self(bytes))
    }
    /// Decodes a standard, padded base64 update and checks it has exactly one feed.
    ///
    /// Decodes in place when `base64` is the only owner of its buffer, such as a `String`, a
    /// `Vec<u8>` or the bytes of an unshared `bytestring::ByteString`, and copies it otherwise.
    pub fn from_base64(base64: impl Into<Bytes>) -> Result<Self, Error> {
        Self::from_bytes(decode_base64(base64.into(), "Pyth")?)
    }
    /// The whole update.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    /// The whole update, to share without copying.
    pub const fn bytes(&self) -> &Bytes {
        &self.0
    }
    /// The whole update, without copying.
    pub fn into_bytes(self) -> Bytes {
        self.0
    }
    /// The Wormhole VAA of the update.
    pub fn vaa(&self) -> &[u8] {
        vaa(&self.0)
    }
    /// The feed of the update.
    pub fn feed(&self) -> Feed {
        feeds(&self.0).next().expect("checked on construction")
    }
}

/// A Pyth accumulator (`PNAU`) update with any number of feeds.
///
/// Constructors check the layout only and never verify the VAA or Merkle proofs, which is left to
/// the on-chain Pyth contract.
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
        Self::from_bytes(decode_base64(base64.into(), "Pyth")?)
    }
    /// The whole update.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    /// The whole update, to share without copying.
    pub const fn bytes(&self) -> &Bytes {
        &self.0
    }
    /// The whole update, without copying.
    pub fn into_bytes(self) -> Bytes {
        self.0
    }
    /// The Wormhole VAA of the update.
    pub fn vaa(&self) -> &[u8] {
        vaa(&self.0)
    }
    /// Number of feeds in the update.
    pub fn feed_count(&self) -> usize {
        self.feeds().len()
    }
    /// Feeds of the update, in the order sent.
    pub fn feeds(&self) -> impl Clone + ExactSizeIterator<Item = Feed> {
        feeds(&self.0)
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

fn vaa(bytes: &[u8]) -> &[u8] {
    let mut reader = Reader(bytes);
    let header = take_header(&mut reader).expect("checked on construction");
    reader
        .take(header.vaa_len as usize)
        .expect("checked on construction")
}

fn feeds(bytes: &[u8]) -> impl Clone + ExactSizeIterator<Item = Feed> {
    let mut reader = Reader(bytes);
    let header = take_header(&mut reader).expect("checked on construction");
    reader
        .skip(header.vaa_len as usize)
        .expect("checked on construction");
    let feed_count = take_feed_count(&mut reader).expect("checked on construction");
    (0..feed_count)
        .map(move |_| take_feed_with_proof(&mut reader).expect("checked on construction"))
}

/// Checks the structure of an update, not its VAA or Merkle proofs, and returns its feed count.
fn check_bytes(bytes: &[u8]) -> Result<u8, Error> {
    let mut reader = Reader(bytes);
    let header = take_header(&mut reader)?;
    header.check()?;
    reader.skip(header.vaa_len as usize)?;
    let feed_count = take_feed_count(&mut reader)?;
    for _ in 0..feed_count {
        let _feed = take_feed_with_proof(&mut reader)?;
    }
    reader.assert_eof()?;
    Ok(feed_count)
}

fn check_bytes_with_one_feed(bytes: &[u8]) -> Result<(), Error> {
    if check_bytes(bytes)? != 1 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Pyth update does not have exactly one feed",
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct Header {
    magic: u32,
    major_version: u8,
    _minor_version: u8,
    _trailing_payload_len: u8,
    proof_type: u8,
    vaa_len: u16,
}

impl Header {
    fn check(&self) -> Result<(), Error> {
        if self.magic != MAGIC {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Pyth invalid magic number",
            ));
        }
        if self.major_version != MAJOR_VERSION {
            return Err(Error::new(ErrorKind::InvalidData, "Pyth invalid version"));
        }
        if self.proof_type != PROOF_TYPE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Pyth invalid proof type",
            ));
        }
        Ok(())
    }
}

fn take_header(reader: &mut Reader<'_>) -> Result<Header, Error> {
    let magic = u32::from_be_bytes(reader.take_chunk()?);
    let major_version = u8::from_be_bytes(reader.take_chunk()?);
    let minor_version = u8::from_be_bytes(reader.take_chunk()?);
    let trailing_payload_len = u8::from_be_bytes(reader.take_chunk()?);
    reader.skip(trailing_payload_len as usize)?;
    let proof_type = u8::from_be_bytes(reader.take_chunk()?);
    let vaa_length = u16::from_be_bytes(reader.take_chunk()?);
    Ok(Header {
        magic,
        major_version,
        _minor_version: minor_version,
        _trailing_payload_len: trailing_payload_len,
        proof_type,
        vaa_len: vaa_length,
    })
}

fn take_feed_count(reader: &mut Reader<'_>) -> Result<u8, Error> {
    Ok(u8::from_be_bytes(reader.take_chunk()?))
}

fn take_feed_with_proof(reader: &mut Reader<'_>) -> Result<Feed, Error> {
    let feed = take_feed(reader)?;
    let proof_len = 20usize * take_feed_count(reader)? as usize;
    reader.skip(proof_len)?;
    Ok(feed)
}

fn take_feed(reader: &mut Reader<'_>) -> Result<Feed, Error> {
    let message_len = u16::from_be_bytes(reader.take_chunk()?);
    if message_len != MESSAGE_LEN {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Pyth invalid message length",
        ));
    }
    let message_type = u8::from_be_bytes(reader.take_chunk()?);
    if message_type != MESSAGE_TYPE {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Pyth invalid message type",
        ));
    }
    let feed_id = PriceFeedId(U256::from(reader.take_chunk()?));
    let price = i64::from_be_bytes(reader.take_chunk()?);
    let confidence = u64::from_be_bytes(reader.take_chunk()?);
    let exponent = i32::from_be_bytes(reader.take_chunk()?);
    let timestamp = u64::from_be_bytes(reader.take_chunk()?);
    let _prev_timestamp = i64::from_be_bytes(reader.take_chunk()?);
    let ema_price = i64::from_be_bytes(reader.take_chunk()?);
    let ema_confidence = u64::from_be_bytes(reader.take_chunk()?);
    Ok(Feed {
        feed_id,
        price: Price {
            price,
            confidence,
            exponent,
            timestamp,
        },
        ema_price: Price {
            price: ema_price,
            confidence: ema_confidence,
            exponent,
            timestamp,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64ct::{Base64, Encoding as _};

    fn test_feed(id: u8, price: i64) -> Feed {
        Feed {
            feed_id: PriceFeedId(U256::from(u64::from(id))),
            price: Price {
                price,
                confidence: 100,
                exponent: -8,
                timestamp: 1_700_000_000,
            },
            ema_price: Price {
                price,
                confidence: 200,
                exponent: -8,
                timestamp: 1_700_000_000,
            },
        }
    }

    fn push_feed(bytes: &mut Vec<u8>, feed: Feed) {
        bytes.extend_from_slice(&MESSAGE_LEN.to_be_bytes());
        bytes.push(MESSAGE_TYPE);
        bytes.extend_from_slice(feed.feed_id.0.as_slice());
        bytes.extend_from_slice(&feed.price.price.to_be_bytes());
        bytes.extend_from_slice(&feed.price.confidence.to_be_bytes());
        bytes.extend_from_slice(&feed.price.exponent.to_be_bytes());
        bytes.extend_from_slice(&feed.price.timestamp.to_be_bytes());
        bytes.extend_from_slice(&0i64.to_be_bytes());
        bytes.extend_from_slice(&feed.ema_price.price.to_be_bytes());
        bytes.extend_from_slice(&feed.ema_price.confidence.to_be_bytes());
        bytes.push(0);
    }

    fn price_updates_bytes(feeds: &[Feed]) -> Bytes {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC.to_be_bytes());
        bytes.push(MAJOR_VERSION);
        bytes.push(0);
        bytes.push(0);
        bytes.push(PROOF_TYPE);
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.push(feeds.len().try_into().unwrap());
        for feed in feeds {
            push_feed(&mut bytes, *feed);
        }
        bytes.into()
    }

    #[test]
    fn feed_serde_uses_public_field_names() {
        let feed = test_feed(1, 42);
        let json = serde_json::to_value(feed).unwrap();

        assert!(json.get("feed_id").is_some());
        assert_eq!(json["price"]["confidence"], 100);
        assert_eq!(json["price"]["exponent"], -8);
        assert_eq!(serde_json::from_value::<Feed>(json).unwrap(), feed);
    }

    #[test]
    fn price_update_accepts_single_feed_payload() {
        let feed = test_feed(1, 42);
        let update = PriceUpdate::from_bytes(price_updates_bytes(&[feed])).unwrap();

        assert_eq!(update.feed(), feed);
    }

    #[test]
    fn price_update_rejects_multi_feed_payload() {
        let bytes = price_updates_bytes(&[test_feed(1, 42), test_feed(2, 43)]);

        assert!(PriceUpdate::from_bytes(bytes).is_err());
    }

    #[test]
    fn price_updates_iterates_multi_feed_payload() {
        let feeds = [test_feed(1, 42), test_feed(2, 43)];
        let updates = PriceUpdates::from_bytes(price_updates_bytes(&feeds)).unwrap();

        assert_eq!(updates.feed_count(), 2);
        assert_eq!(updates.feeds().collect::<Vec<_>>(), feeds);
        assert_eq!(updates.feed(feeds[1].feed_id), Some(feeds[1]));
        assert_eq!(updates.feed(test_feed(3, 44).feed_id), None);
    }

    #[test]
    fn price_feed_id_converts_from_and_to_bytes() {
        let mut bytes = [0; 32];
        bytes[31] = 7;
        let id = PriceFeedId::from(bytes);

        assert_eq!(id, PriceFeedId(U256::from(7)));
        assert_eq!(id.as_ref(), bytes.as_slice());
    }

    #[test]
    fn price_updates_matches_set_rejects_repeated_ids() {
        let first = test_feed(1, 42);
        let second = test_feed(2, 43);
        let expected = PriceFeedIdSet::from([first.feed_id, second.feed_id]);

        let reversed = PriceUpdates::from_bytes(price_updates_bytes(&[second, first])).unwrap();
        assert!(reversed.matches_set(&expected));

        let repeated = PriceUpdates::from_bytes(price_updates_bytes(&[first, first])).unwrap();
        assert!(!repeated.matches_set(&expected));
    }

    #[test]
    fn price_update_and_updates_construct_alike() {
        let feeds = [test_feed(1, 42)];
        let bytes = price_updates_bytes(&feeds);

        let update = PriceUpdate::from_slice(&bytes).unwrap();
        assert_eq!(update, PriceUpdate::from_bytes(bytes.clone()).unwrap());
        assert_eq!(
            update,
            PriceUpdate::try_from(BytesMut::from(&bytes[..])).unwrap()
        );
        assert_eq!(update.bytes(), &bytes);

        let updates = PriceUpdates::from_slice(&bytes).unwrap();
        assert_eq!(updates, PriceUpdates::from_bytes(bytes.clone()).unwrap());
        assert_eq!(
            updates,
            PriceUpdates::try_from(BytesMut::from(&bytes[..])).unwrap()
        );
        assert_eq!(updates.into_bytes(), bytes);
    }

    /// Hermes update of USDC/USD and USDT/USD with a 952-byte VAA and 12 proof hashes per feed.
    const HERMES_PRICE_UPDATES: &[u8] = &[
        80, 78, 65, 85, 1, 0, 0, 0, 3, 184, 1, 0, 0, 0, 4, 13, 2, 227, 24, 96, 99, 71, 25, 153,
        168, 64, 62, 5, 239, 214, 91, 168, 20, 170, 63, 109, 144, 13, 7, 115, 70, 224, 77, 98, 81,
        30, 84, 91, 39, 13, 232, 125, 67, 36, 53, 99, 202, 197, 133, 80, 74, 157, 12, 189, 220, 28,
        151, 199, 188, 225, 109, 209, 249, 96, 183, 5, 42, 226, 192, 38, 138, 0, 3, 249, 120, 36,
        58, 7, 92, 85, 172, 179, 237, 151, 245, 248, 82, 75, 158, 19, 63, 114, 199, 222, 9, 163,
        231, 65, 141, 52, 212, 96, 238, 6, 49, 87, 79, 72, 233, 130, 88, 55, 156, 73, 61, 84, 147,
        109, 16, 223, 189, 158, 197, 53, 244, 65, 187, 33, 120, 99, 149, 185, 191, 242, 92, 66,
        191, 0, 4, 30, 240, 168, 168, 247, 51, 78, 48, 103, 240, 249, 4, 192, 211, 169, 152, 41,
        145, 18, 28, 200, 5, 38, 124, 61, 24, 19, 172, 116, 80, 164, 66, 21, 133, 130, 184, 8, 98,
        220, 30, 120, 114, 33, 149, 117, 191, 18, 181, 131, 235, 27, 36, 209, 115, 130, 163, 6, 23,
        124, 236, 171, 124, 112, 44, 1, 6, 209, 141, 41, 100, 143, 78, 41, 21, 243, 59, 150, 177,
        90, 178, 237, 29, 145, 149, 37, 46, 240, 3, 195, 241, 80, 245, 180, 84, 61, 14, 170, 237,
        122, 201, 27, 198, 73, 186, 234, 154, 66, 207, 67, 75, 240, 174, 249, 142, 48, 228, 111, 4,
        200, 190, 172, 246, 63, 254, 34, 163, 128, 122, 16, 210, 0, 8, 191, 220, 51, 253, 57, 50,
        163, 79, 88, 232, 126, 172, 170, 89, 130, 65, 176, 182, 91, 240, 164, 35, 213, 132, 158,
        97, 102, 50, 162, 96, 114, 139, 98, 157, 119, 102, 231, 146, 166, 8, 6, 18, 60, 177, 86,
        233, 26, 235, 227, 155, 225, 197, 132, 61, 6, 118, 109, 248, 24, 140, 30, 52, 6, 43, 1, 10,
        85, 29, 249, 163, 156, 13, 212, 77, 180, 47, 238, 93, 158, 18, 64, 40, 222, 167, 1, 33,
        140, 168, 197, 159, 241, 17, 243, 0, 58, 92, 199, 31, 89, 162, 159, 123, 3, 64, 152, 23,
        197, 125, 37, 84, 80, 165, 182, 200, 181, 78, 196, 118, 123, 1, 70, 177, 37, 116, 240, 202,
        224, 220, 109, 158, 1, 11, 221, 209, 41, 251, 32, 55, 58, 61, 152, 110, 10, 62, 186, 60,
        86, 65, 160, 71, 81, 1, 59, 126, 69, 71, 216, 204, 52, 229, 0, 185, 127, 57, 14, 206, 70,
        155, 120, 85, 117, 170, 225, 235, 12, 190, 234, 184, 58, 164, 225, 5, 12, 223, 251, 0, 52,
        59, 221, 225, 126, 233, 217, 92, 116, 213, 0, 12, 152, 157, 113, 192, 198, 238, 172, 110,
        191, 190, 246, 149, 85, 188, 218, 83, 7, 104, 78, 189, 150, 0, 236, 198, 121, 181, 51, 177,
        154, 109, 144, 89, 125, 120, 175, 40, 22, 245, 71, 29, 146, 119, 31, 193, 103, 5, 250, 72,
        228, 130, 239, 17, 240, 130, 196, 167, 133, 169, 212, 235, 207, 43, 210, 171, 1, 13, 172,
        96, 175, 189, 130, 166, 32, 89, 101, 136, 219, 122, 111, 245, 232, 150, 200, 191, 50, 211,
        176, 232, 11, 87, 250, 222, 112, 160, 149, 249, 101, 52, 88, 103, 53, 13, 80, 101, 152, 75,
        5, 191, 204, 201, 122, 107, 135, 35, 88, 199, 254, 233, 74, 253, 15, 116, 58, 170, 106,
        100, 219, 174, 254, 183, 0, 14, 112, 221, 237, 236, 198, 85, 71, 134, 82, 99, 128, 145, 28,
        187, 213, 66, 75, 75, 9, 124, 139, 242, 194, 156, 80, 157, 44, 46, 189, 214, 30, 32, 22,
        219, 212, 50, 92, 236, 147, 183, 177, 185, 71, 140, 28, 189, 62, 96, 176, 208, 55, 230, 78,
        248, 218, 35, 242, 234, 238, 14, 212, 97, 166, 75, 1, 15, 169, 162, 176, 251, 32, 122, 144,
        165, 152, 196, 29, 22, 221, 250, 160, 216, 139, 215, 33, 58, 218, 234, 26, 248, 82, 215,
        204, 15, 96, 79, 31, 182, 22, 97, 57, 75, 199, 107, 14, 2, 18, 146, 69, 92, 159, 99, 120,
        191, 171, 3, 156, 101, 66, 131, 158, 82, 117, 42, 117, 67, 57, 141, 168, 28, 1, 16, 247, 2,
        110, 59, 2, 102, 15, 43, 243, 29, 48, 44, 244, 115, 217, 192, 162, 199, 181, 250, 26, 227,
        46, 145, 85, 190, 125, 226, 207, 225, 151, 247, 42, 140, 96, 25, 47, 210, 129, 51, 67, 133,
        201, 98, 151, 19, 159, 167, 59, 177, 193, 240, 217, 142, 41, 26, 226, 2, 34, 62, 39, 94,
        156, 232, 1, 17, 175, 70, 184, 165, 201, 132, 123, 10, 1, 152, 213, 221, 143, 15, 127, 102,
        4, 200, 101, 172, 116, 105, 115, 239, 27, 66, 67, 7, 69, 215, 19, 158, 54, 227, 2, 53, 97,
        243, 163, 250, 62, 100, 112, 39, 242, 95, 186, 138, 251, 250, 229, 5, 133, 6, 225, 249,
        221, 150, 166, 18, 21, 71, 247, 142, 1, 104, 63, 14, 207, 0, 0, 0, 0, 0, 26, 225, 1, 250,
        237, 172, 88, 81, 227, 43, 155, 35, 181, 249, 65, 26, 140, 43, 172, 74, 174, 62, 212, 221,
        123, 129, 29, 209, 167, 46, 164, 170, 113, 0, 0, 0, 0, 8, 31, 235, 202, 1, 65, 85, 87, 86,
        0, 0, 0, 0, 0, 13, 45, 109, 26, 0, 0, 39, 16, 10, 180, 224, 234, 57, 33, 123, 195, 241, 46,
        211, 107, 68, 85, 235, 149, 196, 95, 190, 111, 2, 0, 85, 0, 234, 160, 32, 198, 28, 196,
        121, 113, 40, 19, 70, 28, 225, 83, 137, 74, 150, 166, 192, 11, 33, 237, 12, 252, 39, 152,
        209, 249, 169, 233, 201, 74, 0, 0, 0, 0, 5, 245, 137, 177, 0, 0, 0, 0, 0, 1, 173, 206, 255,
        255, 255, 248, 0, 0, 0, 0, 104, 63, 14, 207, 0, 0, 0, 0, 104, 63, 14, 206, 0, 0, 0, 0, 5,
        245, 135, 212, 0, 0, 0, 0, 0, 1, 163, 1, 12, 59, 102, 57, 97, 120, 243, 39, 75, 249, 134,
        125, 242, 206, 38, 134, 17, 150, 44, 146, 76, 241, 107, 157, 216, 217, 170, 43, 18, 17,
        227, 13, 103, 49, 173, 226, 195, 235, 215, 254, 89, 241, 120, 15, 3, 70, 9, 118, 41, 60,
        125, 193, 189, 62, 158, 1, 21, 216, 218, 234, 190, 94, 186, 201, 201, 211, 111, 180, 93,
        250, 13, 40, 61, 191, 149, 246, 78, 42, 86, 28, 112, 229, 220, 15, 66, 200, 80, 170, 108,
        96, 175, 200, 103, 77, 39, 57, 241, 217, 167, 70, 198, 74, 7, 203, 19, 145, 241, 242, 61,
        152, 213, 227, 81, 121, 83, 202, 207, 24, 215, 179, 29, 184, 162, 32, 199, 60, 119, 46,
        246, 1, 134, 69, 66, 26, 126, 207, 182, 189, 208, 142, 189, 21, 28, 184, 186, 26, 235, 67,
        250, 102, 225, 215, 63, 29, 56, 232, 247, 24, 1, 51, 116, 69, 38, 195, 244, 248, 180, 50,
        43, 104, 58, 203, 159, 30, 53, 114, 207, 143, 42, 28, 167, 75, 204, 97, 2, 222, 80, 53,
        178, 137, 26, 243, 203, 140, 83, 97, 75, 172, 245, 248, 76, 139, 58, 225, 142, 207, 32, 15,
        69, 159, 149, 236, 65, 146, 92, 99, 233, 61, 66, 165, 230, 18, 50, 243, 5, 47, 247, 1, 145,
        156, 73, 65, 244, 92, 190, 23, 37, 180, 126, 155, 198, 0, 85, 0, 43, 137, 185, 220, 143,
        223, 159, 52, 112, 154, 91, 16, 107, 71, 47, 15, 57, 187, 108, 169, 206, 4, 176, 253, 127,
        46, 151, 22, 136, 226, 229, 59, 0, 0, 0, 0, 5, 246, 228, 237, 0, 0, 0, 0, 0, 0, 245, 28,
        255, 255, 255, 248, 0, 0, 0, 0, 104, 63, 14, 207, 0, 0, 0, 0, 104, 63, 14, 206, 0, 0, 0, 0,
        5, 246, 198, 226, 0, 0, 0, 0, 0, 0, 238, 115, 12, 25, 196, 58, 108, 30, 84, 208, 12, 80,
        67, 9, 137, 31, 95, 121, 104, 222, 40, 236, 228, 119, 232, 203, 135, 231, 159, 142, 125,
        131, 59, 142, 30, 106, 236, 110, 62, 201, 199, 87, 89, 208, 80, 128, 78, 67, 108, 208, 27,
        98, 210, 56, 122, 86, 140, 91, 35, 230, 10, 150, 225, 245, 69, 141, 19, 245, 33, 175, 204,
        82, 113, 138, 28, 81, 255, 237, 128, 239, 38, 155, 12, 194, 83, 191, 149, 143, 4, 243, 4,
        124, 172, 77, 176, 90, 41, 130, 63, 245, 165, 181, 217, 246, 165, 105, 205, 226, 193, 170,
        150, 170, 44, 192, 62, 147, 214, 143, 243, 89, 182, 76, 73, 201, 35, 194, 60, 149, 118, 78,
        143, 237, 202, 62, 38, 11, 131, 165, 23, 199, 225, 248, 38, 25, 9, 20, 63, 136, 146, 201,
        252, 7, 210, 157, 124, 142, 174, 215, 157, 168, 52, 174, 112, 43, 172, 168, 228, 12, 65,
        32, 7, 137, 2, 23, 201, 115, 89, 209, 151, 11, 86, 147, 213, 250, 164, 241, 244, 174, 66,
        47, 165, 81, 114, 139, 189, 101, 212, 89, 149, 66, 219, 71, 186, 184, 228, 173, 170, 31, 5,
        7, 144, 218, 34, 141, 15, 197, 201, 124, 159, 133, 70, 119, 16, 18, 50, 243, 5, 47, 247, 1,
        145, 156, 73, 65, 244, 92, 190, 23, 37, 180, 126, 155, 198,
    ];

    /// Offset of the VAA, after the magic, version, trailing payload length, proof type and VAA
    /// length.
    const HERMES_VAA_START: usize = 4 + 1 + 1 + 1 + 1 + 2;
    const HERMES_VAA_LEN: usize = 952;

    fn hermes_feed(
        feed_id: &str,
        price: i64,
        confidence: u64,
        ema_price: i64,
        ema_confidence: u64,
    ) -> Feed {
        let timestamp = 1_748_963_023;
        Feed {
            feed_id: feed_id.parse().unwrap(),
            price: Price {
                price,
                confidence,
                exponent: -8,
                timestamp,
            },
            ema_price: Price {
                price: ema_price,
                confidence: ema_confidence,
                exponent: -8,
                timestamp,
            },
        }
    }

    #[test]
    fn price_updates_parses_hermes_payload() {
        let feeds = [
            hermes_feed(
                "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a",
                99_977_649,
                110_030,
                99_977_172,
                107_265,
            ),
            hermes_feed(
                "0x2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b",
                100_066_541,
                62_748,
                100_058_850,
                61_043,
            ),
        ];
        let updates = PriceUpdates::from_bytes(Bytes::from_static(HERMES_PRICE_UPDATES)).unwrap();

        assert_eq!(updates.feed_count(), 2);
        assert_eq!(updates.feeds().collect::<Vec<_>>(), feeds);
        assert_eq!(
            updates.vaa(),
            &HERMES_PRICE_UPDATES[HERMES_VAA_START..HERMES_VAA_START + HERMES_VAA_LEN],
        );
        assert!(updates.matches_set(&PriceFeedIdSet::from(feeds.map(|feed| feed.feed_id))));

        let base64 = Base64::encode_string(HERMES_PRICE_UPDATES);
        assert_eq!(PriceUpdates::from_base64(base64).unwrap(), updates);
    }

    #[test]
    fn price_updates_rejects_malformed_hermes_payload() {
        let modified = |index: usize, byte: u8| {
            let mut bytes = HERMES_PRICE_UPDATES.to_vec();
            bytes[index] = byte;
            bytes
        };
        let mut trailing_data = HERMES_PRICE_UPDATES.to_vec();
        trailing_data.push(0);
        let message_start = HERMES_VAA_START + HERMES_VAA_LEN + 1;

        let cases = [
            ("invalid magic", modified(0, 0)),
            ("invalid major version", modified(4, 2)),
            ("invalid proof type", modified(7, 1)),
            ("invalid message length", modified(message_start + 1, 84)),
            ("invalid message type", modified(message_start + 2, 1)),
            ("trailing data", trailing_data),
            (
                "truncated proof",
                HERMES_PRICE_UPDATES[..HERMES_PRICE_UPDATES.len() - 1].to_vec(),
            ),
            (
                "truncated VAA",
                HERMES_PRICE_UPDATES[..HERMES_VAA_START + HERMES_VAA_LEN - 1].to_vec(),
            ),
        ];
        for (name, bytes) in cases {
            assert!(PriceUpdates::from_slice(&bytes).is_err(), "{name}");
        }
        assert!(PriceUpdate::from_bytes(Bytes::from_static(HERMES_PRICE_UPDATES)).is_err());
    }
}
