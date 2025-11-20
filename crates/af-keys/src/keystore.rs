use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{Context, Error, bail};
use serde::{Deserialize, Serialize};
use sui_crypto::Signer as _;
use sui_crypto::ed25519::Ed25519PrivateKey;
use sui_crypto::secp256k1::Secp256k1PrivateKey;
use sui_crypto::secp256r1::Secp256r1PrivateKey;
use sui_crypto::simple::SimpleKeypair;
use sui_sdk_types::bcs::FromBcs;
use sui_sdk_types::{
    Address,
    MultisigAggregatedSignature,
    MultisigCommittee,
    MultisigMemberPublicKey,
    MultisigMemberSignature,
    SignatureScheme,
    SimpleSignature,
    Transaction,
};

use crate::PublicKey;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Alias {
    pub alias: String,
    pub public_key_base64: String,
}

#[derive(Debug)]
pub struct Keystore {
    path: PathBuf,
    keys: BTreeMap<Address, SimpleKeypair>,
    aliases: BTreeMap<Address, Alias>,
}

impl Keystore {
    /// Loads a keystore from the default path: `$HOME/.sui/sui_config/sui.keystore`.
    pub fn new_default() -> Result<Self, Error> {
        let keystore_path = match std::env::home_dir() {
            Some(v) => v.join(".sui/sui_config/sui.keystore"),
            None => bail!("cannot obtain home directory path"),
        };
        Self::new(keystore_path)
    }
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let keys = if path.exists() {
            let path_display = path.display();
            let f = File::open(&path)
                .with_context(|| format!("unable to open the keystore file \"{path_display}\""))?;
            let reader = BufReader::new(f);
            let kp_strings: Vec<String> = serde_json::from_reader(reader).with_context(|| {
                format!("unable to deserialize the keystore file \"{path_display}\"")
            })?;
            kp_strings
                .iter()
                .map(|kpstr| {
                    let key = keypair_from_base64(kpstr)?;
                    let address = PublicKey::from(key.public_key()).address()?;
                    Ok((address, key))
                })
                .collect::<Result<BTreeMap<_, _>, Error>>()
                .with_context(|| format!("invalid keystore file \"{path_display}\""))?
        } else {
            BTreeMap::new()
        };

