use std::sync::LazyLock;

use af_sui_types::ObjectId;
use extension_traits::extension;
use jsonrpsee_core::ClientError;
use jsonrpsee_types::{ErrorCode, ErrorObject, ErrorObjectOwned};

pub type JsonRpcClientResult<T = ()> = Result<T, JsonRpcClientError>;

pub type JsonRpcClientError = ClientError;

static OBJECT_UNAVAILABLE_FOR_CONSUMPTION_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::RegexBuilder::new("Object .* not available for consumption, current version:")
        .case_insensitive(true)
        .build()
        .expect("Tested below for panics")
});

/// Breakdown:
/// - "object_id: ([[:alnum:]]+)" captures the object id
/// - "version: (?:...|None)" matches either the first pattern or "None", but doesn't capture its
///   content
/// - "Some\(SequenceNumber\((\d+)\)\)": captures the object version
static OBJECT_NOT_FOUND_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::RegexBuilder::new(r"Error checking transaction input objects: ObjectNotFound \{ object_id: ([[:alnum:]]+), version: (?:Some\(SequenceNumber\((\d+)\)\)|None) \}")
        .case_insensitive(true)
        .build()
        .expect("Tested below for panics")
});

static RETRIED_TRANSACTION_SUCCESS_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::RegexBuilder::new(r"Failed to sign transaction by a quorum of validators because one or more of its objects is reserved for another transaction. Retried transaction [[:word:]]+ \(succeeded\) because it was able to gather the necessary votes.")
        .case_insensitive(true)
        .build()
        .expect("Tested below for panics")
});

/// Helpers to inspect the error type from the client implementation.
#[extension(pub trait JsonRpcClientErrorExt)]
impl JsonRpcClientError {
    /// If this is a JSON-RPC [error object], return a reference to it.
    ///
    /// If submitting a transaction, these are usually pre-execution validity check failures, e.g.,
    /// wrong signatures, equivocated objects, etc. Use [`ErrorObjectExt`] to extract more
    /// information from those.
    ///
    /// [error object]: https://www.jsonrpc.org/specification#response_object
    fn as_error_object(&self) -> Option<&ErrorObjectOwned> {
        match &self {
            Self::Call(err_obj) => Some(err_obj),
            _ => None,
        }
    }
}

/// Helpers to inspect the error type from JSON-RPC calls.
///
/// JSON-RPC [error object] codes taken from the [original implementation].
///
/// See the [source] for codes and messages in the quorum driver FN API.
///
/// [error object]: https://www.jsonrpc.org/specification#response_object
/// [original implementation]: https://github.com/MystenLabs/sui/blob/main/crates/sui-json-rpc-api/src/lib.rs
/// [source]: https://github.com/MystenLabs/sui/blob/testnet-v1.35.1/crates/sui-json-rpc/src/error.rs
#[extension(pub trait ErrorObjectExt)]
impl<'a> ErrorObject<'a> {
    const TRANSIENT_ERROR_CODE: i32 = -32050;
    const TRANSACTION_EXECUTION_CLIENT_ERROR_CODE: i32 = -32002;

    /// Transient error, suggesting it may be possible to retry.
    ///
    /// # Example error messages
    ///
    /// - Transaction timed out before reaching finality
    /// - Transaction failed to reach finality with transient error after X attempts
    /// - Transaction is not processed because [...] of validators by stake are overloaded with
    ///   certificates pending execution.
    fn is_transient_error(&self) -> bool {
        self.code() == Self::TRANSIENT_ERROR_CODE
    }

    /// Error in transaction execution (pre-consensus)
    ///
    /// # Example error messages
    ///
    /// - Invalid user signature
    /// - Failed to sign transaction by a quorum of validators because one or more of its objects
    ///   is {reason}. {retried_info} Other transactions locking these objects: [...]
    ///   - reason:
    ///     - equivocated until the next epoch
    ///     - reserved for another transaction
    ///   - retried_info: Retried transaction [...] (success/failure) because it was able to
    ///     gather the necessary votes
    /// - Transaction validator signing failed due to issues with transaction inputs, please review
    ///   the errors and try again: {reason}
    ///   - reason:
    ///     - Balance of gas object [...] is lower than the needed amount
    ///     - Object [...] not available for consumption, current version: [...]
    ///     - Could not find the referenced object [...] at version [...]
    fn is_execution_error(&self) -> bool {
        self.code() == Self::TRANSACTION_EXECUTION_CLIENT_ERROR_CODE
    }

