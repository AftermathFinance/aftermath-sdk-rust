<!-- cargo-rdme start -->

# Sui GraphQL client

This is a first version of Aftermath's Sui GraphQL client using [`cynic`].

The main item here is the `GraphQlClient` trait, defining the common
interface for clients interacting with an RPC. See the `reqwest` feature for a pre-made
implementation.

The queries inclued here (under feature `queries`) were constructed with the help of `cynic`s
[generator] and use the scalars defined in [`sui_gql_schema`].

Users building their own queries should first import [`sui_gql_schema::schema`] and register
its schema. Read more about it in <https://cynic-rs.dev/large-apis>. You can check this crate's
`[build-dependencies]` and `build.rs` for an example of how to do so.

It is recommended to use the [generator] with Sui's GraphQL [schema][sui_schema] and to try
reusing the scalars defined in [`sui_gql_schema`] as those automatically convert opaque types to
more useful ones like [`af_sui_types`].

# Features

- `move-types`: compatibility with `af-move-type` types
- `mutations`: enables the `mutations` submodule
- `queries`: enables the `queries` submodule with pre-made queries
- `reqwest`: enables the `reqwest` submodule with an implementation of
  `GraphQlClient`
- `scalars`: re-exports the `scalars` module of [`sui_gql_schema`]

# Handy links:

- Query builder: [generator.cynic-rs.dev][generator]. When prompted either
  - click the "A URL" button and pass in:
    - `https://sui-testnet.mystenlabs.com/graphql` to build queries against the testnet schema
    - `https://sui-mainnet.mystenlabs.com/graphql` for the mainnet one
  - click the "I'll Paste It" button and paste the [schema][sui_schema]
- Cynic's [guide](https://cynic-rs.dev/)

[generator]: https://generator.cynic-rs.dev/
[sui_schema]: https://github.com/MystenLabs/sui/blob/main/crates/sui-graphql-rpc/schema.graphql

<!-- cargo-rdme end -->
