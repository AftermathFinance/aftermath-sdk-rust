use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

// Execute with
// cargo run --example transaction_blocks_status

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
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
    let tx_blocks = client.transaction_blocks_status(digests).await?;
    for res in tx_blocks {
        let (digest, status) = res?;
        println!("Tx digest: {digest}, Status: {status}");
    }

    Ok(())
}