    /// Error with message "Object [...] not available for consumption, current version: [...]".
    ///
    /// TLDR: usually happens when the client's state sync is lagging too much.
    ///
    /// Note that this may not be the single reason why the transaction failed. Other errors
    /// related to the transaction inputs may be present.
    ///
    /// May be due to state sync lag. For example, the client submits two transactions in quick
    /// succession but doesn't sync owned objects quick enough for the second transaction, therefore
    /// it uses the same owned object reference twice.
    fn is_object_unavailable_for_consumption(&self) -> bool {
        OBJECT_UNAVAILABLE_FOR_CONSUMPTION_REGEX.is_match(self.message())
    }

    /// Like [`Self::as_object_not_found`], but doesn't extract the object id and version.
    fn is_object_not_found(&self) -> bool {
        if self.code() != ErrorCode::InvalidParams.code() {
            return false;
        }
        OBJECT_NOT_FOUND_REGEX.is_match(self.message())
    }

    /// Whether the transaction didn't fail because the RPC retried the submission and it
    /// succeeded.
    ///
    /// The message for such an error looks like:
    /// ```text
    /// Failed to sign transaction by a quorum of validators because one or more of its objects is reserved for another transaction. Retried transaction [...] (succeeded) because it was able to gather the necessary votes
    /// ```
    fn is_transaction_retried_success(&self) -> bool {
        self.is_execution_error() && RETRIED_TRANSACTION_SUCCESS_REGEX.is_match(self.message())
    }

