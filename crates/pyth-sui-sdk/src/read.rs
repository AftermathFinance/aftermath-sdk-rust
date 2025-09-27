//! JSON-RPC methods for querying Pyth on-chain data.
use af_ptbuilder::ptb;
use af_sui_types::{
    Address as SuiAddress,
    ObjectArg,
    ObjectId,
    TransactionKind,
    encode_base64_default,
};
use sui_framework_sdk::object::ID;
use sui_jsonrpc::api::WriteApiClient;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    JsonRpcClient(#[from] sui_jsonrpc::error::JsonRpcClientError),
    #[error("In ProgrammableTransactionBuilder: {0}")]
    PtBuilder(#[from] af_ptbuilder::Error),
    #[error(transparent)]
    FromHex(#[from] hex::FromHexError),
    #[error("Serializing to BCS: {0}")]
    Bcs(#[from] bcs::Error),
    #[error("DevInspectResults.results is None")]
    DevInspectResults,
}

/// Performs a dev-inspect with a client implementation to return the object ID for an off-chain
/// price identifier.
///
/// Price identifiers can be found in <https://www.pyth.network/developers/price-feed-ids>
pub async fn get_price_info_object_id_from_pyth_state<C>(
    client: &C,
    package: ObjectId,
    price_identifier_hex: String,
    pyth_state: ObjectArg,
) -> Result<ObjectId>
where
    C: WriteApiClient + Sync,
{
    let price_identifier_bytes = &hex::decode(price_identifier_hex.replace("0x", ""))?;

    let inspect_tx = ptb!(
        package pyth: package;

        input obj pyth_state;
        input pure price_identifier_bytes;

        pyth::state::get_price_info_object_id(pyth_state, price_identifier_bytes);
    );

    let mut results = {
        let tx_bytes = encode_base64_default(bcs::to_bytes(
            &TransactionKind::ProgrammableTransaction(inspect_tx),
        )?);
        let resp = client
            .dev_inspect_transaction_block(
                SuiAddress::ZERO, // doesn't matter
                tx_bytes,
                None,
                None,
                None,
            )
            .await?;
        resp.results.ok_or(Error::DevInspectResults)?
    };
    let sui_exec_result = results.swap_remove(0);
    let mut return_values = sui_exec_result.return_values;
    let (bytes, _sui_type_tag) = return_values.swap_remove(0);
    let id: ID = bcs::from_bytes(&bytes)?;
    Ok(id.bytes)
}
