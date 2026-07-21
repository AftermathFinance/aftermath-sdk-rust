<!-- cargo-rdme start -->

Macro to extract data from deeply nested types representing GraphQL results

# Suggested workflow

1. Generate query types using `cynic` and its `generator`
1. Use `insta` to define an inline snapshot test so that the query string is visible in the
   module that defines the query types
1. Define an `extract` function that takes the root query type and returns the data of interest
1. Inside `extract`, use `extract!` as `extract!(data => { ... })`
1. Inside the curly braces, past the query string from the snapshot test above
1. Change all node names from `camelCase` to `snake_case`
1. Add `?` after the nodes that are nullable
1. Add `[]` after the nodes that are iterable

# Examples

The following omits the `derive`s for `cynic` traits that are usually implemented for GraphQL
queries. This is so that we can focus on the nesting of the structures and how the macro helps
to 'extract' the leaves.

```rust
struct Query {
    object: Option<Object>,
}

struct Object {
    dynamic_field: Option<DynamicField>,
}

struct DynamicField {
    value: Option<DynamicFieldValue>,
}

enum DynamicFieldValue {
    MoveValue(MoveValue),
    Unknown,
}

struct MoveValue {
    type_: MoveType,
    bcs: Option<String>,
}

struct MoveType {
    repr: String,
}

fn extract(data: Option<Query>) -> Result<(MoveType, String), &'static str> {
    use graphql_extract::extract;
    use DynamicFieldValue::MoveValue;

    // Leafs become available as variables
    extract!(data => {
        object? {
            dynamic_field? {
                value? {
                    // `MoveValue` is the enum variant name we're interested in
                    ... on MoveValue {
                        type_
                        bcs?
                    }
                }
            }
        }
    });
    Ok((type_, bcs))
}
```

```rust
struct Query {
    address: Option<Address2>,
    object: Option<Object>,
}

struct Address2 {
    address: String,
}

struct Object {
    version: u64,
    dynamic_field: Option<DynamicField>,
    dynamic_fields: DynamicFieldConnection,
}

struct DynamicFieldConnection {
    nodes: Vec<DynamicField>,
}

struct DynamicField {
    value: Option<DynamicFieldValue>,
}

enum DynamicFieldValue {
    MoveValue(MoveValue),
    Unknown,
}

struct MoveValue {
    type_: MoveType,
    bcs: String,
}

struct MoveType {
    repr: String,
}

type Item = Result<(MoveType, String), &'static str>;

fn extract(data: Option<Query>) -> Result<(u64, impl Iterator<Item = Item>), &'static str> {
    use graphql_extract::extract;
    use DynamicFieldValue::MoveValue;

    extract!(data => {
        object? {
            version
            dynamic_fields {
                // `nodes` becomes a variable in the namespace. It implements `Iterator`
                nodes[] {
                    // Everything underneath an iterator node works the same, except it 'maps'
                    // the items of the iterator (check the `Item` type alias above)
                    value? {
                        ... on MoveValue {
                            type_
                            bcs
                        }
                    }
                }
            }
        }
    });
    Ok((version, nodes))
}
```

A caveat to the above is that nested `iterator[]` nodes aren't handled yet. They'll likely be
forbidden in the future.

[cynic]: https://cynic-rs.dev/
[generator]: https://generator.cynic-rs.dev/
[insta]: https://insta.rs/

<!-- cargo-rdme end -->
