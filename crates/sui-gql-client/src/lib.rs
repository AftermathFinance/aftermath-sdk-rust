#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]

//! # Sui GraphQL client
//!
//! This is a first version of Aftermath's Sui GraphQL client using [`cynic`].
//!
//! The main item here is the [`GraphQlClient`](crate::GraphQlClient) trait, defining the common
//! interface for clients interacting with an RPC. See the `reqwest` feature for a pre-made
//! implementation.
//!
//! The queries inclued here (under feature `queries`) were constructed with the help of `cynic`s
//! [generator] and use the scalars defined in [`sui_gql_schema`].
//!
//! Users building their own queries should first import [`sui_gql_schema::schema`] and register
//! its schema. Read more about it in <https://cynic-rs.dev/large-apis>. You can check this crate's
//! `[build-dependencies]` and `build.rs` for an example of how to do so.
//!
//! It is recommended to use the [generator] with Sui's GraphQL [schema][sui_schema] and to try
//! reusing the scalars defined in [`sui_gql_schema`] as those automatically convert opaque types to
//! more useful ones like [`af_sui_types`].
//!
//! # Features
//!
//! - `move-types`: compatibility with `af-move-type` types
//! - `mutations`: enables the `mutations` submodule
//! - `queries`: enables the `queries` submodule with pre-made queries
//! - `reqwest`: enables the `reqwest` submodule with an implementation of
//!   [`GraphQlClient`](crate::GraphQlClient)
//! - `scalars`: re-exports the `scalars` module of [`sui_gql_schema`]
//!
//! # Handy links:
//!
//! - Query builder: [generator.cynic-rs.dev][generator]. When prompted either
//!   - click the "A URL" button and pass in:
//!     - `https://sui-testnet.mystenlabs.com/graphql` to build queries against the testnet schema
//!     - `https://sui-mainnet.mystenlabs.com/graphql` for the mainnet one
//!   - click the "I'll Paste It" button and paste the [schema][sui_schema]
//! - Cynic's [guide](https://cynic-rs.dev/)
//!
//! [generator]: https://generator.cynic-rs.dev/
//! [sui_schema]: https://github.com/MystenLabs/sui/blob/main/crates/sui-graphql-rpc/schema.graphql

pub use cynic;
use cynic::schema::{MutationRoot, QueryRoot};
use cynic::serde::de::DeserializeOwned;
use cynic::serde::Serialize;
use cynic::{GraphQlError, GraphQlResponse, Operation, QueryFragment, QueryVariables};
use extension_traits::extension;
#[cfg(feature = "scalars")]
pub use sui_gql_schema::scalars;
pub use sui_gql_schema::schema;

#[cfg(feature = "mutations")]
pub mod mutations;
#[cfg(feature = "queries")]
pub mod queries;
#[cfg(feature = "raw")]
mod raw_client;
#[cfg(feature = "reqwest")]
pub mod reqwest;

pub mod extract;
mod paged;

pub use self::paged::{Paged, PagedResponse, PagesDataResult};
#[cfg(feature = "raw")]
pub use self::raw_client::{Error as RawClientError, RawClient};

/// A generic GraphQL client. Agnostic to the backend used.
#[trait_variant::make(Send)]
pub trait GraphQlClient: Sync {
    type Error: std::error::Error + Send + 'static;

    async fn query_paged<Init>(&self, vars: Init::Input) -> Result<PagedResponse<Init>, Self::Error>
    where
        Init: Paged + Send + 'static,
        Init::SchemaType: QueryRoot,
        Init::Input: Clone,
        Init::NextPage:
            Paged<Input = Init::NextInput, NextInput = Init::NextInput, NextPage = Init::NextPage>,
        <Init::NextPage as QueryFragment>::SchemaType: QueryRoot,
        <Init::NextPage as Paged>::Input: Clone,
    {
        async {
            let initial: GraphQlResponse<Init> = self.query(vars.clone()).await?;
            let mut next_vars = initial.data.as_ref().and_then(|d| d.next_variables(vars));
            let mut pages = vec![];
            while let Some(vars) = next_vars {
                let next_page: GraphQlResponse<Init::NextPage> = self.query(vars.clone()).await?;
                next_vars = next_page.data.as_ref().and_then(|d| d.next_variables(vars));
                pages.push(next_page);
            }
            Ok(PagedResponse(initial, pages))
        }
    }

    async fn query<Query, Variables>(
        &self,
        vars: Variables,
    ) -> Result<GraphQlResponse<Query>, Self::Error>
    where
        Variables: QueryVariables + Send + Serialize,
        Query: DeserializeOwned + QueryFragment<VariablesFields = Variables::Fields> + 'static,
        Query::SchemaType: QueryRoot,
    {
        use cynic::QueryBuilder as _;
        self.run_graphql(Query::build(vars))
    }

    async fn mutation<Mutation, Vars>(
        &self,
        vars: Vars,
    ) -> Result<GraphQlResponse<Mutation>, Self::Error>
    where
        Vars: QueryVariables + Send + Serialize,
        Mutation: DeserializeOwned + QueryFragment<VariablesFields = Vars::Fields> + 'static,
        Mutation::SchemaType: MutationRoot,
    {
        use cynic::MutationBuilder as _;
        self.run_graphql(Mutation::build(vars))
    }

    async fn run_graphql<Query, Vars>(
        &self,
        operation: Operation<Query, Vars>,
    ) -> Result<GraphQlResponse<Query>, Self::Error>
    where
        Vars: Serialize + Send,
        Query: DeserializeOwned + 'static;
}

/// Adds [`try_into_data`](GraphQlResponseExt::try_into_data).
#[extension(pub trait GraphQlResponseExt)]
impl<T> GraphQlResponse<T> {
    /// Extract the `data` field from the response, if any, or fail if the `errors` field contains
    /// any errors.
    fn try_into_data(self) -> Result<Option<T>, GraphQlErrors> {
        if let Some(errors) = self.errors {
            if !errors.is_empty() {
                return Err(GraphQlErrors { errors, page: None });
            }
        }

        let Some(data) = self.data else {
            return Ok(None);
        };
        Ok(Some(data))
    }
}

/// Error for [`GraphQlResponseExt::try_into_data`].
#[derive(thiserror::Error, Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct GraphQlErrors<Extensions = serde::de::IgnoredAny> {
    pub errors: Vec<GraphQlError<Extensions>>,
    pub page: Option<usize>,
}

impl<Extensions> std::fmt::Display for GraphQlErrors<Extensions> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let page_info = self
            .page
            .map_or_else(String::new, |page| format!(" at page {page}"));
        writeln!(
            f,
            "Query execution produced the following errors{page_info}:"
        )?;
        for error in &self.errors {
            writeln!(f, "{error}")?;
        }
        Ok(())
    }
}