    /// Error with [`InvalidParams`] code and message "Error checking transaction input objects:
    /// ObjectNotFound { ... }"
    ///
    /// TLDR: usually happens when the client's state sync is faster than the full node's.
    ///
    /// Seen in practice when dry-running a transaction and the full node hasn't synchronized yet
    /// with the effects of a previous one.
    ///
    /// [`InvalidParams`]: ErrorCode::InvalidParams
    fn as_object_not_found(&self) -> Option<(ObjectId, Option<u64>)> {
        if self.code() != ErrorCode::InvalidParams.code() {
            return None;
        }
        let captures = OBJECT_NOT_FOUND_REGEX.captures(self.message())?;
        // Version may be None so we don't use `?` after `.get()`
        let version = captures.get(2).and_then(|c| c.as_str().parse().ok());
        Some((captures.get(1)?.as_str().parse().ok()?, version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_unavailable_for_consumption_regex_builds() {
        let _ = &*OBJECT_UNAVAILABLE_FOR_CONSUMPTION_REGEX;
    }

    #[test]
    fn object_not_found_regex_builds() {
        let _ = &*OBJECT_NOT_FOUND_REGEX;
    }

    /// Toy example
    #[test]
    fn object_unavailable_for_consumption_match1() {
        let expect = "Transaction validator signing failed due to issues with transaction inputs, \
            please review the errors and try again:\n\
            - Balance of gas object 10 is lower than the needed amount: 100\n\
            - Object ID 0x0000000000000000000000000000000000000000000000000000000000000000 \
              Version 0x0 \
              Digest 11111111111111111111111111111111 \
              is not available for consumption, current version: 0xa";
        assert!(OBJECT_UNAVAILABLE_FOR_CONSUMPTION_REGEX.is_match(expect));
    }

    /// Real-world example
    #[test]
    fn object_unavailable_for_consumption_match2() {
        let expect = "Transaction validator signing failed due to issues with transaction inputs, \
            please review the errors and try again:\n\
            - Object ID 0xa3b25765e4f7f4524367fa792b608483157bfef919108f0d998c6980493fc7bc \
              Version 0xb0cb7f7 \
              Digest FKkELfAR3vP19MrjwEwTapH3JDbdZuxqcC25CoALNUsN \
              is not available for consumption, current version: 0xb0cb7f8";
        assert!(OBJECT_UNAVAILABLE_FOR_CONSUMPTION_REGEX.is_match(expect));
    }

    /// Real-world example
    #[test]
    fn object_unavailable_for_consumption_not_match1() {
        let expect = "Transaction validator signing failed due to issues with transaction inputs, \
            please review the errors and try again:\n\
            - Transaction was not signed by the correct sender: \
              Object 0x2d13a698a9ef878372210f6d96e2315f368794e1c9c842f6fddacb3815ff749d is owned \
              by account address \
              0x162602a3f40fcab9b513a3fefad1c046ae242bb6bb83334b3aa8cd639e018b28, but given \
              owner/signer address is \
              0x76f9ca7f89994d4039b739859a41b39123d6f695a1b33f7431cee3c6b40a45c2";
        assert!(!OBJECT_UNAVAILABLE_FOR_CONSUMPTION_REGEX.is_match(expect));
    }

    /// Real-world example
    ///
    /// ErrorObject {
    ///     code: InvalidParams,
    ///     message: "Error checking transaction input objects: ObjectNotFound { object_id: 0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a, version: Some(SequenceNumber(247815626)) }",
    ///     data: None
    /// }
    #[test]
    fn object_not_found_match1() {
        let expect = "Error checking transaction input objects: ObjectNotFound { object_id: 0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a, version: Some(SequenceNumber(247815626)) }";
        assert!(OBJECT_NOT_FOUND_REGEX.is_match(expect));
        let matches = OBJECT_NOT_FOUND_REGEX
            .captures(expect)
            .expect("object_id and version present");
        assert_eq!(
            matches.get(1).expect("object_id").as_str(),
            "0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a"
        );
        assert_eq!(matches.get(2).expect("version").as_str(), "247815626");
    }

    /// Hypothetical
    ///
    /// ErrorObject {
    ///     code: InvalidParams,
    ///     message: "Error checking transaction input objects: ObjectNotFound { object_id: 0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a, version: None }",
    ///     data: None
    /// }
    #[test]
    fn object_not_found_match2() {
        let expect = "Error checking transaction input objects: ObjectNotFound { object_id: 0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a, version: None }";
        assert!(OBJECT_NOT_FOUND_REGEX.is_match(expect));
        let matches = OBJECT_NOT_FOUND_REGEX
            .captures(expect)
            .expect("object_id present");
        dbg!(&matches);
        assert_eq!(
            matches.get(1).expect("object_id").as_str(),
            "0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a"
        );
        assert_eq!(matches.get(2), None);
    }

    #[test]
    fn object_not_found1() {
        let error = ErrorObject::owned::<()>(
            ErrorCode::InvalidParams.code(),
            "Error checking transaction input objects: ObjectNotFound { object_id: 0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a, version: Some(SequenceNumber(247815626)) }",
            None,
        );
        assert!(error.is_object_not_found());
        assert_eq!(
            error.as_object_not_found(),
            Some((
                "0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a"
                    .parse()
                    .unwrap(),
                Some(247815626)
            ))
        )
    }

    #[test]
    fn object_not_found2() {
        let error = ErrorObject::owned::<()>(
            ErrorCode::InvalidParams.code(),
            "Error checking transaction input objects: ObjectNotFound { object_id: 0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a, version: None }",
            None,
        );
        assert!(error.is_object_not_found());
        assert_eq!(
            error.as_object_not_found(),
            Some((
                "0x38826d19eb0338509eedf78c4f3a1de6479163e1a0a2fb447aa3fe947ef4cc2a"
                    .parse()
                    .unwrap(),
                None
            ))
        )
    }

    #[test]
    fn retried_transaction_regex_matches() {
        let message = r#"Failed to sign transaction by a quorum of validators because one or more of its objects is reserved for another transaction. Retried transaction EENuLfRygexZrE1ycfsUHRFFenAgPLReFxYTFKQg9Jpu (succeeded) because it was able to gather the necessary votes. Other transactions locking these objects:\n- EENuLfRygexZrE1ycfsUHRFFenAgPLReFxYTFKQg9Jpu (stake 90.17)"#;
        assert!(RETRIED_TRANSACTION_SUCCESS_REGEX.is_match(message));
    }
}
