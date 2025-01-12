#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! Light-weight, read-only version of Sui's file-based keystore.
//!
//! This crate uses [`eyre`] and is meant for applications, not libraries.
//!
//! <div class="warning">
//!
//! The medium-term plan is to deprecate most of this in favor of [`sui_crypto`].
//!
//! </div>
//!
//! [`sui_crypto`]: https://docs.rs/sui-crypto/latest/sui_crypto/
use af_sui_types::{
    Address as SuiAddress,
    GasData,
    TransactionData,
    TransactionDataV1,
    UserSignature,
};
use crypto::Signature;
use eyre::{Context as _, Result};
pub use intent::Intent;
pub use keystore::{FileBasedKeystore, ReadOnlyAccountKeystore};
use multisig::{MultiSig, MultiSigSigner, ThresholdUnit};

pub mod crypto;
pub mod intent;
pub mod keystore;
pub mod multisig;

/// Computes the required signatures for a transaction's data.
///
/// [`TransactionData`] has a sender and a sponsor (which may be equal to sender), which are
/// [`SuiAddress`]es. This function then gets that information and knows who it has to sign for.
///
/// For simple cases, it just uses those `SuiAddress`es and signs for them using the `keystore`.
///
/// However, there's no way to know if `SuiAddress` corresponds to a multisig. So the function has
/// two optional arguments: `multisig_sender` and `multisig_sponsor`. They exist so that the caller
/// can tell the function if the sender and/or sponsor are multisigs. Their value encodes all the
/// public keys that compose the multisig, their weights, and the threshold (public information).
///
/// The function can then sign for the simple addresses that compose the multisig (assuming
/// `Keystore` has the private keys for each) and combine the simple signatures into a generic
/// signature.
///
/// The [`MultiSigSigner`] message declares what public keys the `Keystore` has to sign for. It's
/// not required to sign for all of them, only a subset that has enough weight.
pub fn signatures<K: ReadOnlyAccountKeystore>(
    tx_data: &TransactionData,
    multisig_sender: Option<MultiSigSigner>,
    multisig_sponsor: Option<MultiSigSigner>,
    keystore: &K,
) -> Result<Vec<UserSignature>> {
    let TransactionData::V1(TransactionDataV1 {
        sender,
        gas_data: GasData { owner: sponsor, .. },
        ..
    }) = tx_data;

    let sender_signature = sign_for_address(tx_data, sender, multisig_sender, keystore)
        .context("Signing for sender")?;
    let mut signatures = vec![sender_signature];

    if sender != sponsor {
        signatures.push(
            sign_for_address(tx_data, sponsor, multisig_sponsor, keystore)
                .context("Signing for sponsor")?,
        );
    };
    Ok(signatures)
}

pub fn sign_for_address<K: ReadOnlyAccountKeystore>(
    tx_data: &TransactionData,
    address: &SuiAddress,
    multisig_signer: Option<MultiSigSigner>,
    keystore: &K,
) -> Result<UserSignature> {
    let signature = if let Some(multisig) = multisig_signer {
        let msig_address = SuiAddress::from(&multisig.multisig_pk);
        eyre::ensure!(
            msig_address == *address,
            "Multisig address {msig_address} doesn't match target address {address}"
        );
        sign_transaction_multisig(tx_data, multisig, keystore)?.into()
    } else {
        sign_transaction(tx_data, address, keystore)?.into()
    };
    Ok(signature)
}

pub fn sign_transaction<K: ReadOnlyAccountKeystore>(
    tx_data: &TransactionData,
    signer: &SuiAddress,
    keystore: &K,
) -> Result<Signature> {
    let signature = keystore.sign_hashed(signer, &tx_data.signing_digest())?;
    Ok(signature)
}

pub fn sign_transaction_multisig<K: ReadOnlyAccountKeystore>(
    tx_data: &TransactionData,
    MultiSigSigner {
        multisig_pk,
        signers,
    }: MultiSigSigner,
    keystore: &K,
) -> Result<MultiSig> {
    let mut total_weight = 0;
    let mut signatures = vec![];

    let hashed = tx_data.signing_digest();
    for idx in signers {
        let (pk, weight) = multisig_pk.pubkeys().get(idx).ok_or_else(|| {
            eyre::eyre!("Signer idx {idx} out of bounds for multisig {multisig_pk:?}")
        })?;
        total_weight += *weight as ThresholdUnit;
        signatures.push(keystore.sign_hashed(&pk.to_sui_address(), &hashed)?);
    }

    if total_weight < *multisig_pk.threshold() {
        eyre::bail!("Signers do not have enought weight to sign for multisig");
    }

    Ok(MultiSig::combine(signatures, multisig_pk)?)
}
