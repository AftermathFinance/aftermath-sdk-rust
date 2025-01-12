// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::hash::{Hash, Hasher};
use std::str::FromStr;

use af_sui_types::Address as SuiAddress;
pub use enum_dispatch::enum_dispatch;
use fastcrypto::encoding::{Base64, Encoding};
use fastcrypto::error::FastCryptoError;
use fastcrypto::hash::HashFunction as _;
use fastcrypto::traits::ToFromBytes;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::crypto::{
    CompressedSignature,
    DefaultHash,
    Error,
    PublicKey,
    Signature,
    SignatureScheme,
};

pub type WeightUnit = u8;
pub type ThresholdUnit = u16;
pub type BitmapUnit = u16;
pub const MAX_SIGNER_IN_MULTISIG: usize = 10;
pub const MAX_BITMAP_VALUE: BitmapUnit = 0b1111111111;

// =============================================================================
//  MultiSigSigner
// =============================================================================

/// Data needed for signing as a multisig.
#[derive(Deserialize, Debug)]
pub struct MultiSigSigner {
    pub multisig_pk: MultiSigPublicKey,
    /// The indexes of the public keys in `multisig_pk` to sign for.
    pub signers: Vec<usize>,
}

// =============================================================================
//  MultiSig
// =============================================================================

/// The struct that contains signatures and public keys necessary for authenticating a MultiSig.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiSig {
    /// The plain signature encoded with signature scheme.
    sigs: Vec<CompressedSignature>,
    /// A bitmap that indicates the position of which public key the signature should be authenticated with.
    bitmap: BitmapUnit,
    /// The public key encoded with each public key with its signature scheme used along with the corresponding weight.
    multisig_pk: MultiSigPublicKey,
    /// A bytes representation of [struct MultiSig]. This helps with implementing [trait AsRef<[u8]>].
    #[serde(skip)]
    bytes: OnceCell<Vec<u8>>,
}

impl MultiSig {
    /// This combines a list of [enum Signature] `flag || signature || pk` to a MultiSig.
    /// The order of full_sigs must be the same as the order of public keys in
    /// [enum MultiSigPublicKey]. e.g. for [pk1, pk2, pk3, pk4, pk5],
    /// [sig1, sig2, sig5] is valid, but [sig2, sig1, sig5] is invalid.
    pub fn combine(
        full_sigs: Vec<Signature>,
        multisig_pk: MultiSigPublicKey,
    ) -> Result<Self, Error> {
        multisig_pk
            .validate()
            .map_err(|_| Error::InvalidSignature {
                error: "Invalid multisig public key".to_string(),
            })?;

        if full_sigs.len() > multisig_pk.pk_map.len() || full_sigs.is_empty() {
            return Err(Error::InvalidSignature {
                error: "Invalid number of signatures".to_string(),
            });
        }
        let mut bitmap = 0;
        let mut sigs = Vec::with_capacity(full_sigs.len());
        for s in full_sigs {
            let pk = s.to_public_key()?;
            let index = multisig_pk
                .get_index(&pk)
                .ok_or_else(|| Error::IncorrectSigner {
                    error: format!("pk does not exist: {pk:?}"),
                })?;
            if bitmap & (1 << index) != 0 {
                return Err(Error::InvalidSignature {
                    error: "Duplicate public key".to_string(),
                });
            }
            bitmap |= 1 << index;
            sigs.push(s.to_compressed()?);
        }

        Ok(Self {
            sigs,
            bitmap,
            multisig_pk,
            bytes: OnceCell::new(),
        })
    }

    pub fn init_and_validate(&self) -> Result<Self, FastCryptoError> {
        if self.sigs.len() > self.multisig_pk.pk_map.len()
            || self.sigs.is_empty()
            || self.bitmap > MAX_BITMAP_VALUE
        {
            return Err(FastCryptoError::InvalidInput);
        }
        self.multisig_pk.validate()?;
        Ok(self.to_owned())
    }

    pub const fn get_pk(&self) -> &MultiSigPublicKey {
        &self.multisig_pk
    }

    pub fn get_sigs(&self) -> &[CompressedSignature] {
        &self.sigs
    }

    pub fn get_indices(&self) -> Result<Vec<u8>, Error> {
        as_indices(self.bitmap)
    }
}

/// Necessary trait for [struct SenderSignedData].
impl PartialEq for MultiSig {
    fn eq(&self, other: &Self) -> bool {
        self.sigs == other.sigs
            && self.bitmap == other.bitmap
            && self.multisig_pk == other.multisig_pk
    }
}

/// Necessary trait for [struct SenderSignedData].
impl Eq for MultiSig {}

/// Necessary trait for [struct SenderSignedData].
impl Hash for MultiSig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

/// Interpret a bitmap of 01s as a list of indices that is set to 1s.
/// e.g. 22 = 0b10110, then the result is [1, 2, 4].
pub fn as_indices(bitmap: u16) -> Result<Vec<u8>, Error> {
    if bitmap > MAX_BITMAP_VALUE {
        return Err(Error::InvalidSignature {
            error: "Invalid bitmap".to_string(),
        });
    }
    let mut res = Vec::new();
    for i in 0..10 {
        if bitmap & (1 << i) != 0 {
            res.push(i as u8);
        }
    }
    Ok(res)
}

impl ToFromBytes for MultiSig {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        // The first byte matches the flag of MultiSig.
        if bytes.first().ok_or(FastCryptoError::InvalidInput)? != &SignatureScheme::MultiSig.flag()
        {
            return Err(FastCryptoError::InvalidInput);
        }
        let multisig: Self =
            bcs::from_bytes(&bytes[1..]).map_err(|_| FastCryptoError::InvalidSignature)?;
        multisig.init_and_validate()
    }
}

