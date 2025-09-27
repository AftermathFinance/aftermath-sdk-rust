use af_iperps::graphql::GraphQlClientExt as _;
use af_move_type::MoveInstance;
use af_sui_types::{ObjectId, hex_address_bytes};
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,

    #[arg(long, default_value_t = ObjectId::new(hex_address_bytes(
        b"0x4264c07a42f9d002c1244e43a1f0fa21c49e4a25c7202c597b8476ef6bb57113",
    )))]
    ch: ObjectId,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let Args { rpc, ch } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());

    let package = client.object_type(ch).await?.address;
    let MoveInstance { type_, value } = client.clearing_house_vault(package, ch).await?;
    println!("{type_}");
    println!("{value}");
    Ok(())
}
