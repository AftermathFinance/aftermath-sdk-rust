use color_eyre::Result;
use serde::Serialize;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::queries::model::outputs::RawMoveValue;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://graphql.testnet.sui.io/graphql";
const CH: &str = "0xf6f30ee0450f6e3e628b68ac473699f26da5063f74be1868155a8a83b8b45060";
const VAULT_KEY_TYPE: &str =
    "0xe40e00181e9ffb522589cd329d67048eeff18f76dd7c08a6914193e388a94230::keys::MarketVault";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );

    #[derive(Serialize)]
    struct MarketVault {
        dummy_field: bool,
    }

    let name_val = MarketVault { dummy_field: false };

    let df_name = RawMoveValue {
        type_: VAULT_KEY_TYPE.parse()?,
        bcs: bcs::to_bytes(&name_val)?,
    };
    let result = client.object_df_by_name(CH.parse()?, df_name, None).await?;
    println!("{result:?}");
    Ok(())
}
