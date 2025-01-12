use std::hash::Hash;

use af_sui_types::Address as SuiAddress;
use derive_more::{AsMut, AsRef};
use enum_dispatch::enum_dispatch;
use eyre::eyre;
use fastcrypto::ed25519::{
    Ed25519KeyPair,
    Ed25519PublicKey,
    Ed25519PublicKeyAsBytes,
    Ed25519Signature,
    Ed25519SignatureAsBytes,
};
use fastcrypto::encoding::{Base64 as Base64Wrapper, Bech32, Encoding as _};
use fastcrypto::error::FastCryptoError;
use fastcrypto::hash::{Blake2b256, HashFunction as _};
use fastcrypto::secp256k1::{
    Secp256k1KeyPair,
    Secp256k1PublicKey,
    Secp256k1PublicKeyAsBytes,
    Secp256k1Signature,
    Secp256k1SignatureAsBytes,
};
use fastcrypto::secp256r1::{
    Secp256r1KeyPair,
    Secp256r1PublicKey,
    Secp256r1PublicKeyAsBytes,
    Secp256r1Signature,
    Secp256r1SignatureAsBytes,
};
use fastcrypto::traits::{
    Authenticator,
    EncodeDecodeBase64,
    KeyPair as KeypairTraits,
    Signer,
    ToFromBytes,
    VerifyingKey,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::base64::Base64;
use serde_with::{serde_as, Bytes, IfIsHumanReadable};
use strum::EnumString;

use crate::intent::IntentMessage;

pub type DefaultHash = Blake2b256;

pub const SUI_PRIV_KEY_PREFIX: &str = "suiprivkey";

/// Custom cryptography errors.
#[derive(
    Eq,
    PartialEq,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    thiserror::Error,
    Hash,
    strum::AsRefStr,
    strum::IntoStaticStr,
)]
pub enum Error {
    #[error("Signature key generation error: {0}")]
    SignatureKeyGenError(String),
    #[error("Key Conversion Error: {0}")]
    KeyConversionError(String),
    #[error("Invalid Private Key provided")]
    InvalidPrivateKey,
    #[error("Signature is not valid: {}", error)]
    InvalidSignature { error: String },
    #[error("Value was not signed by the correct sender: {}", error)]
    IncorrectSigner { error: String },
    #[error("Use of disabled feature: {:?}", error)]
    UnsupportedFeatureError { error: String },
}

// =============================================================================
//  CompressedSignature
// =============================================================================

/// Unlike [enum Signature], [enum CompressedSignature] does not contain public key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressedSignature {
    Ed25519(Ed25519SignatureAsBytes),
    Secp256k1(Secp256k1SignatureAsBytes),
    Secp256r1(Secp256r1SignatureAsBytes),
    ZkLogin(ZkLoginAuthenticatorAsBytes),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZkLoginAuthenticatorAsBytes(pub Vec<u8>);

impl AsRef<[u8]> for CompressedSignature {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Ed25519(sig) => &sig.0,
            Self::Secp256k1(sig) => &sig.0,
            Self::Secp256r1(sig) => &sig.0,
            Self::ZkLogin(sig) => &sig.0,
        }
    }
}

// =============================================================================
//  PublicKey
// =============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKey {
    Ed25519(Ed25519PublicKeyAsBytes),
    Secp256k1(Secp256k1PublicKeyAsBytes),
    Secp256r1(Secp256r1PublicKeyAsBytes),
    ZkLogin(ZkLoginPublicIdentifier),
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Ed25519(pk) => &pk.0,
            Self::Secp256k1(pk) => &pk.0,
            Self::Secp256r1(pk) => &pk.0,
            Self::ZkLogin(z) => &z.0,
        }
    }
}

