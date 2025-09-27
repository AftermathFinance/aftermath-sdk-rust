use af_sui_types::ObjectId;
use color_eyre::Result;
use serde::Serialize;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::queries::outputs::RawMoveValue;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://sui-testnet.mystenlabs.com/graphql";
const LINKED_TABLE: &str = "0x4e793859f3276804b8eecfd0fdf07b215e28ffd59b4fefd9605d072c7bce6457";
const SOURCE_WRAPPER_ID: &str =
    "0xbd621bc8d567577e45ded1425271828d3b69f444334d2e48b3366d453e4d9941";
const LINKED_TABLE_KEY_TYPE: &str =
    "0x241537381737a40df6838bc395fb64f04ff604513c18a2ac3308ac810c805fa6::keys::PriceFeedForSource";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );

    #[derive(Serialize)]
    struct PriceFeedForSource {
        source_wrapper_id: ObjectId,
    }

    let name_val = PriceFeedForSource {
        source_wrapper_id: SOURCE_WRAPPER_ID.parse()?,
    };

    let df_name = RawMoveValue {
        type_: LINKED_TABLE_KEY_TYPE.parse()?,
        bcs: bcs::to_bytes(&name_val)?,
    };
    let result = client
        .owner_df_content(LINKED_TABLE.parse()?, df_name, None)
        .await?;
    println!("{result:?}");
    Ok(())
}
