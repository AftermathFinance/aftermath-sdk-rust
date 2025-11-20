use serde::{Deserialize, Serialize};
use sui_sdk_types::bcs::{FromBcs, ToBcs};
use sui_sdk_types::{Address, MultisigMemberPublicKey, SimpleSignature, UserSignature};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicKey(MultisigMemberPublicKey);

impl From<MultisigMemberPublicKey> for PublicKey {
    fn from(pk: MultisigMemberPublicKey) -> Self {
        Self(pk)
    }
}

impl TryFrom<&UserSignature> for PublicKey {
    type Error = anyhow::Error;
    fn try_from(signature: &UserSignature) -> Result<Self, anyhow::Error> {
        Ok(match signature {
            UserSignature::Simple(SimpleSignature::Ed25519 { public_key, .. }) => {
                Self(MultisigMemberPublicKey::Ed25519(*public_key))
            }
            UserSignature::Simple(SimpleSignature::Secp256k1 { public_key, .. }) => {
                Self(MultisigMemberPublicKey::Secp256k1(*public_key))
            }
            UserSignature::Simple(SimpleSignature::Secp256r1 { public_key, .. }) => {
                Self(MultisigMemberPublicKey::Secp256r1(*public_key))
            }
            UserSignature::ZkLogin(zk_login_authenticator) => {
                Self(MultisigMemberPublicKey::ZkLogin(
                    zk_login_authenticator.inputs.public_identifier().clone(),
                ))
            }
            UserSignature::Passkey(passkey_authenticator) => Self(
                MultisigMemberPublicKey::Passkey(passkey_authenticator.public_key()),
            ),
            s => anyhow::bail!(
                "unable to extract public key from signature with scheme {}",
                s.scheme().name(),
            ),
        })
    }
}

impl PublicKey {
    pub fn from_base64(base64: &str) -> Result<Self, anyhow::Error> {
        let pk = MultisigMemberPublicKey::from_bcs_base64(base64)?;
        Ok(Self(pk))
    }
    pub fn to_base64(&self) -> Result<String, anyhow::Error> {
        Ok(self
            .0
            .to_bcs_base64()
            .expect("serializing PublicKey to BCS should not fail"))
    }
    pub fn address(&self) -> Result<Address, anyhow::Error> {
        Ok(match self.0 {
            MultisigMemberPublicKey::Ed25519(public_key) => public_key.derive_address(),
            MultisigMemberPublicKey::Secp256k1(public_key) => public_key.derive_address(),
            MultisigMemberPublicKey::Secp256r1(public_key) => public_key.derive_address(),
            MultisigMemberPublicKey::Passkey(public_key) => public_key.derive_address(),
            _ => anyhow::bail!("unable to extract single address from public key {self:?}"),
        })
    }
}
