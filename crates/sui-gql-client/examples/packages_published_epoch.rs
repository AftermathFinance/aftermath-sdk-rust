use af_sui_types::ObjectId;
use clap::Parser;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    packages: Vec<ObjectId>,
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let Args { packages, rpc } = Args::parse();

    let client = ReqwestClient::new(reqwest::Client::default(), rpc);

    for (package_id, epoch_id, checkpoint) in client.packages_published_epoch(packages).await? {
        println!("Package: {package_id}, Epoch: {epoch_id}, Checkpoint: {checkpoint}");
    }
    Ok(())
}
