use af_sui_types::{Address as SuiAddress, ObjectId, ObjectRef, Version};
use cynic::GraphQlResponse;

use super::fragments::PageInfoForward;
use crate::scalars::{BigInt, Digest};
use crate::{schema, GraphQlClient, GraphQlErrors, GraphQlResponseExt as _};

#[derive(thiserror::Error, Debug)]
pub enum Error<E> {
    #[error("Missing data in initial response")]
    MissingInitialData,
    #[error("Not enough SUI coins found for budget {budget}")]
    NotEnoughSui { budget: u64 },
    #[error("Missing data in page response")]
    PageMissingData,
    #[error("Missing coin balance data")]
    MissingCoinBalance,
    #[error("Missing coin digest data")]
    MissingCoinDigest,
    #[error(transparent)]
    Client(E),
    #[error(transparent)]
    Server(#[from] GraphQlErrors),
}

pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    sponsor: SuiAddress,
    budget: u64,
    exclude: Vec<ObjectId>,
) -> Result<Vec<ObjectRef>, Error<C::Error>> {
    let mut vars = Variables {
        address: sponsor,
        first: None,
        after: None,
    };
    let query: Query = client
        .query(vars.clone())
        .await
        .map_err(Error::Client)?
        .try_into_data()
        .map_err(Error::Server)?
        .ok_or(Error::MissingInitialData)?;

    let Query {
        address: Some(Address {
            coins: mut connection,
        }),
    } = query
    else {
        return Err(Error::MissingInitialData);
    };

    let mut coins = vec![];
    let mut balance = 0;
    loop {
        let CoinConnection { nodes, page_info } = connection;
        for Coin {
            object_id,
            version,
            digest,
            coin_balance,
        } in nodes
            .into_iter()
            .filter(|n| !exclude.contains(&n.object_id))
        {
            let digest = digest.ok_or(Error::MissingCoinDigest)?;
            let coin_balance = coin_balance.ok_or(Error::MissingCoinBalance)?.into_inner();
            coins.push((object_id, version, digest.0.into()));
            balance += coin_balance;
            if balance >= budget {
                return Ok(coins);
            }
        }

        if !page_info.has_next_page {
            break;
        }
        vars.after = page_info.end_cursor;
        connection = {
            let response: GraphQlResponse<Query> =
                client.query(vars.clone()).await.map_err(Error::Client)?;
            let Some(Query {
                address: Some(Address { coins }),
            }) = response.try_into_data()?
            else {
                return Err(Error::PageMissingData);
            };
            coins
        };
    }

    Err(Error::NotEnoughSui { budget })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;

    let vars = Variables {
        address: SuiAddress::new(rand::random()),
        first: None,
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query);
}

// =============================================================================
//  Inner query fragments
// =============================================================================

#[derive(cynic::QueryVariables, Clone, Debug)]
struct Variables {
    address: SuiAddress,
    first: Option<i32>,
    after: Option<String>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Query {
    #[arguments(address: $address)]
    address: Option<Address>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(variables = "Variables")]
struct Address {
    #[arguments(type: "0x2::sui::SUI", first: $first, after: $after)]
    coins: CoinConnection,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct CoinConnection {
    nodes: Vec<Coin>,
    page_info: PageInfoForward,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
struct Coin {
    #[cynic(rename = "address")]
    object_id: ObjectId,
    version: Version,
    digest: Option<Digest>,
    coin_balance: Option<BigInt<u64>>,
}
