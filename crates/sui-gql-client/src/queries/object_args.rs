//! For requesting [`ObjectArg`]s from the server. Defines [`object_args!`](crate::object_args!).
use af_sui_types::{ObjectArg, ObjectId, Version};
use bimap::BiMap;
use futures::TryStreamExt as _;
use itertools::Itertools as _;
use sui_gql_schema::scalars;

use super::fragments::ObjectFilterV2;
use super::objects_flat::Variables;
use crate::{GraphQlClient, GraphQlErrors, GraphQlResponseExt as _, schema};

type Query = super::objects_flat::Query<Object>;

// WARN: can't call this `Result`, otherwise it will mess with the code generated by the derive
// macros in this module.
type Res<T, C> = std::result::Result<T, Error<<C as GraphQlClient>::Error>>;

#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    #[error(transparent)]
    Client(T),
    #[error(transparent)]
    Server(#[from] GraphQlErrors),
    #[error("No data in object args query response")]
    NoData,
    #[error("Response missing object args for pairs: {0:?}")]
    MissingNamedArgs(Vec<(String, ObjectId)>),
}

/// Turn a bijective map of names and object ids into one of names and object args.
///
/// Fails if the query response does not have the necessary data for the input map.
pub(super) async fn query<C: GraphQlClient>(
    client: &C,
    mut names: BiMap<String, ObjectId>,
    page_size: Option<u32>,
) -> Res<BiMap<String, ObjectArg>, C> {
    let object_ids = names.right_values().cloned().collect_vec();
    let filter = ObjectFilterV2 {
        object_ids: Some(&object_ids),
        type_: None,
        owner: None,
    };
    let vars = Variables {
        filter: Some(filter),
        after: None,
        first: page_size.map(|n| n as i32),
    };

    let mut stream = std::pin::pin!(super::stream::forward(client, vars, request));

    let mut result = BiMap::new();

    while let Some(arg) = stream.try_next().await? {
        if let Some((key, _)) = names.remove_by_right(arg.id_borrowed()) {
            result.insert(key, arg);
        }
    }

    if !names.is_empty() {
        return Err(Error::MissingNamedArgs(names.into_iter().collect()));
    }

    Ok(result)
}

