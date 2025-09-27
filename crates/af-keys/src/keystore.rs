// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Write};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use af_sui_types::Address as SuiAddress;
use enum_dispatch::enum_dispatch;
use eyre::{Context as _, bail, eyre};
use fastcrypto::traits::EncodeDecodeBase64;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::crypto::{PublicKey, Signature, SuiKeyPair};
use crate::intent::{Intent, IntentMessage};

pub type Error = eyre::Report;

#[derive(Serialize, Deserialize)]
#[enum_dispatch(ReadOnlyAccountKeystore)]
pub enum Keystore {
    File(FileBasedKeystore),
    InMem(InMemKeystore),
}

impl Display for Keystore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();
        match self {
            Self::File(file) => {
                writeln!(writer, "Keystore Type : File")?;
                write!(writer, "Keystore Path : {:?}", file.path)?;
                write!(f, "{}", writer)
            }
            Self::InMem(_) => {
                writeln!(writer, "Keystore Type : InMem")?;
                write!(f, "{}", writer)
            }
        }
    }
}

/// Read-only version of [AccountKeystore].
///
/// Allows light-weight applications to use the same keystore file as the Sui CLI.
///
/// [AccountKeystore]: https://mystenlabs.github.io/sui/sui_keys/keystore/trait.AccountKeystore.html
#[enum_dispatch]
pub trait ReadOnlyAccountKeystore: Send + Sync {
    fn keys(&self) -> Vec<PublicKey>;

    fn get_key(&self, address: &SuiAddress) -> Result<&SuiKeyPair, Error>;

    fn sign_hashed(&self, address: &SuiAddress, msg: &[u8]) -> Result<Signature, signature::Error>;

    fn sign_secure<T>(
        &self,
        address: &SuiAddress,
        msg: &T,
        intent: Intent,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize;

    fn addresses(&self) -> Vec<SuiAddress> {
        self.keys().iter().map(|k| k.to_sui_address()).collect()
    }

    fn addresses_with_alias(&self) -> Vec<(&SuiAddress, &Alias)>;

    fn aliases(&self) -> Vec<&Alias>;

    fn alias_names(&self) -> Vec<&str> {
        self.aliases()
            .into_iter()
            .map(|a| a.alias.as_str())
            .collect()
    }

    /// Get alias of address
    fn get_alias_by_address(&self, address: &SuiAddress) -> Result<String, Error>;

    fn get_address_by_alias(&self, alias: String) -> Result<&SuiAddress, Error>;

    /// Check if an alias exists by its name
    fn alias_exists(&self, alias: &str) -> bool {
        self.alias_names().contains(&alias)
    }
}

#[derive(Default)]
pub struct FileBasedKeystore {
    keys: BTreeMap<SuiAddress, SuiKeyPair>,
    aliases: BTreeMap<SuiAddress, Alias>,
    path: Option<PathBuf>,
}

impl FileBasedKeystore {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let keys = if path.exists() {
            let reader =
                BufReader::new(File::open(&path).with_context(|| {
                    format!("Cannot open the keystore file: {}", path.display())
                })?);
            let kp_strings: Vec<String> = serde_json::from_reader(reader).with_context(|| {
                format!("Cannot deserialize the keystore file: {}", path.display(),)
            })?;
            kp_strings
                .iter()
                .map(|kpstr| {
                    let key = SuiKeyPair::decode_base64(kpstr);
                    key.map(|k| (k.public().to_sui_address(), k))
                })
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map_err(|e| eyre!("Invalid keystore file: {}. {}", path.display(), e))?
        } else {
            BTreeMap::new()
        };

        // check aliases
        let mut aliases_path = path.clone();
        aliases_path.set_extension("aliases");

