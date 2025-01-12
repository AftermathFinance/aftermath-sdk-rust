#![allow(clippy::unwrap_used)]

fn main() {
    cynic_codegen::register_schema("sui")
        .from_sdl_file("schemas/sui.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
