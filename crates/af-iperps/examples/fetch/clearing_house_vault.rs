use af_iperps::graphql::GraphQlClientExt as _;
use af_move_type::MoveInstance;
use af_sui_types::Address;
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://graphql.testnet.sui.io/graphql")]
    rpc: String,

    #[arg(long, default_value_t = Address::from_static(
        "0xf6f30ee0450f6e3e628b68ac473699f26da5063f74be1868155a8a83b8b45060",
    ))]
    ch: Address,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let Args { rpc, ch } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());

    let package = *client.object_type(ch).await?.address();
    let MoveInstance { type_, value } = client.clearing_house_vault(package, ch).await?;
    println!("{type_}");
    println!("{value}");
    Ok(())
}
