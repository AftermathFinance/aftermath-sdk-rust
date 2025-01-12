use af_sui_types::ObjectId;
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

// Execute with
// cargo run --example packages_from_original

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc } = Args::parse();

    let object_id: ObjectId =
        "0x9725155a70cf2d2241b8cc2fa8376809689312cabb4acaa5ca5ba47eaf4d611f".parse()?;
    let client = ReqwestClient::new(reqwest::Client::default(), rpc);
    let packages = client.packages_from_original(object_id).await?;
    for (id, version) in packages {
        println!("Package id: {id}, Version: {version:?}");
    }

    Ok(())
}