impl EncodeDecodeBase64 for PublicKey {
    fn encode_base64(&self) -> String {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&[self.flag()]);
        bytes.extend_from_slice(self.as_ref());
        Base64Wrapper::encode(&bytes[..])
    }

    fn decode_base64(value: &str) -> Result<Self, eyre::Report> {
        let bytes = Base64Wrapper::decode(value).map_err(|e| eyre!("{}", e.to_string()))?;
        match bytes.first() {
            Some(x) => {
                if x == &SignatureScheme::ED25519.flag() {
                    let pk: Ed25519PublicKey = Ed25519PublicKey::from_bytes(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?;
                    Ok(Self::Ed25519((&pk).into()))
                } else if x == &SignatureScheme::Secp256k1.flag() {
                    let pk = Secp256k1PublicKey::from_bytes(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?;
                    Ok(Self::Secp256k1((&pk).into()))
                } else if x == &SignatureScheme::Secp256r1.flag() {
                    let pk = Secp256r1PublicKey::from_bytes(
                        bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                    )?;
                    Ok(Self::Secp256r1((&pk).into()))
                } else {
                    Err(eyre!("Invalid flag byte"))
                }
            }
            _ => Err(eyre!("Invalid bytes")),
        }
    }
}

impl PublicKey {
    /// Derive a [SuiAddress].
    ///
    /// Extension to the original trait since a generic [From] cannot be implemented here for
    /// [SuiAddress].
    pub fn to_sui_address(&self) -> SuiAddress {
        let mut hasher = DefaultHash::default();
        hasher.update([self.flag()]);
        hasher.update(self);
        let g_arr = hasher.finalize();
        SuiAddress::new(g_arr.digest)
    }

    pub const fn flag(&self) -> u8 {
        self.scheme().flag()
    }

    pub fn try_from_bytes(curve: SignatureScheme, key_bytes: &[u8]) -> Result<Self, eyre::Report> {
        match curve {
            SignatureScheme::ED25519 => Ok(Self::Ed25519(
                (&Ed25519PublicKey::from_bytes(key_bytes)?).into(),
            )),
            SignatureScheme::Secp256k1 => Ok(Self::Secp256k1(
                (&Secp256k1PublicKey::from_bytes(key_bytes)?).into(),
            )),
            SignatureScheme::Secp256r1 => Ok(Self::Secp256r1(
                (&Secp256r1PublicKey::from_bytes(key_bytes)?).into(),
            )),
            _ => Err(eyre!("Unsupported curve")),
        }
    }

    pub const fn scheme(&self) -> SignatureScheme {
        match self {
            Self::Ed25519(_) => Ed25519SuiSignature::SCHEME,
            Self::Secp256k1(_) => Secp256k1SuiSignature::SCHEME,
            Self::Secp256r1(_) => Secp256r1SuiSignature::SCHEME,
            Self::ZkLogin(_) => SignatureScheme::ZkLoginAuthenticator,
        }
    }
}
/// A wrapper struct to retrofit in [enum PublicKey] for zkLogin.
/// Useful to construct [struct MultiSigPublicKey].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkLoginPublicIdentifier(pub Vec<u8>);

// =============================================================================
//  Signature
// =============================================================================

// Enums for signature scheme signatures
#[enum_dispatch]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Signature {
    Ed25519SuiSignature,
    Secp256k1SuiSignature,
    Secp256r1SuiSignature,
}

impl Signature {
    /// The messaged passed in is already hashed form.
    pub fn new_hashed(hashed_msg: &[u8], secret: &dyn Signer<Self>) -> Self {
        Signer::sign(secret, hashed_msg)
    }

