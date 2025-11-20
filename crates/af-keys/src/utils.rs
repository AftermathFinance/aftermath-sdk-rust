use anyhow::{Error, bail};
use sui_sdk_types::{Address, UserSignature};

use crate::PublicKey;

/// Checks if the signer of the signature matches the provided address.
///
/// Checks both padded and unpadded addresses for ZkLogin signature.
/// Fails for Multisig signature as it has combined address only.
pub fn is_signature_signer(signature: &UserSignature, signer: Address) -> Result<bool, Error> {
    Ok(match &signature {
        UserSignature::Multisig(_) => {
            bail!("failed to verify multisig signature signer as it has only combined address")
        }
        UserSignature::ZkLogin(signature) => signature
            .inputs
            .public_identifier()
            .derive_address()
            .any(|address| address == signer),
        _ => {
            let public_key = PublicKey::try_from(signature)?;
            public_key.address()? == signer
        }
    })
}
