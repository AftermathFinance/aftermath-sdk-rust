use af_sui_types::ObjectId;
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

// Execute with
// cargo run --example latest_objects_version

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc } = Args::parse();

    let object_ids: Vec<ObjectId> = vec![
        "0x4264c07a42f9d002c1244e43a1f0fa21c49e4a25c7202c597b8476ef6bb57113".parse()?,
        "0x60d1a85f81172a7418206f4b16e1e07e40c91cf58783f63f18a25efc81442dcb".parse()?,
    ];
    let client = ReqwestClient::new(reqwest::Client::default(), rpc);
    let (ckpt_num, versions) = client.latest_objects_version(&object_ids).await?;
    println!("Checkpoint: {ckpt_num}, Version: {versions:?}");
    Ok(())
}
