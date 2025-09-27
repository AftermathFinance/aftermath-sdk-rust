use af_sui_types::{ObjectId, decode_base64_default, hex_address_bytes};
use sui_sdk_types::{Object, Transaction};

/// This showcases how to obtain a DOF's object ID from its wrapper `Field`'s BCS bytes.
///
/// From the query result:
/// {
///   "data": {
///     "object": {
///       "address": "0x000001d479708e6d43aa5c24e9123874b84f5dd53cd538b377a06cef5e366d10",
///       "bcs": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg1keW5hbWljX2ZpZWxkBUZpZWxkAgcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAhRkeW5hbWljX29iamVjdF9maWVsZAdXcmFwcGVyAQcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgVraW9zawRJdGVtAAcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgZvYmplY3QCSUQAADEpDwEAAAAAYAAAAdR5cI5tQ6pcJOkSOHS4T13VPNU4s3egbO9eNm0QvhD2eKyMAk5FvxFwAKIwssfDany6CNLg5m+8uhx9U5m+EPZ4rIwCTkW/EXAAojCyx8NqfLoI0uDmb7y6HH1TmQHy9nUHfgKdnelfUVVLaXhnO9c3TRsbhJYZWZty27S08SD8V9YtW3Kj2PSMciD4PhTecIJB9EbuaZ+NB4i8B6YCC9BXLQAAAAAA",
///       "asMoveObject": {
///         "contents": {
///           "type": {
///             "repr": "0x0000000000000000000000000000000000000000000000000000000000000002::dynamic_field::Field<0x0000000000000000000000000000000000000000000000000000000000000002::dynamic_object_field::Wrapper<0x0000000000000000000000000000000000000000000000000000000000000002::kiosk::Item>,0x0000000000000000000000000000000000000000000000000000000000000002::object::ID>"
///           }
///         }
///       }
///     }
///   }
/// }
#[test]
fn dof_object_id() {
    const BASE64_BCS: &str = "\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg1keW5hbWljX2ZpZWxkBUZpZWxkAgcAAAAAAAAAAAAAAAAAAAAAAAA\
AAAAAAAAAAAAAAAAAAhRkeW5hbWljX29iamVjdF9maWVsZAdXcmFwcGVyAQcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
AAAgVraW9zawRJdGVtAAcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgZvYmplY3QCSUQAADEpDwEAAAAAYAAAAdR5c\
I5tQ6pcJOkSOHS4T13VPNU4s3egbO9eNm0QvhD2eKyMAk5FvxFwAKIwssfDany6CNLg5m+8uhx9U5m+EPZ4rIwCTkW/EXAAojCy\
x8NqfLoI0uDmb7y6HH1TmQHy9nUHfgKdnelfUVVLaXhnO9c3TRsbhJYZWZty27S08SD8V9YtW3Kj2PSMciD4PhTecIJB9EbuaZ+\
NB4i8B6YCC9BXLQAAAAAA";

    let bytes = decode_base64_default(BASE64_BCS).unwrap();
    let wrapper: Object = bcs::from_bytes(&bytes).unwrap();
    let move_object = wrapper.as_struct().expect("Not a Move object");
    let contents = &move_object.contents();
    println!("{wrapper:#?}");

    let id = ObjectId::new(contents[(contents.len() - 32)..].try_into().unwrap());

    let expected = ObjectId::new(hex_address_bytes(
        b"be10f678ac8c024e45bf117000a230b2c7c36a7cba08d2e0e66fbcba1c7d5399",
    ));
    assert_eq!(id, expected);
}

/// This shows that [`Transaction`] is the equivalent of the old `TransactionData` (in BCS terms).
///
/// epoch = 594 (testnet)
///
/// query epochLastTransaction($epoch: UInt53) {
///   epoch(id: $epoch) {
///     epochId
///     checkpoints(last: 1) {
///       nodes {
///         transactionBlocks(last: 1) {
///           nodes {
///             digest
///             kind {
///               __typename
///             }
///             bcs
///           }
///         }
///       }
///     }
///   }
/// }
#[test]
fn transaction_deser() {
    const BASE64_BCS: &str = "AAUCAlICAAAAAAAAdACtAAAAAAAAUwIAAAAAAABGAAAAAAAAAIDnfFEREwAAGB4InlYEAAAgoP6XPhAAAGAJiQEqAAAAU49m/5MBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAA=";

    let bytes = decode_base64_default(BASE64_BCS).unwrap();
    let _: Transaction = bcs::from_bytes(&bytes).unwrap();
}

#[test]
fn object_deser() {
    const BASE64_BCS: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg1keW5hbWljX2ZpZWxkBUZpZWxkAgIHt638CGj0NHfXLk46bwiSFJ2G+/VXM06NdPq66jFPqq4Lb3JkZXJlZF9tYXAETGVhZgEHt638CGj0NHfXLk46bwiSFJ2G+/VXM06NdPq66jFPqq4Jb3JkZXJib29rBU9yZGVyAADYBYEBAAAAADEmqWX3Wgv95G4Qbg2GD9ZWzpztX2HmrR3P6AKVpA0KcwEAAAAAAACAAAAAAAAAAAAAAdOoZhJMtfvS8ektsEaUhtssBIX2A5XXYxJTL4wvLIUjICMQNMtGMC8YCN5pxm7P87gUyUgRyx8umoFxNl/ygjMLIBgiAAAAAAA=";

    let bytes = decode_base64_default(BASE64_BCS).unwrap();
    let obj: Object = bcs::from_bytes(&bytes).unwrap();
    insta::assert_snapshot!(obj.object_id(), @"0x26a965f75a0bfde46e106e0d860fd656ce9ced5f61e6ad1dcfe80295a40d0a73");
    assert_eq!(obj.version(), 25232856);
    insta::assert_snapshot!(obj.digest().to_base58(), @"F8wHxvPV3CuKLm25wb7B4xL64stTGChfjJpj8RYSyWHX");
}
