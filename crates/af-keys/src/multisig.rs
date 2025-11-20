//! Additional helpers for multisig signing.

use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use sui_sdk_types::{Address, MultisigCommittee, Transaction, UserSignature};

use crate::Keystore;

/// Data needed for signing as a multisig.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MultisigIntent {
    pub committee: MultisigCommittee,
    /// The indexes of the public keys in `committee` to sign for.
    pub signers: Vec<usize>,
}

/// Sign the transaction data with the private key(s) for an address.
///
/// If the address is a native Sui multisig, `multisig_intent` should be specified to tell the
/// function which constituents to sign for.
pub fn sign_for_address(
    transaction: &Transaction,
    address: Address,
    multisig_intent: Option<MultisigIntent>,
    keystore: &Keystore,
) -> Result<UserSignature, Error> {
    let signature = if let Some(intent) = multisig_intent {
        let msig_address = intent.committee.derive_address();
        anyhow::ensure!(
            msig_address == address,
            "multisig address {msig_address} doesn't match target address {address}"
        );
        UserSignature::Multisig(keystore.multisign_tx(
            transaction,
            intent.committee,
            &intent.signers,
        )?)
    } else {
        UserSignature::Simple(keystore.sign_tx(transaction, address)?)
    };
    Ok(signature)
}

/// Computes the required signatures for a transaction's data.
///
/// [`Transaction`] has a sender and a sponsor (which may be equal to sender), which are
/// [`Address`]es. This function then gets that information and knows who it has to sign for.
///
/// For simple cases, it just uses those `Address`es and signs for them using the [`Keystore`].
///
/// However, there's no way to know if `Address` corresponds to a multisig. So the function has
/// two optional arguments: `multisig_sender` and `multisig_sponsor`. They exist so that the caller
/// can tell the function if the sender and/or sponsor are multisigs. Their value encodes all the
/// public keys that compose the multisig, their weights, and the threshold (public information).
///
/// The function can then sign for the simple addresses that compose the multisig (assuming
/// [`Keystore`] has the private keys for each) and combine the simple signatures into a generic
/// signature.
///
/// The [`MultisigIntent`] message declares what public keys the [`Keystore`] has to sign for. It's
/// not required to sign for all of them, only a subset that has enough weight.
pub fn signatures(
    transaction: &Transaction,
    multisig_sender: Option<MultisigIntent>,
    multisig_sponsor: Option<MultisigIntent>,
    keystore: &Keystore,
) -> Result<Vec<UserSignature>, Error> {
    let sender_signature =
        sign_for_address(transaction, transaction.sender, multisig_sender, keystore)
            .context("Signing for sender")?;
    let mut signatures = vec![sender_signature];
    if transaction.sender == transaction.gas_payment.owner {
        if multisig_sponsor.is_some() {
            log::warn!("Ignoring multisig_sponsor since sender owns the gas inputs");
        };
    } else {
        signatures.push(
            sign_for_address(
                transaction,
                transaction.gas_payment.owner,
                multisig_sponsor,
                keystore,
            )
            .context("Signing for sponsor")?,
        );
    };
    Ok(signatures)
}
