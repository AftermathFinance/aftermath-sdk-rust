use af_sui_types::{ObjectId, Version};
use cynic::{QueryFragment, QueryVariables};

use crate::{GraphQlClient, GraphQlErrors, GraphQlResponseExt as _, schema};

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum Error<C: std::error::Error> {
    #[error(
        "The latest checkpoint number ({0}) known to the server is smaller than the queried \
        checkpoint"
    )]
    WatermarkTooLow(u64),
    #[error("Missing latest checkpoint number")]
    NoLatestCheckpoint,
    #[error("No data in GraphQL response")]
    NoData,
    #[error("No transaction blocks found for object {id}")]
    NoTransactionBlocks { id: ObjectId },
    #[error("Missing transaction effects")]
    MissingTxEffects,
    #[error("In client: {0}")]
    Client(C),
    #[error("From server: {0}")]
    Server(#[from] GraphQlErrors),
}

/// Get the object's highest version in the network history up to a checkpoint number.
///
/// IMPORTANT: this query uses the transactions table to retrieve the version of the
/// requested object. In case the GQL server is not storing the transactions table, this
/// query is not going to provide the expected results.
///
/// # Arguments
/// - `client`: GQL client implementation
/// - `id`: ID of the object
/// - `ckpt_num`: highest checkpoint to consider; the server will scan the history `<= ckpt_num`
pub async fn query<C: GraphQlClient>(
    client: &C,
    id: ObjectId,
    ckpt_num: u64,
) -> Result<u64, Error<C::Error>> {
    let Some(mut data): Option<Query> = client
        .query(Variables {
            object_id: Some(id),
            checkpoint_num: Some(ckpt_num + 1),
        })
        .await
        .map_err(Error::Client)?
        .try_into_data()?
    else {
        return Err(Error::NoData);
    };

    let watermark = data
        .checkpoint
        .ok_or(Error::NoLatestCheckpoint)?
        .sequence_number;
    if watermark < ckpt_num {
        return Err(Error::WatermarkTooLow(watermark));
    }

    Ok(data
        .transaction_blocks
        .nodes
        .pop()
        .ok_or_else(|| Error::NoTransactionBlocks { id })?
        .effects
        .ok_or(Error::MissingTxEffects)?
        .lamport_version)
}

#[derive(QueryVariables, Debug)]
struct Variables {
    checkpoint_num: Option<Version>,
    object_id: Option<ObjectId>,
}

#[derive(QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "Variables")]
struct Query {
    checkpoint: Option<CheckpointNumber>,

    #[arguments(last: 1, filter: { beforeCheckpoint: $checkpoint_num, changedObject: $object_id })]
    transaction_blocks: TransactionBlockConnection,
}

#[derive(QueryFragment, Debug)]
#[cynic(graphql_type = "Checkpoint")]
struct CheckpointNumber {
    sequence_number: Version,
}

#[derive(QueryFragment, Debug)]
struct TransactionBlockConnection {
    nodes: Vec<TransactionBlock>,
}

#[derive(QueryFragment, Debug)]
struct TransactionBlock {
    effects: Option<TransactionBlockEffects>,
}

#[derive(QueryFragment, Debug)]
struct TransactionBlockEffects {
    lamport_version: Version,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn init_gql_output() {
    use cynic::QueryBuilder as _;
    let vars = Variables {
        object_id: Some(
            "0x4264c07a42f9d002c1244e43a1f0fa21c49e4a25c7202c597b8476ef6bb57113"
                .parse()
                .unwrap(),
        ),
        checkpoint_num: Some(54773328),
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($checkpointNum: UInt53, $objectId: SuiAddress) {
      checkpoint {
        sequenceNumber
      }
      transactionBlocks(last: 1, filter: {beforeCheckpoint: $checkpointNum, changedObject: $objectId}) {
        nodes {
          effects {
            lamportVersion
          }
        }
      }
    }
    "###);
}
