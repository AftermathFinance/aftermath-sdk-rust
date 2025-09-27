// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::fmt::Display;

use af_sui_types::{
    Address as SuiAddress,
    Identifier,
    ObjectId,
    StructTag,
    TransactionDigest,
    encode_base64_default,
};
use json_to_table::json_to_table;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use serde_with::{DisplayFromStr, IfIsHumanReadable, serde_as};
use tabled::settings::Style as TableStyle;

use super::Page;
use crate::serde::{Base64orBase58, BigInt};

pub type EventPage = Page<SuiEvent, EventID>;

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "Event", rename_all = "camelCase")]
pub struct SuiEvent {
    /// Sequential event ID, ie (transaction seq number, event seq number).
    /// 1) Serves as a unique event ID for each fullnode
    /// 2) Also serves to sequence events for the purposes of pagination and querying.
    ///    A higher id is an event seen later by that fullnode.
    /// This ID is the "cursor" for event querying.
    pub id: EventID,
    /// Move package where this event was emitted.
    pub package_id: ObjectId,
    /// Move module where this event was emitted.
    #[serde_as(as = "DisplayFromStr")]
    pub transaction_module: Identifier,
    /// Sender's Sui address.
    pub sender: SuiAddress,
    /// Move event type.
    // #[serde_as(as = "SuiStructTag")]
    #[serde_as(as = "DisplayFromStr")]
    pub type_: StructTag,
    /// Parsed json value of the event
    pub parsed_json: Value,
    /// Base64 encoded bcs bytes of the move event
    #[serde_as(as = "Base64orBase58")]
    pub bcs: Vec<u8>,
    /// UTC timestamp in milliseconds since epoch (1/1/1970)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<BigInt<u64>>")]
    pub timestamp_ms: Option<u64>,
}

impl Display for SuiEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parsed_json = &mut self.parsed_json.clone();
        bytes_array_to_base64(parsed_json);
        let mut table = json_to_table(parsed_json);
        let style = TableStyle::modern();
        table.collapse().with(style);
        write!(
            f,
            " ┌──\n │ EventID: {}:{}\n │ PackageID: {}\n │ Transaction Module: {}\n │ Sender: {}\n │ EventType: {}\n",
            self.id.tx_digest,
            self.id.event_seq,
            self.package_id,
            self.transaction_module,
            self.sender,
            self.type_
        )?;
        if let Some(ts) = self.timestamp_ms {
            writeln!(f, " │ Timestamp: {}\n └──", ts)?;
        }
        writeln!(f, " │ ParsedJSON:")?;
        let table_string = table.to_string();
        let table_rows = table_string.split_inclusive('\n');
        for r in table_rows {
            write!(f, " │   {r}")?;
        }

        write!(f, "\n └──")
    }
}

/// Convert a json array of bytes to Base64
fn bytes_array_to_base64(v: &mut Value) {
    match v {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => (),
        Value::Array(vals) => {
            if let Some(vals) = vals.iter().map(try_into_byte).collect::<Option<Vec<_>>>() {
                *v = json!(encode_base64_default(vals))
            } else {
                for val in vals {
                    bytes_array_to_base64(val)
                }
            }
        }
        Value::Object(map) => {
            for val in map.values_mut() {
                bytes_array_to_base64(val)
            }
        }
    }
}

/// Try to convert a json Value object into an u8.
fn try_into_byte(v: &Value) -> Option<u8> {
    let num = v.as_u64()?;
    (num <= 255).then_some(num as u8)
}

/// Unique ID of a Sui Event, the ID is a combination of tx seq number and event seq number,
/// the ID is local to the particular fullnode and will be different from other fullnode.
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct EventID {
    pub tx_digest: TransactionDigest,
    #[serde_as(as = "IfIsHumanReadable<BigInt<u64>, _>")]
    pub event_seq: u64,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventFilter {
    /// Return all events.
    All([Box<EventFilter>; 0]),
    /// Query by sender address.
    Sender(SuiAddress),
    /// Return events emitted by the given transaction.
    Transaction(
        ///digest of the transaction, as base-64 encoded string
        TransactionDigest,
    ),
    /// Return events emitted in a specified Move module.
    /// If the event is defined in Module A but emitted in a tx with Module B,
    /// query `MoveModule` by module B returns the event.
    /// Query `MoveEventModule` by module A returns the event too.
    MoveModule {
        /// the Move package ID
        package: ObjectId,
        /// the module name
        #[serde_as(as = "DisplayFromStr")]
        module: Identifier,
    },
    /// Return events with the given Move event struct name (struct tag).
    /// For example, if the event is defined in `0xabcd::MyModule`, and named
    /// `Foo`, then the struct tag is `0xabcd::MyModule::Foo`.
    MoveEventType(#[serde_as(as = "DisplayFromStr")] StructTag),
    /// Return events with the given Move module name where the event struct is defined.
    /// If the event is defined in Module A but emitted in a tx with Module B,
    /// query `MoveEventModule` by module A returns the event.
    /// Query `MoveModule` by module B returns the event too.
    MoveEventModule {
        /// the Move package ID
        package: ObjectId,
        /// the module name
        #[serde_as(as = "DisplayFromStr")]
        module: Identifier,
    },
    /// Return events emitted in [start_time, end_time] interval
    #[serde(rename_all = "camelCase")]
    TimeRange {
        /// left endpoint of time interval, milliseconds since epoch, inclusive
        #[serde_as(as = "BigInt<u64>")]
        start_time: u64,
        /// right endpoint of time interval, milliseconds since epoch, exclusive
        #[serde_as(as = "BigInt<u64>")]
        end_time: u64,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    const NEW_EVENT_JSON: &str = r#"{
    "id": {
        "txDigest": "BwwTktCxZryxsRdQ8JqFNYKYZDLDCQ3L59LCtQzgJgEo",
        "eventSeq": "0"
    },
    "packageId": "0x0000000000000000000000000000000000000000000000000000000000000003",
    "transactionModule": "sui_system",
    "sender": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "type": "0x3::validator::StakingRequestEvent",
    "parsedJson": {
        "amount": "3680004485920",
        "epoch": "0",
        "pool_id": "0x568e13ac056b900ee3ba2f7c85f0c62e19cd25a14ea6f064c3799870ff7d0a9a",
        "staker_address": "0x44b1b319e23495995fc837dafd28fc6af8b645edddff0fc1467f1ad631362c23",
        "validator_address": "0x44b1b319e23495995fc837dafd28fc6af8b645edddff0fc1467f1ad631362c23"
    },
    "bcsEncoding": "base64",
    "bcs": "Vo4TrAVrkA7jui98hfDGLhnNJaFOpvBkw3mYcP99CppEsbMZ4jSVmV/IN9r9KPxq+LZF7d3/D8FGfxrWMTYsI0SxsxniNJWZX8g32v0o/Gr4tkXt3f8PwUZ/GtYxNiwjAAAAAAAAAAAgM1zRWAMAAA==",
    "timestampMs": "1689867116721"
      }"#;

    #[test]
    fn new_bcs_format() {
        serde_json::from_str::<SuiEvent>(NEW_EVENT_JSON).unwrap();
    }
}
