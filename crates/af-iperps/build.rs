fn main() {
    #[cfg(feature = "graphql")]
    sui_gql_schema::register_schemas();
}
