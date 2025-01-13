#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Move types for the `Wormhole` Sui package.

use sui_framework_sdk::table::Table;

af_sui_pkg_sdk::sui_pkg_sdk!(wormhole {
    module vaa {
        /// Container storing verified Wormhole message info. This struct also
        /// caches the digest, which is a double Keccak256 hash of the message body.
        struct VAA {
            /// Guardian set index of Guardians that attested to observing the
            /// Wormhole message.
            guardian_set_index: u32,
            /// Time when Wormhole message was emitted or observed.
            timestamp: u32,
            /// A.K.A. Batch ID.
            nonce: u32,
            /// Wormhole chain ID from which network the message originated from.
            emitter_chain: u16,
            /// Address of contract (standardized to 32 bytes) that produced the
            /// message.
            emitter_address: external_address::ExternalAddress,
            /// Sequence number of emitter's Wormhole message.
            sequence: u64,
            /// A.K.A. Finality.
            consistency_level: u8,
            /// Arbitrary payload encoding data relevant to receiver.
            payload: vector<u8>,

            /// Double Keccak256 hash of message body.
            digest: bytes32::Bytes32
        }
    }

    module external_address {
        /// Container for `Bytes32`.
        struct ExternalAddress has copy, drop, store {
            value: bytes32::Bytes32,
        }
    }

    module consumed_vaas {
        /// Container storing VAA hashes (digests). This will be checked against in
        /// `parse_verify_and_consume` so a particular VAA cannot be replayed. It
        /// is up to the integrator to have this container live in his contract
        /// in order to take advantage of this no-replay protection. Or an
        /// integrator can implement his own method to prevent replay.
        struct ConsumedVAAs has store {
            hashes: set::Set<bytes32::Bytes32>
        }
    }

    module set {
        /// Empty struct. Used as the value type in mappings to encode a set
        struct Empty has store, drop {}

        /// A set containing elements of type `T` with support for membership
        /// checking.
        struct Set<!phantom T: copy + drop + store> has store {
            items: Table<T, Empty>
        }
    }

    module bytes32 {
        /// Container for `vector<u8>`, which has length == 32.
        struct Bytes32 has copy, drop, store {
            data: vector<u8>,
        }
    }
});
