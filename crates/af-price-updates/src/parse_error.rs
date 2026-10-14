/// The error returned when parsing a value of this crate from a string, such as a 32-byte hex ID,
/// a sorted set in its `[a,b,...]` form or a Pyth Lazer channel.
///
/// Its message describes the problem; it has no [`source`](std::error::Error::source).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError(Box<str>);

impl ParseError {
    pub(crate) fn new(message: impl Into<Box<str>>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use crate::{
        U256, pyth, pyth_lazer::FeedProperty, sorted_set::SortedSet, stork::EncodedAssetId,
    };

    /// Converts like `?` does into a boxed error, as `anyhow` and `Box<dyn Error>` callers need.
    fn boxed<E: std::error::Error + Send + Sync + 'static>(
        err: E,
    ) -> Box<dyn std::error::Error + Send + Sync> {
        err.into()
    }

    #[test]
    fn hex_parse_errors() {
        for (input, message) in [
            ("", "expected 1 to 64 hex digits"),
            ("0x", "expected 1 to 64 hex digits"),
            (&"1".repeat(65), "expected 1 to 64 hex digits"),
            ("0xabg", "invalid hex character 'g'"),
        ] {
            assert_eq!(input.parse::<U256>().unwrap_err().to_string(), message);
            assert_eq!(
                boxed(input.parse::<pyth::PriceFeedId>().unwrap_err()).to_string(),
                message
            );
            assert_eq!(
                input.parse::<EncodedAssetId>().unwrap_err().to_string(),
                message
            );
        }
    }

    #[test]
    fn list_and_name_parse_errors() {
        assert_eq!(
            "1,2".parse::<SortedSet<u32>>().unwrap_err().to_string(),
            "expected [id,...]: 1,2"
        );
        assert_eq!(
            "[1,x]".parse::<SortedSet<u32>>().unwrap_err().to_string(),
            "invalid digit found in string"
        );
        assert_eq!(
            boxed("bid".parse::<FeedProperty>().unwrap_err()).to_string(),
            "unknown feed property: bid"
        );
    }
}