async fn request<C: GraphQlClient>(
    client: &C,
    vars: Variables<'_>,
) -> Res<super::stream::Page<impl Iterator<Item = Res<ObjectArg, C>> + 'static + use<C>>, C> {
    let objects = client
        .query::<Query, _>(vars)
        .await
        .map_err(Error::Client)?
        .try_into_data()?
        .ok_or(Error::NoData)?
        .objects;

    Ok(super::stream::Page {
        info: objects.page_info.into(),
        data: objects
            .nodes
            .into_iter()
            .filter_map(Object::object_arg)
            .map(Ok),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[test]
fn gql_output() {
    use cynic::QueryBuilder as _;
    let vars = Variables {
        filter: None,
        first: None,
        after: None,
    };
    let operation = Query::build(vars);
    insta::assert_snapshot!(operation.query, @r###"
    query Query($filter: ObjectFilter, $after: String, $first: Int) {
      objects(filter: $filter, first: $first, after: $after) {
        nodes {
          address
          version
          digest
          owner {
            __typename
            ... on Immutable {
              _
            }
            ... on Shared {
              __typename
              initialSharedVersion
            }
            ... on Parent {
              __typename
            }
            ... on AddressOwner {
              __typename
            }
          }
        }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
    "###);
}

// =============================================================================
//  Macro helper
// =============================================================================

/// Query [ObjectArg]s and assign them to variables. Optionally, set the page size.
///
/// This will panic if the user specifies two different identifiers mapping to the same [ObjectId].
///
/// The `mut` keyword here means we're requesting a mutable [ObjectArg::SharedObject].
///
/// # Example
/// ```no_run
/// # use color_eyre::Result;
/// # use sui_gql_client::object_args;
/// # use sui_gql_client::reqwest::ReqwestClient;
/// # const SUI_GRAPHQL_SERVER_URL: &str = "https://sui-testnet.mystenlabs.com/graphql";
/// # tokio_test::block_on(async {
/// let client = ReqwestClient::new(
///     reqwest::Client::default(),
///     SUI_GRAPHQL_SERVER_URL.to_owned(),
/// );
/// object_args!({
///     mut clearing_house: "0xe4a1c0bfc53a7c2941a433a9a681c942327278b402878e0c45280eecd098c3d1".parse()?,
///     registry: "0x400e84251a6ce2192f69c1aa775d68bab7690e059578317bf9e844d40e07e04d".parse()?,
/// } with { &client } paged by 10);
/// # println!("{clearing_house:?}");
/// # println!("{registry:?}");
/// # Ok::<_, color_eyre::eyre::Error>(())
/// # });
/// ```
#[macro_export]
macro_rules! object_args {
    // Optimization: 1 object arg only
    (
        { $name:ident: $object_id:expr_2021 $(,)? }
        with { $client:expr_2021 }
    ) => {
        let $name = $crate::queries::GraphQlClientExt::object_arg($client, $object_id)
            .await?;
    };

    (
        { mut $name:ident: $object_id:expr_2021 $(,)? }
        with { $client:expr_2021 }
    ) => {
        let $name = {
            let mut oarg = $crate::queries::GraphQlClientExt::object_arg($client, $object_id)
                .await?;
            oarg.set_mutable(true)?;
            oarg
        };
    };

    (
        {$($tt:tt)*}
        with { $client:expr_2021 } $(paged by $page_size:expr_2021)?
    ) => {
        $crate::object_args!(@Names $($tt)*);
        {
            use $crate::queries::GraphQlClientExt as _;
            let mut names = $crate::queries::BiMap::new();
            $crate::object_args! { @Map names $($tt)* }
            let mut oargs = $crate::queries::GraphQlClientExt::object_args(
                $client,
                names,
                $crate::object_args!(@PageSize $($page_size)?)
            ).await?;
            $crate::object_args! { @Result oargs $($tt)* }
        }
    };

    (@Names mut $name:ident: $_:expr_2021 $(, $($rest:tt)*)?) => {
        $crate::object_args!(@Names $name: $_ $(, $($rest)*)?)
    };

    (@Names $name:ident: $_:expr_2021 $(, $($rest:tt)*)?) => {
        let $name;
        $crate::object_args!{ @Names $($($rest)*)? }
    };

    (@Names ) => {};

    (@Map $map:ident mut $name:ident: $object_id:expr_2021 $(, $($rest:tt)*)?) => {
        $crate::object_args! { @Map $map $name: $object_id $(, $($rest)*)? }
    };

    (@Map $map:ident $name:ident: $object_id:expr_2021 $(, $($rest:tt)*)?) => {
        $map.insert(stringify!($name).to_owned(), $object_id);
        $crate::object_args!{ @Map $map $($($rest)*)? }
    };

    (@Map $map:ident) => {};

    (@Result $oargs:ident mut $name:ident: $_:expr_2021 $(, $($rest:tt)*)?) => {
        let mut arg = $oargs
            .remove_by_left(stringify!($name))
            .expect("request_named_object_args should fail if any names are missing")
            .1;
        arg.set_mutable(true)?;
        $name = arg;
        $crate::object_args! {@Result $oargs $($($rest)*)?}
    };

    (@Result $oargs:ident $name:ident: $_:expr_2021 $(, $($rest:tt)*)?) => {
        $name = $oargs
            .remove_by_left(stringify!($name))
            .expect("request_named_object_args should fail if any names are missing")
            .1;
        $crate::object_args! { @Result $oargs $($($rest)*)? }
    };

    (@Result $oargs:ident ) => {
    };

    (@PageSize $page_size:expr_2021) => { Some($page_size) };
    (@PageSize) => { None };
}

// =============================================================================
//  Inner query fragments
// =============================================================================

#[derive(cynic::QueryFragment, Debug)]
struct Object {
    #[cynic(rename = "address")]
    object_id: ObjectId,
    version: Version,
    digest: Option<scalars::Digest>,
    owner: Option<ObjectOwner>,
}

impl Object {
    /// Return the [ObjectArg] or none if missing data.
    ///
    /// For shared objects, `mutable` is set as `false`. Use [ObjectArg::set_mutable] if needed.
    fn object_arg(self) -> Option<ObjectArg> {
        let Self {
            object_id,
            version,
            digest,
            owner: Some(owner),
        } = self
        else {
            return None;
        };

        build_object_arg_default(object_id, version, owner, digest)
    }
}

pub(crate) fn build_object_arg_default(
    id: ObjectId,
    version: Version,
    owner: ObjectOwner,
    digest: Option<scalars::Digest>,
) -> Option<ObjectArg> {
    Some(match owner {
        ObjectOwner::Immutable(_) | ObjectOwner::Parent(_) | ObjectOwner::AddressOwner(_) => {
            ObjectArg::ImmOrOwnedObject((id, version, digest?.0.into()))
        }
        ObjectOwner::Shared(Shared {
            initial_shared_version,
            ..
        }) => ObjectArg::SharedObject {
            id,
            initial_shared_version,
            mutable: false,
        },
        ObjectOwner::Unknown => return None,
    })
}

pub(super) fn build_oarg_set_mut(
    object_id: ObjectId,
    version: Version,
    owner: Option<ObjectOwner>,
    digest: Option<scalars::Digest>,
    mutable_: bool,
) -> Option<ObjectArg> {
    let mut oarg = build_object_arg_default(object_id, version, owner?, digest)?;
    if let ObjectArg::SharedObject {
        ref mut mutable, ..
    } = oarg
    {
        *mutable = mutable_;
    }
    Some(oarg)
}

#[derive(cynic::InlineFragments, Debug)]
pub(super) enum ObjectOwner {
    #[allow(dead_code)]
    Immutable(Immutable),

    Shared(Shared),

    #[allow(dead_code)]
    Parent(Parent),

    #[allow(dead_code)]
    AddressOwner(AddressOwner),

    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
pub(super) struct Immutable {
    #[cynic(rename = "_")]
    __underscore: Option<bool>,
}

#[derive(cynic::QueryFragment, Debug)]
pub(super) struct Shared {
    __typename: String,
    initial_shared_version: Version,
}

#[derive(cynic::QueryFragment, Debug)]
pub(super) struct Parent {
    __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub(super) struct AddressOwner {
    __typename: String,
}