        let mut aliases_path = path.clone();
        aliases_path.set_extension("aliases");
        let aliases = if aliases_path.exists() {
            let path_display = aliases_path.display();
            let reader = BufReader::new(
                File::open(&aliases_path)
                    .with_context(|| format!("unable to open aliases file \"{path_display}\""))?,
            );

            let aliases: Vec<Alias> = serde_json::from_reader(reader).with_context(|| {
                format!("unable to deserialize aliases file \"{path_display}\"")
            })?;

            aliases
                .into_iter()
                .map(|alias| {
                    let key = PublicKey::from_base64(&alias.public_key_base64)?;
                    let address = key.address()?;
                    Ok((address, alias))
                })
                .collect::<Result<BTreeMap<_, _>, Error>>()
                .with_context(|| format!("invalid aliases file \"{path_display}\""))?
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            path,
            keys,
            aliases,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub const fn aliases(&self) -> &BTreeMap<Address, Alias> {
        &self.aliases
    }
    pub fn get_public_key(&self, address: Address) -> Option<PublicKey> {
        self.keys
            .get(&address)
            .map(|keypair| keypair.public_key().into())
    }
    pub fn sign_message(&self, message: &[u8], signer: Address) -> Result<SimpleSignature, Error> {
        let Some(key_pair) = self.keys.get(&signer) else {
            bail!("unable to find keypair for signer address {signer}")
        };
        Ok(key_pair.try_sign(message)?)
    }
    /// Sign the `transaction` for a simple address. Fails if the keystore lacks the private
    /// key for it.
    pub fn sign_tx(
        &self,
        transaction: &Transaction,
        signer: Address,
    ) -> Result<SimpleSignature, Error> {
        let message = transaction.signing_digest();
        self.sign_message(&message, signer)
    }
    /// Sign the `transaction` for a native Sui multisig address. Fails if the keystore lacks the
    /// private keys for the signers with the given `indices`.
    pub fn multisign_tx(
        &self,
        transaction: &Transaction,
        committee: MultisigCommittee,
        indices: &[usize],
    ) -> Result<MultisigAggregatedSignature, Error> {
        let message = transaction.signing_digest();
        let mut total_weight = 0;
        let mut signatures = vec![];
        let mut bitmap = 0;

        for index in indices.iter().copied() {
            let Some(member) = committee.members().get(index) else {
                bail!("signer index {index} out of bounds for multisig {committee:?}");
            };
            total_weight += member.weight() as u16;
            let address = match member.public_key() {
                MultisigMemberPublicKey::Ed25519(public_key) => public_key.derive_address(),
                MultisigMemberPublicKey::Secp256k1(public_key) => public_key.derive_address(),
                MultisigMemberPublicKey::Secp256r1(public_key) => public_key.derive_address(),
                _ => bail!("unsupported public key scheme for multisig member {member:?}"),
            };
            signatures.push(self.sign_message(&message, address)?);
            bitmap |= 1 << index;
        }

        if total_weight < committee.threshold() {
            bail!("signers do not have enough weight to sign for multisig");
        }

        let signatures = signatures
            .into_iter()
            .map(member_signature_from_simple)
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(MultisigAggregatedSignature::new(
            committee, signatures, bitmap,
        ))
    }
}

pub fn member_signature_from_simple(
    signature: SimpleSignature,
) -> Result<MultisigMemberSignature, Error> {
    match signature {
        SimpleSignature::Ed25519 { signature, .. } => {
            Ok(MultisigMemberSignature::Ed25519(signature))
        }
        SimpleSignature::Secp256k1 { signature, .. } => {
            Ok(MultisigMemberSignature::Secp256k1(signature))
        }
        SimpleSignature::Secp256r1 { signature, .. } => {
            Ok(MultisigMemberSignature::Secp256r1(signature))
        }
        _ => bail!(
            "unsupported signature scheme for multisig: {}",
            signature.scheme().name(),
        ),
    }
}

#[derive(Deserialize)]
struct KeyBytes {
    flag: u8,
    key: [u8; 32],
}

pub fn keypair_from_base64(base64: &str) -> Result<SimpleKeypair, Error> {
    let KeyBytes { flag, key } = KeyBytes::from_bcs_base64(base64)?;
    let scheme = SignatureScheme::from_byte(flag).map_err(|err| anyhow::anyhow!(err))?;
    Ok(match scheme {
        SignatureScheme::Ed25519 => SimpleKeypair::from(Ed25519PrivateKey::new(key)),
        SignatureScheme::Secp256k1 => SimpleKeypair::from(Secp256k1PrivateKey::new(key)?),
        SignatureScheme::Secp256r1 => SimpleKeypair::from(Secp256r1PrivateKey::new(key)),
        _ => {
            bail!(
                "unsupported signature scheme {} for a base64-encoded private key",
                scheme.name(),
            );
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTING_PRIVATE_KEYS: [&str; 13] = [
        "AI1TKQ0qPLor32rdLOZiN0/J4qNPyypesT1eE+R/wSCB",
        "AFHMjegm2IwuiLemXb6o7XvuDL7xn1JTHc66CZefYY+B",
        "APhbsR3gpjBIRvZm5ZwMZhncejgYH/hGa6wHVtaTat22",
        "ADO8QyYe0MM+HP0iLjHNLPAxZXNYyE1jieny3iN+fDCS",
        "AKfLSiyx3pUSEpvn0tyY+17ef8AjN7izfQ9qm048BhqM",
        "AOzplQlAK2Uznvog7xmcMtlFC+DfuJx3axo9lfyI876G",
        "AI1I9i3mk2e1kAjPnB7fKiqquxc1OjjAkkpQPIk9Id5Q",
        "AIUAgL5jYMzf0JPCmc263Ou6tH5Z/HuAdtWFFUiz8Zc0",
        "AFmgBTlVGHfYieuSVmQ63BJ+zQSY8pNOUXH99Ucb1ZGl",
        "AAu4ySMvq2wygxl/Ze6AGgkYfxg+rzUElj7UxxI6NHBI",
        "Aoa82Y+xoAzdBLBehaon2kdDst6DNlSOhu+0E43iIfpL",
        "AHAlBn/RWkr6ATvorp6pABpBxy2mRBUNV9RmcU5naeFr",
        "ARn1JTV9CB6x++N/3+BucJFw58vE7p16i1Exd6MOhwnT",
    ];

    const TESTING_PUBLIC_KEYS: [&str; 13] = [
        "ACRAZZ+qMcBA7gJg6iacBSgB4S+DB3nHjk9E1237R4+h",
        "AONa32KBWXqsu6pksuwCLbA0v3JoSPbw8du45Rkw14nm",
        "AKsTkJa8fJg2PJtUTUxIE+FHBBG6IFkHk4385yehR86L",
        "AEIcS8FhN0CjRUGjVHNmXOW6Rb+ootVN3a4kEbBoQ4R6",
        "AP0TE5MM1h7QSZrnlBcdQepKA/6Fh5pja3gjMNpL1fix",
        "AK9WofTFdyBcMpMxzYkbgNQiKLgr9qH8iz9ON6VFxwiW",
        "ALieneYHseSZILiNAda3z29Ob4lZKBAr3jEyP41WsJAG",
        "ABm2kTdq/96JsbsTMunKZDqJbIsEa1lwIJ0cA2CJ4z5l",
        "ADSxYutFskDwLNnEto/E+KDJe4QXWHkO7d8Ha6nqBR0/",
        "ALmzETq2T6c06a+VXJzx1pkfuLBVetRs5q537l6UO4KI",
        "AgJFm9OwmeDknCkUElQlg0e0fJmZg/McSUm6UJH37r61uQ==",
        "ABAiMvjSzayOOYjqNhi2vSgc0qasEQbdJI8ponQ6scXI",
        "AQIu5EC7mUXcgF3oVvqIuCzbp562mUtBqQ/sG+tUqo5KVQ==",
    ];

    const TESTING_ADDRESSES: [&str; 13] = [
        "0x02ef3105413b0bd2ea2f1eee19df48ef4b873694e75b36eaa81c1d1e7d9cf13c",
        "0x2d78d396d59080e2ee66d73cb09ce28b70708b0672c390bcb68cff529e298964",
        "0x43bb1276973beb02c31854145b5c726715c27d8cd49c99534b504ef49951b5fa",
        "0x8cecbc32959d9f610c19b96fe134aa53aa6ba608afaac4e081cd71b30de3459a",
        "0x93d43128794ae9ace7aa8f456ab42281b322c48c0785589ef729bfc3fbd0cda5",
        "0x98e9cafb116af9d69f77ce0d644c60e384f850f8af050b268377d8293d7fe7c6",
        "0xb7d13dae9aec267ae30bbb0811247c032647bf07d4025e97a576dd5a055a713e",
        "0xc4cc77d7de4418d1b84c04e1061f43b74ff2b1e39a85551a3a72fcfe5b8198b5",
        "0xe3a9692f8423d893f87201445a07e24d7d29f997d7ecf8ae880bd635c9845ed4",
        "0xe94c6ed879599794a241d748d714e130da5401489be5d44868377e8c66b620e2",
        "0x0e1205897f909f80a4c6f199abf201cec0f1198ffe4d3c99944be0c7ecb2e2f0",
        "0x3fa7feadd12495a52edc6228cdc1447a8930824dd0fc44eca5909263ec4aa211",
        "0xe0d6507e453e43b6e71400083ef56b658c04de4ab49344d9ceb16f8275845231",
    ];

    #[test]
    fn test_keypair_from_base64_and_address_from_public_key() -> Result<(), Error> {
        let iter = std::iter::zip(
            std::iter::zip(TESTING_PRIVATE_KEYS, TESTING_PUBLIC_KEYS),
            TESTING_ADDRESSES,
        );
        for ((encoded_private, encoded_public), encoded_address) in iter {
            let keypair = keypair_from_base64(encoded_private)?;
            let public_key = PublicKey::from_base64(encoded_public)?;
            assert_eq!(PublicKey::from(keypair.public_key()), public_key);
            let address = Address::from_hex(encoded_address)?;
            assert_eq!(public_key.address()?, address);
        }
        Ok(())
    }
}