        let aliases = if aliases_path.exists() {
            let reader = BufReader::new(File::open(&aliases_path).with_context(|| {
                format!(
                    "Cannot open aliases file in keystore: {}",
                    aliases_path.display()
                )
            })?);

            let aliases: Vec<Alias> = serde_json::from_reader(reader).with_context(|| {
                format!(
                    "Cannot deserialize aliases file in keystore: {}",
                    aliases_path.display(),
                )
            })?;

            aliases
                .into_iter()
                .map(|alias| {
                    let key = PublicKey::decode_base64(&alias.public_key_base64);
                    key.map(|k| (k.to_sui_address(), alias))
                })
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map_err(|e| {
                    eyre!(
                        "Invalid aliases file in keystore: {}. {}",
                        aliases_path.display(),
                        e
                    )
                })?
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            keys,
            aliases,
            path: Some(path),
        })
    }

    pub fn key_pairs(&self) -> Vec<&SuiKeyPair> {
        self.keys.values().collect()
    }
}

impl Serialize for FileBasedKeystore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let path_default = PathBuf::default();
        serializer.serialize_str(
            self.path
                .as_ref()
                .unwrap_or(&path_default)
                .to_str()
                .unwrap_or(""),
        )
    }
}

impl<'de> Deserialize<'de> for FileBasedKeystore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        Self::new(PathBuf::from(String::deserialize(deserializer)?)).map_err(D::Error::custom)
    }
}

impl ReadOnlyAccountKeystore for FileBasedKeystore {
    fn sign_hashed(&self, address: &SuiAddress, msg: &[u8]) -> Result<Signature, signature::Error> {
        Ok(Signature::new_hashed(
            msg,
            self.keys.get(address).ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?,
        ))
    }
    fn sign_secure<T>(
        &self,
        address: &SuiAddress,
        msg: &T,
        intent: Intent,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        Ok(Signature::new_secure(
            &IntentMessage::new(intent, msg),
            self.keys.get(address).ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?,
        ))
    }

    /// Return an array of `Alias`, consisting of every alias and its corresponding public key.
    fn aliases(&self) -> Vec<&Alias> {
        self.aliases.values().collect()
    }

    fn addresses_with_alias(&self) -> Vec<(&SuiAddress, &Alias)> {
        self.aliases.iter().collect::<Vec<_>>()
    }

    fn keys(&self) -> Vec<PublicKey> {
        self.keys.values().map(|key| key.public()).collect()
    }

    /// Get the address by its alias
    fn get_address_by_alias(&self, alias: String) -> Result<&SuiAddress, Error> {
        self.addresses_with_alias()
            .iter()
            .find(|x| x.1.alias == alias)
            .ok_or_else(|| eyre!("Cannot resolve alias {alias} to an address"))
            .map(|x| x.0)
    }

    /// Get the alias if it exists, or return an error if it does not exist.
    fn get_alias_by_address(&self, address: &SuiAddress) -> Result<String, Error> {
        match self.aliases.get(address) {
            Some(alias) => Ok(alias.alias.clone()),
            None => bail!("Cannot find alias for address {address}"),
        }
    }

    fn get_key(&self, address: &SuiAddress) -> Result<&SuiKeyPair, Error> {
        #[allow(clippy::option_if_let_else)]
        match self.keys.get(address) {
            Some(key) => Ok(key),
            None => Err(eyre!("Cannot find key for address: [{address}]")),
        }
    }
}

/// Carry-over from the original code, but un-initializable.
#[derive(Default, Serialize, Deserialize)]
pub struct InMemKeystore {
    aliases: BTreeMap<SuiAddress, Alias>,
    keys: BTreeMap<SuiAddress, SuiKeyPair>,
}

impl ReadOnlyAccountKeystore for InMemKeystore {
    fn sign_hashed(&self, address: &SuiAddress, msg: &[u8]) -> Result<Signature, signature::Error> {
        Ok(Signature::new_hashed(
            msg,
            self.keys.get(address).ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?,
        ))
    }
    fn sign_secure<T>(
        &self,
        address: &SuiAddress,
        msg: &T,
        intent: Intent,
    ) -> Result<Signature, signature::Error>
    where
        T: Serialize,
    {
        Ok(Signature::new_secure(
            &IntentMessage::new(intent, msg),
            self.keys.get(address).ok_or_else(|| {
                signature::Error::from_source(format!("Cannot find key for address: [{address}]"))
            })?,
        ))
    }

