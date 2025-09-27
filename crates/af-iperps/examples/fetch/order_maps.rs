use af_iperps::graphql::{GraphQlClientExt as _, OrderMaps};
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
        b"0x49bd40cc7880bd358465116157f0271c25d23361b94eace9a25dc2019b449bfc",
    )))]
    ch: ObjectId,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let Args { rpc, ch } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());

    let package = client.object_type(ch).await?.address;
    let OrderMaps {
        orderbook,
        asks,
        bids,
    } = client.order_maps(package, ch).await?;

    println!("Orderbook: {orderbook}");
    println!("Asks Map: {asks}");
    println!("Bids Map: {bids}");
    Ok(())
}