    pub fn new_secure<T>(value: &IntentMessage<T>, secret: &dyn Signer<Self>) -> Self
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(bcs::to_bytes(&value).expect("Message serialization should not fail"));
        Signer::sign(secret, &hasher.finalize().digest)
    }

    pub fn to_compressed(&self) -> Result<CompressedSignature, Error> {
        let bytes = self.signature_bytes();
        match self.scheme() {
            SignatureScheme::ED25519 => Ok(CompressedSignature::Ed25519(
                (&Ed25519Signature::from_bytes(bytes).map_err(|_| Error::InvalidSignature {
                    error: "Cannot parse ed25519 sig".to_string(),
                })?)
                    .into(),
            )),
            SignatureScheme::Secp256k1 => Ok(CompressedSignature::Secp256k1(
                (&Secp256k1Signature::from_bytes(bytes).map_err(|_| Error::InvalidSignature {
                    error: "Cannot parse secp256k1 sig".to_string(),
                })?)
                    .into(),
            )),
            SignatureScheme::Secp256r1 => Ok(CompressedSignature::Secp256r1(
                (&Secp256r1Signature::from_bytes(bytes).map_err(|_| Error::InvalidSignature {
                    error: "Cannot parse secp256r1 sig".to_string(),
                })?)
                    .into(),
            )),
            _ => Err(Error::UnsupportedFeatureError {
                error: "Unsupported signature scheme".to_string(),
            }),
        }
    }

    pub fn to_public_key(&self) -> Result<PublicKey, Error> {
        let bytes = self.public_key_bytes();
        match self.scheme() {
            SignatureScheme::ED25519 => Ok(PublicKey::Ed25519(
                (&Ed25519PublicKey::from_bytes(bytes).map_err(|_| {
                    Error::KeyConversionError("Cannot parse ed25519 pk".to_string())
                })?)
                    .into(),
            )),
            SignatureScheme::Secp256k1 => Ok(PublicKey::Secp256k1(
                (&Secp256k1PublicKey::from_bytes(bytes).map_err(|_| {
                    Error::KeyConversionError("Cannot parse secp256k1 pk".to_string())
                })?)
                    .into(),
            )),
            SignatureScheme::Secp256r1 => Ok(PublicKey::Secp256r1(
                (&Secp256r1PublicKey::from_bytes(bytes).map_err(|_| {
                    Error::KeyConversionError("Cannot parse secp256r1 pk".to_string())
                })?)
                    .into(),
            )),
            _ => Err(Error::UnsupportedFeatureError {
                error: "Unsupported signature scheme in Signature".to_string(),
            }),
        }
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.as_ref();

        if serializer.is_human_readable() {
            let s = Base64Wrapper::encode(bytes);
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_bytes(bytes)
        }
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let bytes = if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Base64Wrapper::decode(&s).map_err(|e| Error::custom(e.to_string()))?
        } else {
            let data: Vec<u8> = Vec::deserialize(deserializer)?;
            data
        };

        Self::from_bytes(&bytes).map_err(|e| Error::custom(e.to_string()))
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Ed25519SuiSignature(sig) => sig.as_ref(),
            Self::Secp256k1SuiSignature(sig) => sig.as_ref(),
            Self::Secp256r1SuiSignature(sig) => sig.as_ref(),
        }
    }
}
impl AsMut<[u8]> for Signature {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Ed25519SuiSignature(sig) => sig.as_mut(),
            Self::Secp256k1SuiSignature(sig) => sig.as_mut(),
            Self::Secp256r1SuiSignature(sig) => sig.as_mut(),
        }
    }
}

impl ToFromBytes for Signature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        match bytes.first() {
            Some(x) => {
                if x == &Ed25519SuiSignature::SCHEME.flag() {
                    Ok(<Ed25519SuiSignature as ToFromBytes>::from_bytes(bytes)?.into())
                } else if x == &Secp256k1SuiSignature::SCHEME.flag() {
                    Ok(<Secp256k1SuiSignature as ToFromBytes>::from_bytes(bytes)?.into())
                } else if x == &Secp256r1SuiSignature::SCHEME.flag() {
                    Ok(<Secp256r1SuiSignature as ToFromBytes>::from_bytes(bytes)?.into())
                } else {
                    Err(FastCryptoError::InvalidInput)
                }
            }
            _ => Err(FastCryptoError::InvalidInput),
        }
    }
}

impl From<Signature> for af_sui_types::UserSignature {
    fn from(value: Signature) -> Self {
        Self::from_bytes(value.as_bytes()).expect("Compatible")
    }
}

// =============================================================================
//  SignatureScheme
// =============================================================================

#[derive(Clone, Copy, Deserialize, Serialize, Debug, EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SignatureScheme {
    ED25519,
    Secp256k1,
    Secp256r1,
    BLS12381, // This is currently not supported for user Sui Address.
    MultiSig,
    ZkLoginAuthenticator,
}

impl SignatureScheme {
    pub const fn flag(&self) -> u8 {
        match self {
            Self::ED25519 => 0x00,
            Self::Secp256k1 => 0x01,
            Self::Secp256r1 => 0x02,
            Self::MultiSig => 0x03,
            Self::BLS12381 => 0x04, // This is currently not supported for user Sui Address.
            Self::ZkLoginAuthenticator => 0x05,
        }
    }

