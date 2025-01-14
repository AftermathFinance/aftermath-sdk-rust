mod data;

pub use self::data::{
    GasData,
    ImmOwnedOrReceivingError,
    ObjectArg,
    TransactionData,
    TransactionDataAPI,
    TransactionDataV1,
};

/// Temporarily here just to enable [`CheckpointTransaction`] serde.
///
/// [`CheckpointTransaction`]: crate::CheckpointTransaction
pub(crate) mod _serde {
    // Copyright (c) Mysten Labs, Inc.
    // SPDX-License-Identifier: Apache-2.0
    use serde::ser::SerializeSeq as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_with::{serde_as, DeserializeAs, SerializeAs};
    use sui_sdk_types::types::{SignedTransaction, Transaction, UserSignature};

    #[expect(clippy::redundant_pub_crate, reason = "wanna keep it explicit")]
    pub(crate) struct SignedTransactionWithIntentMessage;

    #[serde_as]
    #[derive(Serialize)]
    struct BinarySignedTransactionWithIntentMessageRef<'a> {
        #[serde_as(as = "IntentMessageWrappedTransaction")]
        transaction: &'a Transaction,
        signatures: &'a Vec<UserSignature>,
    }

    #[serde_as]
    #[derive(Deserialize)]
    struct BinarySignedTransactionWithIntentMessage {
        #[serde_as(as = "IntentMessageWrappedTransaction")]
        transaction: Transaction,
        signatures: Vec<UserSignature>,
    }

    impl SerializeAs<SignedTransaction> for SignedTransactionWithIntentMessage {
        fn serialize_as<S>(
            transaction: &SignedTransaction,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                transaction.serialize(serializer)
            } else {
                let SignedTransaction {
                    transaction,
                    signatures,
                } = transaction;
                let binary = BinarySignedTransactionWithIntentMessageRef {
                    transaction,
                    signatures,
                };

                let mut s = serializer.serialize_seq(Some(1))?;
                s.serialize_element(&binary)?;
                s.end()
            }
        }
    }

    impl<'de> DeserializeAs<'de, SignedTransaction> for SignedTransactionWithIntentMessage {
        fn deserialize_as<D>(deserializer: D) -> Result<SignedTransaction, D::Error>
        where
            D: Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                SignedTransaction::deserialize(deserializer)
            } else {
                struct V;
                impl<'de> serde::de::Visitor<'de> for V {
                    type Value = SignedTransaction;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("expected a sequence with length 1")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        if seq.size_hint().is_some_and(|size| size != 1) {
                            return Err(serde::de::Error::custom(
                                "expected a sequence with length 1",
                            ));
                        }

                        let BinarySignedTransactionWithIntentMessage {
                            transaction,
                            signatures,
                        } = seq.next_element()?.ok_or_else(|| {
                            serde::de::Error::custom("expected a sequence with length 1")
                        })?;
                        Ok(SignedTransaction {
                            transaction,
                            signatures,
                        })
                    }
                }

                deserializer.deserialize_seq(V)
            }
        }
    }

    /// serde implementation that serializes a transaction prefixed with the signing intent. See
    /// [struct Intent] for more info.
    ///
    /// So we need to serialize Transaction as (0, 0, 0, Transaction)
    struct IntentMessageWrappedTransaction;

    impl SerializeAs<Transaction> for IntentMessageWrappedTransaction {
        fn serialize_as<S>(transaction: &Transaction, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            use serde::ser::SerializeTuple;

            let mut s = serializer.serialize_tuple(4)?;
            s.serialize_element(&0u8)?;
            s.serialize_element(&0u8)?;
            s.serialize_element(&0u8)?;
            s.serialize_element(transaction)?;
            s.end()
        }
    }

    impl<'de> DeserializeAs<'de, Transaction> for IntentMessageWrappedTransaction {
        fn deserialize_as<D>(deserializer: D) -> Result<Transaction, D::Error>
        where
            D: Deserializer<'de>,
        {
            let (scope, version, app, transaction): (u8, u8, u8, Transaction) =
                Deserialize::deserialize(deserializer)?;
            match (scope, version, app) {
                (0, 0, 0) => {}
                _ => {
                    return Err(serde::de::Error::custom(format!(
                        "invalid intent message ({scope}, {version}, {app})"
                    )))
                }
            }

            Ok(transaction)
        }
    }
}
