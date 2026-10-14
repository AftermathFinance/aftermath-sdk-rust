use base64ct::{Base64, Encoding as _};
use bytes::{Bytes, BytesMut};
use std::io::{Error, ErrorKind};
use std::num::{NonZeroI64, NonZeroU64};

/// Decodes standard, padded base64, in place when `base64` is the only owner of its buffer.
///
/// `oracle` names the format in the error message.
pub(crate) fn decode_base64(base64: Bytes, oracle: &str) -> Result<Bytes, Error> {
    let mut bytes = BytesMut::from(base64);
    let len = Base64::decode_in_place(&mut bytes)
        .map_err(|err| {
            Error::new(
                ErrorKind::InvalidData,
                format!("{oracle} base64 decode error: {err}"),
            )
        })?
        .len();
    bytes.truncate(len);
    Ok(bytes.freeze())
}

/// Cursor over binary oracle updates.
///
/// Typed `take_*` methods read little-endian values; use [`Self::take_chunk`] for other layouts.
#[derive(Clone, Debug)]
pub(crate) struct Reader<'t>(pub(crate) &'t [u8]);

impl<'t> Reader<'t> {
    pub(crate) fn assert_eof(self) -> Result<(), Error> {
        if !self.0.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "unexpected trailing data",
            ));
        }
        Ok(())
    }
    pub(crate) fn assert_remain(&self, n: usize) -> Result<(), Error> {
        if self.0.len() != n {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "unexpected data length",
            ));
        }
        Ok(())
    }
    pub(crate) fn take_chunk<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let (result, reader) = self
            .0
            .split_first_chunk()
            .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, "unexpected end of data"))?;
        self.0 = reader;
        Ok(*result)
    }
    pub(crate) fn take_bool(&mut self) -> Result<bool, Error> {
        let byte = self.take_u8()?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::new(ErrorKind::InvalidData, "invalid boolean value")),
        }
    }
    pub(crate) fn take_u8(&mut self) -> Result<u8, Error> {
        Ok(u8::from_le_bytes(self.take_chunk()?))
    }
    pub(crate) fn take_u16(&mut self) -> Result<u16, Error> {
        Ok(u16::from_le_bytes(self.take_chunk()?))
    }
    pub(crate) fn take_i16(&mut self) -> Result<i16, Error> {
        Ok(i16::from_le_bytes(self.take_chunk()?))
    }
    pub(crate) fn take_u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.take_chunk()?))
    }
    pub(crate) fn take_u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_le_bytes(self.take_chunk()?))
    }
    /// Reads a bool-prefixed optional u64. Returns `Some(value)` when the bool is `true`,
    /// `None` when the bool is `false` (Pyth Lazer's encoding for "not reported").
    pub(crate) fn take_option_u64(&mut self) -> Result<Option<u64>, Error> {
        if self.take_bool()? {
            Ok(Some(self.take_u64()?))
        } else {
            Ok(None)
        }
    }
    pub(crate) fn take_i64(&mut self) -> Result<i64, Error> {
        Ok(i64::from_le_bytes(self.take_chunk()?))
    }
    /// Reads an i64, returning `None` for zero (Pyth Lazer's encoding for an unavailable price).
    pub(crate) fn take_nonzero_i64(&mut self) -> Result<Option<i64>, Error> {
        Ok(NonZeroI64::new(self.take_i64()?).map(NonZeroI64::get))
    }
    /// Reads a u64, returning `None` for zero (Pyth Lazer's encoding for an unavailable price).
    pub(crate) fn take_nonzero_u64(&mut self) -> Result<Option<u64>, Error> {
        Ok(NonZeroU64::new(self.take_u64()?).map(NonZeroU64::get))
    }
    /// Reads a ULEB128 vector length the same way as `sui::bcs::peel_vec_length`: at most five
    /// bytes are read and non-canonical encodings are accepted.
    pub(crate) fn take_vec_length(&mut self) -> Result<usize, Error> {
        let mut total = 0u64;
        for shift in [0, 7, 14, 21, 28] {
            let byte = self.take_u8()?;
            total |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return usize::try_from(total).map_err(|_| {
                    Error::new(ErrorKind::InvalidData, "vector length overflows usize")
                });
            }
        }
        Err(Error::new(
            ErrorKind::InvalidData,
            "vector length is longer than five ULEB128 bytes",
        ))
    }
    pub(crate) fn take(&mut self, n: usize) -> Result<&'t [u8], Error> {
        let (result, reader) = self
            .0
            .split_at_checked(n)
            .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, "unexpected end of data"))?;
        self.0 = reader;
        Ok(result)
    }
    pub(crate) fn skip(&mut self, n: usize) -> Result<(), Error> {
        let _ = self.take(n)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_length(bytes: &[u8]) -> Result<(usize, usize), Error> {
        let mut reader = Reader(bytes);
        let length = reader.take_vec_length()?;
        Ok((length, bytes.len() - reader.0.len()))
    }

    #[test]
    fn take_vec_length() {
        assert_eq!(vec_length(&[0x00]).unwrap(), (0, 1));
        assert_eq!(vec_length(&[0x7f]).unwrap(), (127, 1));
        assert_eq!(vec_length(&[0x80, 0x01]).unwrap(), (128, 2));
        assert_eq!(vec_length(&[0xac, 0x02, 0xff]).unwrap(), (300, 2));
        assert_eq!(vec_length(&[0x80, 0x80, 0x01]).unwrap(), (16_384, 3));
        assert_eq!(
            vec_length(&[0xff, 0xff, 0xff, 0xff, 0x0f]).unwrap(),
            (u32::MAX as usize, 5),
        );
    }

    #[test]
    fn take_vec_length_accepts_non_canonical_encoding() {
        assert_eq!(vec_length(&[0x80, 0x00]).unwrap(), (0, 2));
        assert_eq!(vec_length(&[0x81, 0x80, 0x80, 0x80, 0x00]).unwrap(), (1, 5));
    }

    #[test]
    fn take_vec_length_rejects_more_than_five_bytes() {
        let err = vec_length(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x00]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn take_vec_length_rejects_truncated_input() {
        assert_eq!(
            vec_length(&[]).unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
        assert_eq!(
            vec_length(&[0x80, 0x80]).unwrap_err().kind(),
            ErrorKind::UnexpectedEof,
        );
    }
}