    pub fn from_flag(flag: &str) -> Result<Self, Error> {
        let byte_int = flag
            .parse::<u8>()
            .map_err(|_| Error::KeyConversionError("Invalid key scheme".to_string()))?;
        Self::from_flag_byte(&byte_int)
    }

    pub fn from_flag_byte(byte_int: &u8) -> Result<Self, Error> {
        match byte_int {
            0x00 => Ok(Self::ED25519),
            0x01 => Ok(Self::Secp256k1),
            0x02 => Ok(Self::Secp256r1),
            0x03 => Ok(Self::MultiSig),
            0x04 => Ok(Self::BLS12381),
            0x05 => Ok(Self::ZkLoginAuthenticator),
            _ => Err(Error::KeyConversionError("Invalid key scheme".to_string())),
        }
    }
}

// =============================================================================
//  SuiKeyPair
// =============================================================================

#[allow(clippy::large_enum_variant)]
#[derive(Debug, derive_more::From, PartialEq, Eq)]
pub enum SuiKeyPair {
    Ed25519(Ed25519KeyPair),
    Secp256k1(Secp256k1KeyPair),
    Secp256r1(Secp256r1KeyPair),
}

impl SuiKeyPair {
    pub fn public(&self) -> PublicKey {
        match self {
            Self::Ed25519(kp) => PublicKey::Ed25519(kp.public().into()),
            Self::Secp256k1(kp) => PublicKey::Secp256k1(kp.public().into()),
            Self::Secp256r1(kp) => PublicKey::Secp256r1(kp.public().into()),
        }
    }
}

impl Signer<Signature> for SuiKeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        match self {
            Self::Ed25519(kp) => kp.sign(msg),
            Self::Secp256k1(kp) => kp.sign(msg),
            Self::Secp256r1(kp) => kp.sign(msg),
        }
    }
}

impl EncodeDecodeBase64 for SuiKeyPair {
    fn encode_base64(&self) -> String {
        Base64Wrapper::encode(self.to_bytes())
    }

    fn decode_base64(value: &str) -> Result<Self, eyre::Report> {
        let bytes = Base64Wrapper::decode(value).map_err(|e| eyre!("{}", e.to_string()))?;
        Self::from_bytes(&bytes)
    }
}
impl SuiKeyPair {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(self.public().flag());

