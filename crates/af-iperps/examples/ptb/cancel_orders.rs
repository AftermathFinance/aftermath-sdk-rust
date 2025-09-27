use std::str::FromStr;

use af_ptbuilder::ptb;
use af_sui_types::{ObjectId, TypeTag};
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::object_args;
use sui_gql_client::reqwest::ReqwestClient;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    rpc: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let Args { rpc } = Args::parse();
    let client = ReqwestClient::new(reqwest::Client::default(), rpc.to_owned());

    let perpetuals_package =
        ObjectId::from_str("0x3bee2aefb42092215d5a22808e2a0abd49f58ca5a8ecfee9634c406370890233")?;
    let account_obj_id =
        ObjectId::from_str("0x2b01d94dcb7a9451306223acefd7867968f881d24ce8f9bc564acc767f32e4b7")?;
    // BTC/USD market
    let ch_id =
        ObjectId::from_str("0xa2c011a2834c6e4db3db1179ea414411b9523739d47b800d9fcd4aa691f94a04")?;
    let otw = TypeTag::from_str(
        "0x457049371f5b5dc2bda857bb804ca6e93c5a3cae1636d0cd17bb6b6070d19458::usdc::USDC",
    )?;

    // Here you would need to fetch the order ids from the orderbook object onchain
    // and get the `order_id` of the orders you want to cancel.
    // You can cancel multiple orders in the same market by passing the order ids
    // in a vector.
    let order_ids = vec![1757315054637997671365303735253u128];

    // Fetch the required objects to cancel orders
    object_args!({
        clearing_house: ch_id,
        account: account_obj_id,
    } with { &client });

    // Similar function exists for `SubAccount`, named `cancel_orders_subaccount`.
    // To use it, you must fetch your subaccount's `ObjectArg` before and pass it instead
    // of `account`
    let ptb = ptb!(
        package perpetuals: perpetuals_package;

        type T = otw.into();

        input obj clearing_house: clearing_house;
        input obj account: account;
        input pure order_ids: &order_ids;

        perpetuals::interface::cancel_orders<T>(clearing_house, account, order_ids);
    );

    println!("PTB: {ptb:?}");

    Ok(())
}