impl FromStr for MultiSig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = Base64::decode(s).map_err(|_| Error::InvalidSignature {
            error: "Invalid base64 string".to_string(),
        })?;
        let sig = Self::from_bytes(&bytes).map_err(|_| Error::InvalidSignature {
            error: "Invalid multisig bytes".to_string(),
        })?;
        Ok(sig)
    }
}

/// This initialize the underlying bytes representation of MultiSig. It encodes
/// [struct MultiSig] as the MultiSig flag (0x03) concat with the bcs bytes
/// of [struct MultiSig] i.e. `flag || bcs_bytes(MultiSig)`.
impl AsRef<[u8]> for MultiSig {
    fn as_ref(&self) -> &[u8] {
        self.bytes
            .get_or_try_init::<_, eyre::Report>(|| {
                let as_bytes = bcs::to_bytes(self).expect("BCS serialization should not fail");
                let mut bytes = Vec::with_capacity(1 + as_bytes.len());
                bytes.push(SignatureScheme::MultiSig.flag());
                bytes.extend_from_slice(as_bytes.as_slice());
                Ok(bytes)
            })
            .expect("OnceCell invariant violated")
    }
}

impl From<MultiSig> for af_sui_types::UserSignature {
    fn from(value: MultiSig) -> Self {
        Self::from_bytes(value.as_bytes()).expect("Compatible")
    }
}

// =============================================================================
//  MultiSigPublicKey
// =============================================================================

/// The struct that contains the public key used for authenticating a MultiSig.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiSigPublicKey {
    /// A list of public key and its corresponding weight.
    pk_map: Vec<(PublicKey, WeightUnit)>,
    /// If the total weight of the public keys corresponding to verified signatures is larger than threshold, the MultiSig is verified.
    threshold: ThresholdUnit,
}

impl MultiSigPublicKey {
    /// Construct MultiSigPublicKey without validation.
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Don't want to risk breaking the API if this uses a non-const init in the future"
    )]
    pub fn insecure_new(pk_map: Vec<(PublicKey, WeightUnit)>, threshold: ThresholdUnit) -> Self {
        Self { pk_map, threshold }
    }

    pub fn new(
        pks: Vec<PublicKey>,
        weights: Vec<WeightUnit>,
        threshold: ThresholdUnit,
    ) -> Result<Self, Error> {
        if pks.is_empty()
            || weights.is_empty()
            || threshold == 0
            || pks.len() != weights.len()
            || pks.len() > MAX_SIGNER_IN_MULTISIG
            || weights.iter().any(|w| *w == 0)
            || weights
                .iter()
                .map(|w| *w as ThresholdUnit)
                .sum::<ThresholdUnit>()
                < threshold
            || pks
                .iter()
                .enumerate()
                .any(|(i, pk)| pks.iter().skip(i + 1).any(|other_pk| *pk == *other_pk))
        {
            return Err(Error::InvalidSignature {
                error: "Invalid multisig public key construction".to_string(),
            });
        }

        Ok(Self {
            pk_map: pks.into_iter().zip(weights).collect(),
            threshold,
        })
    }

    pub fn get_index(&self, pk: &PublicKey) -> Option<u8> {
        self.pk_map.iter().position(|x| &x.0 == pk).map(|x| x as u8)
    }

    pub const fn threshold(&self) -> &ThresholdUnit {
        &self.threshold
    }

    pub const fn pubkeys(&self) -> &Vec<(PublicKey, WeightUnit)> {
        &self.pk_map
    }

    pub fn validate(&self) -> Result<Self, FastCryptoError> {
        let pk_map = self.pubkeys();
        if self.threshold == 0
            || pk_map.is_empty()
            || pk_map.len() > MAX_SIGNER_IN_MULTISIG
            || pk_map.iter().any(|(_pk, weight)| *weight == 0)
            || pk_map
                .iter()
                .map(|(_pk, weight)| *weight as ThresholdUnit)
                .sum::<ThresholdUnit>()
                < self.threshold
            || pk_map.iter().enumerate().any(|(i, (pk, _weight))| {
                pk_map
                    .iter()
                    .skip(i + 1)
                    .any(|(other_pk, _weight)| *pk == *other_pk)
            })
        {
            return Err(FastCryptoError::InvalidInput);
        }
        Ok(self.to_owned())
    }
}

impl From<&MultiSigPublicKey> for SuiAddress {
    /// Derive a SuiAddress from [struct MultiSigPublicKey]. A MultiSig address
    /// is defined as the 32-byte Blake2b hash of serializing the flag, the
    /// threshold, concatenation of all n flag, public keys and
    /// its weight. `flag_MultiSig || threshold || flag_1 || pk_1 || weight_1
    /// || ... || flag_n || pk_n || weight_n`.
    ///
    /// When flag_i is ZkLogin, pk_i refers to [struct ZkLoginPublicIdentifier]
    /// derived from padded address seed in bytes and iss.
    fn from(multisig_pk: &MultiSigPublicKey) -> Self {
        let mut hasher = DefaultHash::default();
        hasher.update([SignatureScheme::MultiSig.flag()]);
        hasher.update(multisig_pk.threshold().to_le_bytes());
        multisig_pk.pubkeys().iter().for_each(|(pk, w)| {
            hasher.update([pk.flag()]);
            hasher.update(pk.as_ref());
            hasher.update(w.to_le_bytes());
        });
        Self::new(hasher.finalize().digest)
    }
}
