use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

// Execute with
// cargo run --example transaction_blocks_status

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://graphql.testnet.sui.io/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc } = Args::parse();

    let digests: Vec<String> = vec![
        "on9FwM19hquj2tapUCH9WxfVHMFW5YwhK66Hna7TpzP".into(),
        "BpkRMRTeG4WuoKPcH8n881XrVcDWVSfDz1yMRfSuvFyR".into(),
    ];
    let client = ReqwestClient::new(reqwest::Client::default(), rpc);
    let tx_blocks = client.transactions(digests).await?;
    for res in tx_blocks {
        println!("Tx digest: {}", res.digest());
    }

    Ok(())
}