    /// Get all aliases objects
    fn aliases(&self) -> Vec<&Alias> {
        self.aliases.values().collect()
    }

    fn addresses_with_alias(&self) -> Vec<(&SuiAddress, &Alias)> {
        self.aliases.iter().collect::<Vec<_>>()
    }

    fn keys(&self) -> Vec<PublicKey> {
        self.keys.values().map(|key| key.public()).collect()
    }

    fn get_key(&self, address: &SuiAddress) -> Result<&SuiKeyPair, Error> {
        #[allow(clippy::option_if_let_else)]
        match self.keys.get(address) {
            Some(key) => Ok(key),
            None => Err(eyre!("Cannot find key for address: [{address}]")),
        }
    }

    /// Get alias of address
    fn get_alias_by_address(&self, address: &SuiAddress) -> Result<String, Error> {
        match self.aliases.get(address) {
            Some(alias) => Ok(alias.alias.clone()),
            None => bail!("Cannot find alias for address {address}"),
        }
    }

    /// Get the address by its alias
    fn get_address_by_alias(&self, alias: String) -> Result<&SuiAddress, Error> {
        self.addresses_with_alias()
            .iter()
            .find(|x| x.1.alias == alias)
            .ok_or_else(|| eyre!("Cannot resolve alias {alias} to an address"))
            .map(|x| x.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Alias {
    pub alias: String,
    pub public_key_base64: String,
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn new_file_keystore() -> eyre::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path();
        let mut keystore = File::create(path.join("sui.keystore"))?;
        serde_json::to_writer(
            &keystore,
            &serde_json::json!([
                "AKd4u480uT0eLUNe7vh2zHHYdbpUXY/fwcL13eJQ5/zs",
                "AI1TKQ0qPLor32rdLOZiN0/J4qNPyypesT1eE+R/wSCB",
                "AFHMjegm2IwuiLemXb6o7XvuDL7xn1JTHc66CZefYY+B",
                "APhbsR3gpjBIRvZm5ZwMZhncejgYH/hGa6wHVtaTat22",
                "ADO8QyYe0MM+HP0iLjHNLPAxZXNYyE1jieny3iN+fDCS",
                "AKfLSiyx3pUSEpvn0tyY+17ef8AjN7izfQ9qm048BhqM",
                "AOzplQlAK2Uznvog7xmcMtlFC+DfuJx3axo9lfyI876G",
                "AI1I9i3mk2e1kAjPnB7fKiqquxc1OjjAkkpQPIk9Id5Q",
                "AIUAgL5jYMzf0JPCmc263Ou6tH5Z/HuAdtWFFUiz8Zc0",
                "AFmgBTlVGHfYieuSVmQ63BJ+zQSY8pNOUXH99Ucb1ZGl",
                "AAu4ySMvq2wygxl/Ze6AGgkYfxg+rzUElj7UxxI6NHBI"
            ]),
        )?;
        keystore.flush()?;
        let mut aliases = File::create(path.join("sui.aliases"))?;
        serde_json::to_writer(
            &aliases,
            &serde_json::json!([
              {
                "alias": "grace",
                "public_key_base64": "ABhIIE33kaUT1rr9rNrh0XJNb7AC6EBSdh5Ku4a2B7wU"
              },
              {
                "alias": "heidi",
                "public_key_base64": "ACRAZZ+qMcBA7gJg6iacBSgB4S+DB3nHjk9E1237R4+h"
              },
              {
                "alias": "admin",
                "public_key_base64": "AONa32KBWXqsu6pksuwCLbA0v3JoSPbw8du45Rkw14nm"
              },
              {
                "alias": "ivan",
                "public_key_base64": "AKsTkJa8fJg2PJtUTUxIE+FHBBG6IFkHk4385yehR86L"
              },
              {
                "alias": "judy",
                "public_key_base64": "AEIcS8FhN0CjRUGjVHNmXOW6Rb+ootVN3a4kEbBoQ4R6"
              },
              {
                "alias": "eve",
                "public_key_base64": "AP0TE5MM1h7QSZrnlBcdQepKA/6Fh5pja3gjMNpL1fix"
              },
              {
                "alias": "alice",
                "public_key_base64": "AK9WofTFdyBcMpMxzYkbgNQiKLgr9qH8iz9ON6VFxwiW"
              },
              {
                "alias": "bob",
                "public_key_base64": "ALieneYHseSZILiNAda3z29Ob4lZKBAr3jEyP41WsJAG"
              },
              {
                "alias": "charlie",
                "public_key_base64": "ABm2kTdq/96JsbsTMunKZDqJbIsEa1lwIJ0cA2CJ4z5l"
              },
              {
                "alias": "frank",
                "public_key_base64": "ADSxYutFskDwLNnEto/E+KDJe4QXWHkO7d8Ha6nqBR0/"
              },
              {
                "alias": "dave",
                "public_key_base64": "ALmzETq2T6c06a+VXJzx1pkfuLBVetRs5q537l6UO4KI"
              }
            ]),
        )?;
        aliases.flush()?;
        let keystore = FileBasedKeystore::new(path.join("sui.keystore"))?;
        assert!(!keystore.key_pairs().is_empty());
        assert!(
            keystore
                .get_key(
                    &"0x98e9cafb116af9d69f77ce0d644c60e384f850f8af050b268377d8293d7fe7c6"
                        .parse()?
                )
                .is_ok()
        );
        assert!(keystore.get_address_by_alias("alice".to_owned()).is_ok());
        Ok(())
    }

    #[test]
    fn new_file_keystore_no_aliases() -> eyre::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path();
        let keystore_path = path.join("sui.keystore");
        serde_json::to_writer(
            File::create(keystore_path.clone())?,
            &serde_json::json!([
                "AKd4u480uT0eLUNe7vh2zHHYdbpUXY/fwcL13eJQ5/zs",
                "AI1TKQ0qPLor32rdLOZiN0/J4qNPyypesT1eE+R/wSCB",
                "AFHMjegm2IwuiLemXb6o7XvuDL7xn1JTHc66CZefYY+B",
                "APhbsR3gpjBIRvZm5ZwMZhncejgYH/hGa6wHVtaTat22",
                "ADO8QyYe0MM+HP0iLjHNLPAxZXNYyE1jieny3iN+fDCS",
                "AKfLSiyx3pUSEpvn0tyY+17ef8AjN7izfQ9qm048BhqM",
                "AOzplQlAK2Uznvog7xmcMtlFC+DfuJx3axo9lfyI876G",
                "AI1I9i3mk2e1kAjPnB7fKiqquxc1OjjAkkpQPIk9Id5Q",
                "AIUAgL5jYMzf0JPCmc263Ou6tH5Z/HuAdtWFFUiz8Zc0",
                "AFmgBTlVGHfYieuSVmQ63BJ+zQSY8pNOUXH99Ucb1ZGl",
                "AAu4ySMvq2wygxl/Ze6AGgkYfxg+rzUElj7UxxI6NHBI"
            ]),
        )?;
        let keystore = FileBasedKeystore::new(keystore_path)?;
        assert!(!keystore.key_pairs().is_empty());
        assert!(
            keystore
                .get_key(
                    &"0x98e9cafb116af9d69f77ce0d644c60e384f850f8af050b268377d8293d7fe7c6"
                        .parse()?
                )
                .is_ok()
        );
        assert!(keystore.get_address_by_alias("alice".to_owned()).is_err());
        Ok(())
    }
}