        match self {
            Self::Ed25519(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
            Self::Secp256k1(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
            Self::Secp256r1(kp) => {
                bytes.extend_from_slice(kp.as_bytes());
            }
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, eyre::Report> {
        match SignatureScheme::from_flag_byte(bytes.first().ok_or_else(|| eyre!("Invalid length"))?)
        {
            Ok(x) => match x {
                SignatureScheme::ED25519 => Ok(Self::Ed25519(Ed25519KeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
                SignatureScheme::Secp256k1 => Ok(Self::Secp256k1(Secp256k1KeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
                SignatureScheme::Secp256r1 => Ok(Self::Secp256r1(Secp256r1KeyPair::from_bytes(
                    bytes.get(1..).ok_or_else(|| eyre!("Invalid length"))?,
                )?)),
                _ => Err(eyre!("Invalid flag byte")),
            },
            _ => Err(eyre!("Invalid bytes")),
        }
    }
    /// Encode a SuiKeyPair as `flag || privkey` in Bech32 starting with "suiprivkey" to a string. Note that the pubkey is not encoded.
    pub fn encode(&self) -> Result<String, eyre::Report> {
        Bech32::encode(self.to_bytes(), SUI_PRIV_KEY_PREFIX)
    }

    /// Decode a SuiKeyPair from `flag || privkey` in Bech32 starting with "suiprivkey" to SuiKeyPair. The public key is computed directly from the private key bytes.
    pub fn decode(value: &str) -> Result<Self, eyre::Report> {
        let bytes = Bech32::decode(value, SUI_PRIV_KEY_PREFIX)?;
        Self::from_bytes(&bytes)
    }
}

impl Serialize for SuiKeyPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.encode_base64();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for SuiKeyPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        Self::decode_base64(&s).map_err(|e| Error::custom(e.to_string()))
    }
}

// =============================================================================
//  Ed25519 Sui Signature port
// =============================================================================

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct Ed25519SuiSignature(
    #[serde_as(as = "IfIsHumanReadable<Base64, Bytes>")]
    [u8; Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1],
);

// Implementation useful for simplify testing when mock signature is needed
impl Default for Ed25519SuiSignature {
    fn default() -> Self {
        Self([0; Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1])
    }
}

impl SuiSignatureInner for Ed25519SuiSignature {
    type Sig = Ed25519Signature;
    type PubKey = Ed25519PublicKey;
    type KeyPair = Ed25519KeyPair;
    const LENGTH: usize = Ed25519PublicKey::LENGTH + Ed25519Signature::LENGTH + 1;
}

impl SuiPublicKey for Ed25519PublicKey {
    const SIGNATURE_SCHEME: SignatureScheme = SignatureScheme::ED25519;
}

impl ToFromBytes for Ed25519SuiSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != Self::LENGTH {
            return Err(FastCryptoError::InputLengthWrong(Self::LENGTH));
        }
        let mut sig_bytes = [0; Self::LENGTH];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for Ed25519KeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        Ed25519SuiSignature::new(self, msg).into()
    }
}

// =============================================================================
//  Secp256k1 Sui Signature port
// =============================================================================

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct Secp256k1SuiSignature(
    #[serde_as(as = "IfIsHumanReadable<Base64, Bytes>")]
    [u8; Secp256k1PublicKey::LENGTH + Secp256k1Signature::LENGTH + 1],
);

impl SuiSignatureInner for Secp256k1SuiSignature {
    type Sig = Secp256k1Signature;
    type PubKey = Secp256k1PublicKey;
    type KeyPair = Secp256k1KeyPair;
    const LENGTH: usize = Secp256k1PublicKey::LENGTH + Secp256k1Signature::LENGTH + 1;
}

impl SuiPublicKey for Secp256k1PublicKey {
    const SIGNATURE_SCHEME: SignatureScheme = SignatureScheme::Secp256k1;
}

impl ToFromBytes for Secp256k1SuiSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != Self::LENGTH {
            return Err(FastCryptoError::InputLengthWrong(Self::LENGTH));
        }
        let mut sig_bytes = [0; Self::LENGTH];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for Secp256k1KeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        Secp256k1SuiSignature::new(self, msg).into()
    }
}

// =============================================================================
// Secp256r1 Sui Signature port
// =============================================================================

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, AsRef, AsMut)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct Secp256r1SuiSignature(
    #[serde_as(as = "IfIsHumanReadable<Base64, Bytes>")]
    [u8; Secp256r1PublicKey::LENGTH + Secp256r1Signature::LENGTH + 1],
);

impl SuiSignatureInner for Secp256r1SuiSignature {
    type Sig = Secp256r1Signature;
    type PubKey = Secp256r1PublicKey;
    type KeyPair = Secp256r1KeyPair;
    const LENGTH: usize = Secp256r1PublicKey::LENGTH + Secp256r1Signature::LENGTH + 1;
}

impl SuiPublicKey for Secp256r1PublicKey {
    const SIGNATURE_SCHEME: SignatureScheme = SignatureScheme::Secp256r1;
}

impl ToFromBytes for Secp256r1SuiSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        if bytes.len() != Self::LENGTH {
            return Err(FastCryptoError::InputLengthWrong(Self::LENGTH));
        }
        let mut sig_bytes = [0; Self::LENGTH];
        sig_bytes.copy_from_slice(bytes);
        Ok(Self(sig_bytes))
    }
}

impl Signer<Signature> for Secp256r1KeyPair {
    fn sign(&self, msg: &[u8]) -> Signature {
        Secp256r1SuiSignature::new(self, msg).into()
    }
}

// =============================================================================
//  This struct exists due to the limitations of the `enum_dispatch` library.
// =============================================================================

pub trait SuiSignatureInner: Sized + ToFromBytes + PartialEq + Eq + Hash {
    type Sig: Authenticator<PubKey = Self::PubKey>;
    type PubKey: VerifyingKey<Sig = Self::Sig> + SuiPublicKey;
    type KeyPair: KeypairTraits<PubKey = Self::PubKey, Sig = Self::Sig>;

