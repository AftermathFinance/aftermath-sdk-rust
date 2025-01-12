use af_sui_types::ObjectId;
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

// Execute with
// cargo run --example latest_object_version

#[derive(Parser)]
struct Args {
    #[arg(
        long,
        default_value = "0x4264c07a42f9d002c1244e43a1f0fa21c49e4a25c7202c597b8476ef6bb57113"
    )]
    object: ObjectId,

    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { object, rpc } = Args::parse();

    let client = ReqwestClient::new(reqwest::Client::default(), rpc);
    let (ckpt_num, version) = client.latest_object_version(object).await?;
    println!("Checkpoint: {ckpt_num}, Version: {version}");
    Ok(())
}
