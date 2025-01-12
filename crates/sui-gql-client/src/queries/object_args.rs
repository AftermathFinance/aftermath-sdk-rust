//! For requesting [`ObjectArg`]s from the server. Defines [`object_args!`](crate::object_args!).
use af_sui_types::{ObjectArg, ObjectId};
pub use bimap::BiMap;
use sui_gql_schema::scalars::{self, UInt53};

use super::fragments::ObjectFilter;
pub use super::objects_flat::Variables;
use crate::{schema, GraphQlClient, GraphQlErrors, PagedResponse};

type Query = super::objects_flat::Query<Object>;

#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    #[error(transparent)]
    Client(T),
    #[error(transparent)]
    Server(#[from] GraphQlErrors),
    #[error("No data in object args query response")]
    NoData,
    #[error("Missing data for object: {0}")]
    MissingObject(ObjectId),
    #[error("Response missing object args for pairs: {0:?}")]
    MissingNamedArgs(Vec<(String, ObjectId)>),
}

/// Turn a bijective map of names and object ids into one of names and object args.
///
/// Fails if the query response does not have the necessary data for the input map.
pub async fn query<C: GraphQlClient>(
    client: &C,
    mut names: BiMap<String, ObjectId>,
    page_size: Option<u32>,
) -> Result<BiMap<String, ObjectArg>, Error<C::Error>> {
    let filter = ObjectFilter {
        object_ids: Some(names.right_values().cloned().collect()),
        type_: None,
        owner: None,
        object_keys: None,
    };
    let vars = Variables {
        filter: Some(filter),
        after: None,
        first: page_size.map(|n| n as i32),
    };
    let response: PagedResponse<Query> = client.query_paged(vars).await.map_err(Error::Client)?;
    let Some((init, pages)) = response.try_into_data()? else {
        return Err(Error::NoData);
    };

    let mut result = BiMap::new();
    for arg in init
        .objects
        .nodes
        .into_iter()
        .chain(pages.into_iter().flat_map(|p| p.objects.nodes))
        .filter_map(|o| o.object_arg())
    {
        if let Some((key, _)) = names.remove_by_right(arg.id_borrowed()) {
            result.insert(key, arg);
        }
    }

    if !names.is_empty() {
        return Err(Error::MissingNamedArgs(names.into_iter().collect()));
    }

    Ok(result)
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
    insta::assert_snapshot!(operation.query);
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
    (
        {$($tt:tt)*}
        with { $client:expr } $(paged by $page_size:expr)?
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

    (@Names mut $name:ident: $_:expr $(, $($rest:tt)*)?) => {
        $crate::object_args!(@Names $name: $_ $(, $($rest)*)?)
    };

    (@Names $name:ident: $_:expr $(, $($rest:tt)*)?) => {
        let $name;
        $crate::object_args!{ @Names $($($rest)*)? }
    };

    (@Names ) => {};

    (@Map $map:ident mut $name:ident: $object_id:expr $(, $($rest:tt)*)?) => {
        $crate::object_args! { @Map $map $name: $object_id $(, $($rest)*)? }
    };

    (@Map $map:ident $name:ident: $object_id:expr $(, $($rest:tt)*)?) => {
        $map.insert(stringify!($name).to_owned(), $object_id);
        $crate::object_args!{ @Map $map $($($rest)*)? }
    };

    (@Map $map:ident) => {};

    (@Result $oargs:ident mut $name:ident: $_:expr $(, $($rest:tt)*)?) => {
        let mut arg = $oargs
            .remove_by_left(stringify!($name))
            .expect("request_named_object_args should fail if any names are missing")
            .1;
        arg.set_mutable(true)?;
        $name = arg;
        $crate::object_args! {@Result $oargs $($($rest)*)?}
    };

    (@Result $oargs:ident $name:ident: $_:expr $(, $($rest:tt)*)?) => {
        $name = $oargs
            .remove_by_left(stringify!($name))
            .expect("request_named_object_args should fail if any names are missing")
            .1;
        $crate::object_args! { @Result $oargs $($($rest)*)? }
    };

    (@Result $oargs:ident ) => {
    };

    (@PageSize $page_size:expr) => { Some($page_size) };
    (@PageSize) => { None };
}

// =============================================================================
//  Inner query fragments
// =============================================================================

#[derive(cynic::QueryFragment, Debug)]
struct Object {
    #[cynic(rename = "address")]
    object_id: ObjectId,
    version: UInt53,
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

fn build_object_arg_default(
    id: ObjectId,
    version: UInt53,
    owner: ObjectOwner,
    digest: Option<scalars::Digest>,
) -> Option<ObjectArg> {
    let version = version.0;

    Some(match owner {
        ObjectOwner::Immutable(_) | ObjectOwner::Parent(_) | ObjectOwner::AddressOwner(_) => {
            ObjectArg::ImmOrOwnedObject((id, version, digest?.0.into()))
        }
        ObjectOwner::Shared(Shared {
            initial_shared_version,
            ..
        }) => ObjectArg::SharedObject {
            id,
            initial_shared_version: initial_shared_version.0,
            mutable: false,
        },
        ObjectOwner::Unknown => return None,
    })
}

pub(super) fn build_oarg_set_mut(
    object_id: ObjectId,
    version: UInt53,
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
    initial_shared_version: UInt53,
}

#[derive(cynic::QueryFragment, Debug)]
pub(super) struct Parent {
    __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub(super) struct AddressOwner {
    __typename: String,
}
