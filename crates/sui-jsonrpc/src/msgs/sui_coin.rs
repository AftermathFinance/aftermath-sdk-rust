// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use af_sui_types::{EpochId, ObjectDigest, ObjectId, ObjectRef, TransactionDigest};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, IfIsHumanReadable};
use sui_sdk_types::Version;

use super::Page;
use crate::serde::BigInt;

pub type CoinPage = Page<Coin, String>;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin_type: String,
    pub coin_object_count: usize,
    #[serde_as(as = "BigInt<u128>")]
    pub total_balance: u128,
    #[serde_as(as = "HashMap<BigInt<u64>, BigInt<u128>>")]
    pub locked_balance: HashMap<EpochId, u128>,
}

impl Balance {
    pub fn zero(coin_type: String) -> Self {
        Self {
            coin_type,
            coin_object_count: 0,
            total_balance: 0,
            locked_balance: HashMap::new(),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Coin {
    pub coin_type: String,
    pub coin_object_id: ObjectId,
    #[serde_as(as = "BigInt<u64>")]
    pub version: Version,
    pub digest: ObjectDigest,
    #[serde_as(as = "BigInt<u64>")]
    pub balance: u64,
    pub previous_transaction: TransactionDigest,
}

impl Coin {
    pub fn object_ref(&self) -> ObjectRef {
        (self.coin_object_id, self.version, self.digest)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoinMetadata {
    /// Number of decimal places the coin uses.
    pub decimals: u8,
    /// Name for the token
    pub name: String,
    /// Symbol for the token
    pub symbol: String,
    /// Description of the token
    pub description: String,
    /// URL for the token logo
    pub icon_url: Option<String>,
    /// Object id for the CoinMetadata object
    pub id: Option<ObjectId>,
}

/// Originally from `sui_types::balance`.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Supply {
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub value: u64,
}

#[cfg(test)]
mod tests {
    use super::CoinPage;

    #[test]
    fn coin_page_json() {
        let value = serde_json::json!({
          "data": [
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x778e75c813cc7bf615a3477b5c2e5cc4364cde735b80c43597ec8175fd943671",
              "version": "364688977",
              "digest": "7yc3F75UbHU9Qshf951zmbkQ7rgQmE8r1YRKGTbmLJ2f",
              "balance": "9998727478293",
              "previousTransaction": "CQHxo6mn7zNQX11guHTUPNa5PMhCvodhmoz982PFnErX"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x15be68e1baa31868d463e0676fa46e1fb6b0f51063d804e857f0113d2920c67a",
              "version": "204442968",
              "digest": "4YBFFvLDMhuQPsLtENqfGsvAcV4mDqzaiZBvhJ7KdjiH",
              "balance": "996650164",
              "previousTransaction": "HsstpwxjrXBJCf5eqSzV7asXnE2e2JVDYQxCNtzEiDHG"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x4aca24029ffb5583adc4e9a298244ced3faf6827412269aa1ea0feb89038f0be",
              "version": "281343209",
              "digest": "5zSTxMKJABvj31nw5ZYrffwU6Adeiu5vcNsP2h55JNFU",
              "balance": "995220764",
              "previousTransaction": "HLY1QY8okXbXmPGKvdpYSgTvZyjgJECpiXjQNQTnBiQj"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0xbe02f79a382915959c457880f12f03b1ec475d3084795fc014c4f43e50e5afe6",
              "version": "169438719",
              "digest": "ApvnC81FCi71JhMnZjBn1y1NN6Lw279j2NiG1ehvtDhM",
              "balance": "993603580",
              "previousTransaction": "F9u1PgGAyaBkKYGs8r7L8zjTJxa6SwamuzpTxEjUyizy"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x89f6504590d43becf58fd949772e609731bffbd72e6ac5521357b12a0b2fc2d2",
              "version": "204443012",
              "digest": "2soVvf43GDoEUvNy2sSYxbmwketGdS6fU65H1Ze4BqJD",
              "balance": "320804480",
              "previousTransaction": "37skCQJmUCn5sPJJShi8sk1EhvKMuxjAuN7pT9nEydvi"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x00c4eaac095d54118eba1ba78c1f7e59b42620e070baeb6b7ff2ee481ccda314",
              "version": "204443010",
              "digest": "7H3SutEWwU5uJSx6AB2vAUCRz9QdrAvfJU2f4DXVUQiX",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x00d3f6aa6e02253f6f8f128af2772be4922d28b38909746bc3aee9528fd02c1f",
              "version": "204443012",
              "digest": "mBS7PgFZ3kEWXdEjf9H87zLXAXusUm33zP4hvf2c6PN",
              "balance": "100000000",
              "previousTransaction": "37skCQJmUCn5sPJJShi8sk1EhvKMuxjAuN7pT9nEydvi"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x01180d0165e367f8cae862e2f8bb31db099de3127cac2e015f482a330ad53f0c",
              "version": "204443011",
              "digest": "9o9fNdXkgQ1Q9obzPXnRykrADrfvXYEVkpHihBh3bkAW",
              "balance": "100000000",
              "previousTransaction": "C8reWy1j85aBxeKRweoeW1amkiWBdLmHpdajzFzAh9vN"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x011bf3b329893627c875da9136af03dd4c80a0b50df80b620317d72f867a57a5",
              "version": "204443011",
              "digest": "HbGQS5rvjrZ7Kb5rGtuqqhyQzDwzQca7ZwTqqXejWgny",
              "balance": "100000000",
              "previousTransaction": "C8reWy1j85aBxeKRweoeW1amkiWBdLmHpdajzFzAh9vN"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x0137cbf95c63ad3f5eebcb3ce00eddb2469c033b84aa652853fa3f8ab1673e91",
              "version": "204443009",
              "digest": "BrrqeYcBhYXXQTNVb93fD5XBMT8qkLKzMjcWnhc6HQxF",
              "balance": "100000000",
              "previousTransaction": "ErMm3QXPzdk6hnWDnm4ikvzGqEoLptGNRbXxf4hetxfZ"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x0142b2b7a38fbbe2be62c367d475960ec59d4a1f270d51d575ff477ac833e331",
              "version": "204443009",
              "digest": "E2JFhJuutsVYp6AfcWnLT6UBydfHc7fQQA6t3AMJ1YEV",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x014a72c032016a3e735b1b3c87ac63b7b56ece83a70074d69ffd334a5b890dbb",
              "version": "204443009",
              "digest": "5nLWKmLfbBZZJ54UFV3FUgWycLj5VSvno7oYsM5xcn5r",
              "balance": "100000000",
              "previousTransaction": "ErMm3QXPzdk6hnWDnm4ikvzGqEoLptGNRbXxf4hetxfZ"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x01ee86bd47d7e193b22f63475b62bb7b4c86dcbd245ad835116d450fc5002d0c",
              "version": "204443009",
              "digest": "59iohfduZsQMUXiNZgY36y5kyaZqTYoEPpRHvJXqCyMA",
              "balance": "100000000",
              "previousTransaction": "ErMm3QXPzdk6hnWDnm4ikvzGqEoLptGNRbXxf4hetxfZ"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x01fb41a94580b9815ab5b64f909b5c76e57648dffa17a46902e854735c7b2664",
              "version": "204443009",
              "digest": "A9f9GxA4ceAhRetrDDg9gQEdt6yBJKr1CAikYHChcbUv",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x020c674a7dfe642fab957ae1781a72e220b7e4e161dac134be3ee21c67397659",
              "version": "204443010",
              "digest": "3jFHQm2pUcZvFdRmBrQiqfPQtUi2oyAbRUHNHXRTiWRp",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x0302b7bfeb5523014f9293d86ad63da8a4e85d7397b58c9488c7279b571608e8",
              "version": "204443009",
              "digest": "C1V1zxgv42UdkUEMxQy3aSeE7CqWpw1RHZaswW377Bco",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x035d124b3577893985e3def4f7822178ffce315b010250148cb4d9fe4eb26fbd",
              "version": "204443012",
              "digest": "C7L7hhpNavzyrEHSFa3vMd84dPomMmA27nT5mQoj7wC7",
              "balance": "100000000",
              "previousTransaction": "37skCQJmUCn5sPJJShi8sk1EhvKMuxjAuN7pT9nEydvi"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x037c3b0c4604963cf7e5b96b9e692045961d1871c8ebbc0eb5e06cb3c5641239",
              "version": "204443011",
              "digest": "6a36ybzpPogNRfcdJ8CSeEwUFBjEpZPBEmZUwXgHxgdQ",
              "balance": "100000000",
              "previousTransaction": "C8reWy1j85aBxeKRweoeW1amkiWBdLmHpdajzFzAh9vN"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x039783a068009f91dcd4ef048626834b9a0fbb9d75ec6c3aefde2e4a68940340",
              "version": "204443012",
              "digest": "31b4YUjmo25h4SHP4hc5JhMRZ2Lpak1AjfNmLQJFxDhc",
              "balance": "100000000",
              "previousTransaction": "37skCQJmUCn5sPJJShi8sk1EhvKMuxjAuN7pT9nEydvi"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x03bb15c8b1efb827c0f05a8ca90c4629bcf7eac577f021d3be26356543664e7d",
              "version": "204443010",
              "digest": "HVWRpHFmmqaPXvPcZ4bcEyCV6hJ5pCgqnZX3ZANfsiPP",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x03c9ab723d12ed8096e5e9bbfa171ae79de975bc1519d054b52e50f93566519c",
              "version": "204443011",
              "digest": "7A17g6F3XSdBPuZ6UPJmepQdJBovmmcS4PETH8pusHDt",
              "balance": "100000000",
              "previousTransaction": "C8reWy1j85aBxeKRweoeW1amkiWBdLmHpdajzFzAh9vN"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x03ce6f9849e55f20699b32df05d2b31bb69affe67c74a1e655d287430d08b86a",
              "version": "204443011",
              "digest": "FWaFBi4tEMEV2ca54KYpBCGd7xoYqNcyzWmjZbrcD5mW",
              "balance": "100000000",
              "previousTransaction": "C8reWy1j85aBxeKRweoeW1amkiWBdLmHpdajzFzAh9vN"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x040363647d1f837eedd65712144ec12b0d08d3f9843fe88260c738dea483c191",
              "version": "204443009",
              "digest": "AyQAViL6gkkg2bUojgP2tNcSxtUe1HhSvvAyqvmYdgvk",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x04078b551cc46943c62e138ee84a5b74c4572afc2a7ceadce2db87362d7d3fbd",
              "version": "204443010",
              "digest": "HtahaMU1PShZU52dnnte9wz6wjym2JrUdQe94H363g3V",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x04223a97a52778a930caeac1095064c9992145bf6c736acd67c215c3b749dc85",
              "version": "204443009",
              "digest": "9Wis1StoPER6VQKF4niT2Et5AWFKrQCC1xF4vBuwTZiY",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x043527fbcd0a98dfcd24a553c16c2e424a6c0eb511cd28c7a0fd0dcc0cae7db5",
              "version": "204443009",
              "digest": "7WMkbHnFRqyu6bh7j9aC87sZT7dv3v5fN7rt9tSx3RFD",
              "balance": "100000000",
              "previousTransaction": "ErMm3QXPzdk6hnWDnm4ikvzGqEoLptGNRbXxf4hetxfZ"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x043d85bc1fd4e22da944c2aadb079c42baa8f412cb2ada9495768b2760000dbf",
              "version": "204443011",
              "digest": "6TfSFvfyAbrP86xB18YY3DSoy2KqHxj2gg8Dg4XWrqfK",
              "balance": "100000000",
              "previousTransaction": "C8reWy1j85aBxeKRweoeW1amkiWBdLmHpdajzFzAh9vN"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x046a3e3af89ac4787770aa5528b1e920812a1ec012aefba05799f1f45cede1d7",
              "version": "204443009",
              "digest": "8DSbVPCKABx7P5cY8LxNWiXyZhcKju5vUbTRRgqh49NL",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x04f907b30aa0fe2168d887c7e8971944a48bee5fb810d6a07ff4c944665d964c",
              "version": "204443012",
              "digest": "Ge4ZBwXFyfMXj1Fe9GurS5RTHwY29a3jo6Fu7AB3QEtX",
              "balance": "100000000",
              "previousTransaction": "37skCQJmUCn5sPJJShi8sk1EhvKMuxjAuN7pT9nEydvi"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x05001d2eb977205bdb6e928a374adaa67049b0f2e42b1c8ca51649fba7ec8c2a",
              "version": "204443009",
              "digest": "8ScwnnePv1A6Gjw2CKs6VDuvDSEtyfWcJyDLCk8a2SiY",
              "balance": "100000000",
              "previousTransaction": "ErMm3QXPzdk6hnWDnm4ikvzGqEoLptGNRbXxf4hetxfZ"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x050ac19b018586da46bbb980d2a60d5cc1e34059a9da7b1a8365bb4129d98b8f",
              "version": "204443009",
              "digest": "GUnP1FZ89C8gMCz864SyqeUnFEAAAUXfcWTZUw8bBrrG",
              "balance": "100000000",
              "previousTransaction": "ErMm3QXPzdk6hnWDnm4ikvzGqEoLptGNRbXxf4hetxfZ"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x051fe3a93a08feaa0321702eda9c17484cc33f139be93b19fba6ec0e386b2e7b",
              "version": "204443009",
              "digest": "43mCkekG6meCs1srwrCnQkSm5zyH9MaUUyHPMD4u2GN4",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x052078a0212f3589d0b7a605377edbdd35c8728c7ec1dfe5ac68020bbd213e74",
              "version": "204443011",
              "digest": "9Av7girBZB9nRMidT485NtbihgALzjnQ2KVk9XGKG3PJ",
              "balance": "100000000",
              "previousTransaction": "C8reWy1j85aBxeKRweoeW1amkiWBdLmHpdajzFzAh9vN"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x05314165d8133f713f95164af7566e4d21a25bca64d0dea55355f66ed5913ee4",
              "version": "204443009",
              "digest": "4hn3chb1uXvYqYDMAtrQLPyPMxE7fQA3ceQTYjiaVF3D",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x054c752dcc8ecd01c57c0f6238d923a343be7f4b5caff289b3b18aa828e94c2e",
              "version": "204443010",
              "digest": "4ZyXKGvoVrgRt8MXzFoLV2Lo6Cau4f9JVaV2HKvQ5am8",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x0571b0dc38a486690c350b00a4fa1d3715de639eaea44da358abb3cbf6bfdff3",
              "version": "204443010",
              "digest": "3n1pvJgv71Rm7KndZHEdrF3WhdfTdMkzMEYsqTrvqeuJ",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x057b88851b9e39d0f718ddb93213c59195104266d40ee403861097de67cadd12",
              "version": "204443009",
              "digest": "8ov5nQehtR7X85hmw7jmyc2qvFB8jM9BgXQGXa1Ht1z5",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x05a0295f2e2b20131537040d84a03057b044cb8fc35d9e2707e827daaa103e43",
              "version": "204443009",
              "digest": "8h94mavAQ3PD3N2XT2RqzPNdZhid4tYcnbg2RGDMJoFc",
              "balance": "100000000",
              "previousTransaction": "ErMm3QXPzdk6hnWDnm4ikvzGqEoLptGNRbXxf4hetxfZ"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x060106721f9a27d742b4d6dfe8cf240d1088807ac884f81297e80db07933b72e",
              "version": "204443009",
              "digest": "95MZwBssgeR4Ap2LsuqxM798XxrTXZNRftL9V45fCfbL",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x074964684593b5db26c7bedd364eb04358e031533264e37fa4630ad84b72c4db",
              "version": "204443009",
              "digest": "A1dsMg84rTkPuGy35eYvfQPMwHyhWKD5MajhGHyDTeV7",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x07912db60cc3872aa8e5d72442fd1a5bf716d56fe49e35656f5f671ddadda32f",
              "version": "204443010",
              "digest": "Hb6vHTUm7zWE7BadXDfebGwiA4qZx2XUWct57BTWZWpE",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x07f5770a3a77bb38ac14a047eed5eef48b17e6a215f803e3b55a5b85185f8df3",
              "version": "204443009",
              "digest": "HVVyCc5cmthowwiy1yqoRLbHb9gaZXzNdTebYXu6DEYa",
              "balance": "100000000",
              "previousTransaction": "ErMm3QXPzdk6hnWDnm4ikvzGqEoLptGNRbXxf4hetxfZ"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x07fa317c23168c9323c6cac95f25ef56a3c99aa05c641934601d24a09553e647",
              "version": "204443012",
              "digest": "9oH5MfTXX7ofLpLQFvUbPTgAVt5uk731WiHJMsXPX7fT",
              "balance": "100000000",
              "previousTransaction": "37skCQJmUCn5sPJJShi8sk1EhvKMuxjAuN7pT9nEydvi"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x08331c331296daeedf0db448ab5ea214208c1b5c325fe2a6f3fd3a97515d93a4",
              "version": "204443009",
              "digest": "CqbRHiAyxMuzbf8jpGEQa781628mY3KDY4eU1EnHWYha",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x0843c7b6884aa474e5293ccc2859fb3d1728692f53ede91755440f3d9fcdef3c",
              "version": "204443009",
              "digest": "4eN8p8PQxDjdc9ckqH2stjG8VeyicgQnM857GjgQ93Aj",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x084714e3ecf41f41699abe128053f469851a9bd4021a4ca4eb39fa794e42fd97",
              "version": "204443010",
              "digest": "372tDbWkUc4jEdnosLNYWUfJda4JvgE2tS9TNfwsm7AR",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x0847e6d0f41de20ce3dde0d7ca3bae0e6367a3da6858c60b46b15e651e092883",
              "version": "204443009",
              "digest": "3wXmPuCgDXBHJstTEosuEggCkSaHXoy6ZhD9YS1huVbB",
              "balance": "100000000",
              "previousTransaction": "5odsa6V4gKA8w1ZQN9RjyBfjX1qPVd1LTu1DnnbpF7At"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x084c775207a1cd82d2c2f4c9224f03c0c5a75d275c788cf60c1a0d3eb6e6cd33",
              "version": "204443011",
              "digest": "3jyRrKtMo2aCaANGwDCDCDNCy5JRHhsV7v4iWJJJcPFp",
              "balance": "100000000",
              "previousTransaction": "C8reWy1j85aBxeKRweoeW1amkiWBdLmHpdajzFzAh9vN"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x08c21fe0fc06e0646a08e0f45249cdd2ffe2db352b6a5593a5246a0a38377485",
              "version": "204443010",
              "digest": "ArwKyA8rFg1MTgmTUFDC45KUNunUB8QskPd1k1hJFSX9",
              "balance": "100000000",
              "previousTransaction": "67YwHzL7hY1E6CC3WQ7a9M3neAGQpPK56MnLGYHC63Lc"
            },
            {
              "coinType": "0x2::sui::SUI",
              "coinObjectId": "0x091b13426217f690598ba2f86fa16d71b054bce516f0d8d73369b19dd5fd3114",
              "version": "204443012",
              "digest": "5YgtXfLYeqPaSsHsAggdwL9CkENZNCs2am2Wo4sGKf9v",
              "balance": "100000000",
              "previousTransaction": "37skCQJmUCn5sPJJShi8sk1EhvKMuxjAuN7pT9nEydvi"
            }
          ],
          "nextCursor": "eyJjb2luX3R5cGUiOiIweDI6OnN1aTo6U1VJIiwiaW52ZXJ0ZWRfYmFsYW5jZSI6MTg0NDY3NDQwNzM2MDk1NTE2MTUsIm9iamVjdF9pZCI6IjB4MDkxYjEzNDI2MjE3ZjY5MDU5OGJhMmY4NmZhMTZkNzFiMDU0YmNlNTE2ZjBkOGQ3MzM2OWIxOWRkNWZkMzExNCJ9",
          "hasNextPage": true
        });
        let _: CoinPage = serde_json::from_value(value).unwrap();
    }
}
