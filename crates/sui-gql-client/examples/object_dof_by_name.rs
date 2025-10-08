use color_eyre::Result;
use serde::Serialize;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::queries::model::outputs::RawMoveValue;
use sui_gql_client::reqwest::ReqwestClient;

const SUI_GRAPHQL_SERVER_URL: &str = "https://graphql.testnet.sui.io/graphql";
const VAULT_OBJECT_ID: &str = "0x69bdd2ad28bc3291ecdffee8e4caeba67b7576e2997ae5f8f9e6cf81dfd423a8";
const VAULT_KEY_TYPE: &str =
    "0x584064bfaac7f3765f3e85b84256f3f10de673c8abb7198c9e9087a8cb5b692d::keys::AccountCapKey";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let client = ReqwestClient::new(
        reqwest::Client::default(),
        SUI_GRAPHQL_SERVER_URL.to_owned(),
    );

    #[derive(Serialize)]
    struct AccountCapKey {
        dummy_field: bool,
    }

    let name_val = AccountCapKey { dummy_field: false };

    let df_name = RawMoveValue {
        type_: VAULT_KEY_TYPE.parse()?,
        bcs: bcs::to_bytes(&name_val)?,
    };
    let result = client
        .object_dof_by_name(VAULT_OBJECT_ID.parse()?, df_name, None)
        .await?;
    println!("{result:?}");
    Ok(())
}