    const LENGTH: usize = Self::Sig::LENGTH + Self::PubKey::LENGTH + 1;
    const SCHEME: SignatureScheme = Self::PubKey::SIGNATURE_SCHEME;

    /// Returns the deserialized signature and deserialized pubkey.
    fn get_verification_inputs(&self) -> Result<(Self::Sig, Self::PubKey), Error> {
        let pk = Self::PubKey::from_bytes(self.public_key_bytes())
            .map_err(|_| Error::KeyConversionError("Invalid public key".to_string()))?;

        // deserialize the signature
        let signature =
            Self::Sig::from_bytes(self.signature_bytes()).map_err(|_| Error::InvalidSignature {
                error: "Fail to get pubkey and sig".to_string(),
            })?;

        Ok((signature, pk))
    }

    fn new(kp: &Self::KeyPair, message: &[u8]) -> Self {
        let sig = Signer::sign(kp, message);

        let mut signature_bytes: Vec<u8> = Vec::new();
        signature_bytes
            .extend_from_slice(&[<Self::PubKey as SuiPublicKey>::SIGNATURE_SCHEME.flag()]);
        signature_bytes.extend_from_slice(sig.as_ref());
        signature_bytes.extend_from_slice(kp.public().as_ref());
        Self::from_bytes(&signature_bytes[..])
            .expect("Serialized signature did not have expected size")
    }
}

#[enum_dispatch(Signature)]
pub trait SuiSignature: Sized + ToFromBytes {
    fn signature_bytes(&self) -> &[u8];
    fn public_key_bytes(&self) -> &[u8];
    fn scheme(&self) -> SignatureScheme;

    fn verify_secure<T>(
        &self,
        value: &IntentMessage<T>,
        author: SuiAddress,
        scheme: SignatureScheme,
    ) -> Result<(), Error>
    where
        T: Serialize;
}

impl<S: SuiSignatureInner + Sized> SuiSignature for S {
    fn signature_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_ref()[1..1 + S::Sig::LENGTH]
    }

    fn public_key_bytes(&self) -> &[u8] {
        // Access array slice is safe because the array bytes is initialized as
        // flag || signature || pubkey with its defined length.
        &self.as_ref()[S::Sig::LENGTH + 1..]
    }

    fn scheme(&self) -> SignatureScheme {
        S::PubKey::SIGNATURE_SCHEME
    }

    fn verify_secure<T>(
        &self,
        value: &IntentMessage<T>,
        author: SuiAddress,
        scheme: SignatureScheme,
    ) -> Result<(), Error>
    where
        T: Serialize,
    {
        let mut hasher = DefaultHash::default();
        hasher.update(bcs::to_bytes(&value).expect("Message serialization should not fail"));
        let digest = hasher.finalize().digest;

        let (sig, pk) = &self.get_verification_inputs()?;
        match scheme {
            SignatureScheme::ZkLoginAuthenticator => {} // Pass this check because zk login does not derive address from pubkey.
            _ => {
                let address = pk.to_sui_address();
                if author != address {
                    return Err(Error::IncorrectSigner {
                        error: format!(
                            "Incorrect signer, expected {:?}, got {:?}",
                            author, address
                        ),
                    });
                }
            }
        }

        pk.verify(&digest, sig)
            .map_err(|e| Error::InvalidSignature {
                error: format!("Fail to verify user sig {}", e),
            })
    }
}

// =============================================================================
//  SuiPublicKey
// =============================================================================

pub trait SuiPublicKey: VerifyingKey {
    const SIGNATURE_SCHEME: SignatureScheme;

    /// Convert to a [SuiAddress].
    ///
    /// Extension to the original trait since a generic [From] cannot be implemented here for
    /// [SuiAddress].
    fn to_sui_address(&self) -> SuiAddress {
        let mut hasher = DefaultHash::default();
        hasher.update([Self::SIGNATURE_SCHEME.flag()]);
        hasher.update(self);
        let g_arr = hasher.finalize();
        SuiAddress::new(g_arr.digest)
    }
}
