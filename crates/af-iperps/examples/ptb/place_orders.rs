use std::str::FromStr;

use af_iperps::ClearingHouse;
use af_iperps::math::OrderBookUnits;
use af_iperps::order_helpers::{OrderType, Side};
use af_move_type::ObjectExt;
use af_ptbuilder::ptb;
use af_sui_types::sui::object::ObjectHelpers as _;
use af_sui_types::{ObjectArg, ObjectId};
use af_utilities::IFixed;
use clap::Parser;
use color_eyre::Result;
use sui_gql_client::object_args;
use sui_gql_client::queries::GraphQlClientExt as _;
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

    // Fetch the clearing house object reference AND the object contents.
    // The object contents are used to access information such
    // - what oracles object to use in the transaction
    // - what is the `lot_size` and `tick_size` to pass the correct prices to the functions
    let ch_obj = client.full_object(ch_id, None).await?;
    // Get the `ObjectArg` to use in the transaction for the `ClearingHouse` object.
    // This has not been fetched in the `object_args!` macro because it was previously
    // fetched by the `client.full_object` call.
    let ch_oarg = ch_obj.object_arg(true);
    // Deserialize the object into a `ClearingHouse`
    let clearing_house = ch_obj.struct_instance::<ClearingHouse>()?;

    // Size must be expressed in "number of lots". You can obtain the number of lots
    // by starting from a size expressed with `IFixed` (18 decimals). Similar way for `price`.
    let size1 = clearing_house.value.ifixed_to_size(IFixed::one())?; // 1 unit of BTC
    let price1 = clearing_house.value.ifixed_to_price(IFixed::one())?; // Price 1$
    let side1 = Side::Bid;
    let order_type1 = OrderType::PostOnly;

    // It is possible to place multiple orders in the same transaction
    let size2 = clearing_house.value.ifixed_to_size(IFixed::from(2))?; // 2 units of BTC
    let price2 = clearing_house.value.ifixed_to_price(IFixed::from(2))?; // Price 2$
    let side2 = Side::Ask;
    let order_type2 = OrderType::ImmediateOrCancel;

    // Fetch the required objects to perform a trading session
    object_args!({
        account: account_obj_id,
        base_oracle: clearing_house.value.market_params.base_pfs_id.bytes,
        collateral_oracle: clearing_house.value.market_params.collateral_pfs_id.bytes,
    } with { &client });

    let ptb = ptb!(
        package perpetuals: perpetuals_package;

        // The collateral type can also be obtained in this way
        type T = clearing_house.type_.t.into();

        input obj account;
        input obj clearing_house: ch_oarg;
        input obj base_oracle;
        input obj collateral_oracle;
        input obj clock: ObjectArg::CLOCK_IMM;
        input pure side1: &side1;
        input pure size1: &size1;
        input pure price1: &price1;
        input pure order_type1: &order_type1;
        input pure side2: &side2;
        input pure size2: &size2;
        input pure price2: &price2;
        input pure order_type2: &order_type2;

        // Start a trading session by calling `start_session`
        // Similar function exists for `SubAccount`, named `start_session_subaccount`.
        // To use it, you must fetch your subaccount's `ObjectArg` before and pass it instead
        // of `account`
        let hot_potato = perpetuals::interface::start_session<T>(
            clearing_house,
            account,
            base_oracle,
            collateral_oracle,
            clock,
        );

        // Add 1+ `place_limit_order` and/or `place_market_order` here.
        // They will be all executed together. If just one fails, the whole
        // transaction is aborted.
        perpetuals::interface::place_limit_order<T>(
            hot_potato,
            side1,
            size1,
            price1,
            order_type1,
        );
        perpetuals::interface::place_limit_order<T>(
            hot_potato,
            side2,
            size2,
            price2,
            order_type2
        );

        // End the trading session and perform margin checks
        let (ch, _summary) = perpetuals::interface::end_session<T>(hot_potato);

        // This function must be called after a trading session, otherwise the
        // transaction will abort.
        perpetuals::interface::share_clearing_house<T>(ch);
    );

    println!("PTB: {ptb:?}");

    Ok(())
}
