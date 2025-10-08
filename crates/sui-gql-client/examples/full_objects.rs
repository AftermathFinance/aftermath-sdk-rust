use clap::Parser;
use color_eyre::Result;
use sui_gql_client::queries::GraphQlClientExt as _;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://graphql.testnet.sui.io/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());
    let keys = vec![
        (
            "0xf996a61534f678bd01ff43a7ce1762b0accab7b76725086bda35f74b36478705".parse()?,
            None,
        ),
        // This object ID does not exist. Uncomment to make the example fail.
        // (
        //     "0xf996a61534f678bd01ff43a7ce1762b0accab7b76725086bda35f74b36478704".parse()?,
        //     None,
        // ),
    ];

    let ckpt = Some(250054888);
    let objects = client.full_objects(keys, ckpt).await?;

    println!("Objects count: {}", objects.len());
    for obj in objects {
        println!("Object: {obj:?}");
    }

    Ok(())
}
