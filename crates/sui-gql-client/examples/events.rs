use af_sui_types::Address as SuiAddress;
use clap::Parser;
use color_eyre::eyre::OptionExt as _;
use sui_gql_client::queries::fragments::MoveValueRaw;
use sui_gql_client::reqwest::ReqwestClient;
use sui_gql_client::{scalars, schema, GraphQlClient as _, GraphQlResponseExt as _};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "https://sui-testnet.mystenlabs.com/graphql")]
    url: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let Args { url } = Args::parse();

    let client = ReqwestClient::new(reqwest::Client::default(), url);

    let result: Query = client
        .query(QueryVariables {
            first: Some(1),
            after: None,
            filter: None,
        })
        .await?
        .try_into_data()?
        .ok_or_eyre("No data")?;

    println!("{result:#?}");
    Ok(())
}

#[derive(cynic::QueryVariables, Debug)]
pub struct QueryVariables {
    pub first: Option<i32>,
    pub after: Option<String>,
    pub filter: Option<EventFilter>,
}

#[derive(cynic::InputObject, Debug)]
pub struct EventFilter {
    pub sender: Option<SuiAddress>,
    pub transaction_digest: Option<String>,
    pub emitting_module: Option<String>,
    pub event_type: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "QueryVariables")]
pub struct Query {
    #[arguments(after: $after, first: $first, filter: $filter)]
    pub events: EventConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct EventConnection {
    pub nodes: Vec<Event>,
    pub page_info: PageInfo,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Event {
    pub timestamp: Option<scalars::DateTime>,
    pub contents: Option<MoveValueRaw>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct MoveType {
    pub signature: scalars::MoveTypeSignature,
}
