use sui_sdk_types::hash::Hasher;
use sui_sdk_types::{Intent, IntentAppId, IntentScope, IntentVersion, SigningDigest};

use crate::sui::transaction::ObjectArg;
use crate::{Digest, Object, ObjectDigest, Owner, TransactionData, TransactionDigest};

impl Object {
    pub fn digest(&self) -> ObjectDigest {
        const SALT: &str = "Object::";
        let digest = type_digest(SALT, self);
        ObjectDigest::new(digest.into_inner())
    }

    /// Input for transactions that interact with this object.
    pub fn object_arg(&self, mutable: bool) -> ObjectArg {
        use Owner::*;
        let id = self.id();
        match self.owner {
            AddressOwner(_) | ObjectOwner(_) | Immutable => {
                ObjectArg::ImmOrOwnedObject((id, self.version(), self.digest()))
            }
            Shared {
                initial_shared_version,
            }
            | ConsensusV2 {
                start_version: initial_shared_version,
                ..
            } => ObjectArg::SharedObject {
                id,
                initial_shared_version,
                mutable,
            },
        }
    }
}

impl TransactionData {
    pub fn digest(&self) -> TransactionDigest {
        const SALT: &str = "TransactionData::";
        let digest = type_digest(SALT, self);
        TransactionDigest::new(digest.into_inner())
    }
}

fn type_digest<T: serde::Serialize>(salt: &str, ty: &T) -> Digest {
    let mut hasher = Hasher::new();
    hasher.update(salt);
    bcs::serialize_into(&mut hasher, ty).expect("All types used are BCS-compatible");
    Digest::new(hasher.finalize().into_inner())
}

impl TransactionData {
    pub fn signing_digest(&self) -> SigningDigest {
        const INTENT: Intent = Intent {
            scope: IntentScope::TransactionData,
            version: IntentVersion::V0,
            app_id: IntentAppId::Sui,
        };
        let digest = signing_digest(INTENT, self);
        digest.into_inner()
    }
}

fn signing_digest<T: serde::Serialize + ?Sized>(intent: Intent, ty: &T) -> sui_sdk_types::Digest {
    let mut hasher = Hasher::new();
    hasher.update(intent.to_bytes());
    bcs::serialize_into(&mut hasher, ty).expect("T is BCS-compatible");
    hasher.finalize()
}
