use af_sui_types::{ObjectId, Version};
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(
        long,
        default_value = "0x9725155a70cf2d2241b8cc2fa8376809689312cabb4acaa5ca5ba47eaf4d611f"
    )]
    package: ObjectId,

    #[arg(long, default_value_t = 1)]
    version: Version,

    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args {
        package,
        version,
        rpc,
    } = Args::parse();

    let client = ReqwestClient::new_default(rpc);
    let address = client.package_at_version(package, version).await?;
    println!("Address: {address}");
    Ok(())
}
