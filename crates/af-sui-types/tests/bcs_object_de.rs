use af_sui_types::{ObjectId, decode_base64_default, hex_address_bytes};
use sui_sdk_types::Object;
// query Transactions($tx_cursor: String) {
//   transactionBlocks(
//     filter: {
//       changedObject: "0xe4a1c0bfc53a7c2941a433a9a681c942327278b402878e0c45280eecd098c3d1",
//       beforeCheckpoint: 28160634
//     },
//     after: $tx_cursor,
//   ) {
//     pageInfo {
//       hasNextPage
//       endCursor
//     }
//     nodes {
//       digest
//       effects {
//         lamportVersion
//         objectChanges {
//           pageInfo {
//             hasNextPage
//             endCursor
//           }
//           nodes {
//             address
//             idCreated
//             idDeleted
//             outputState {
//               bcs
//               asMoveObject {
//                 contents {
//                   type {
//                     repr
//                   }
//                 }
//               }
//             }
//           }
//         }
//       }
//     }
//   }
// }
// ---
// {
//   "data": {
//     "transactionBlocks": {
//       "pageInfo": {
//         "hasNextPage": true,
//         "endCursor": "eyJjIjozMTI4OTE2NSwidCI6OTg3NDU0OTYwLCJ0YyI6MjU4OTg1NjN9"
//       },
//       "nodes": [
//         {
//           "digest": "3Msbw79PjkxrzpHvSvkHsp3ez1UgFWjiWgfUWYuPU9W2",
//           "effects": {
//             "lamportVersion": 25232856,
//             "objectChanges": {
//               "pageInfo": {
//                 "hasNextPage": true,
//                 "endCursor": "eyJpIjo0LCJjIjozMTI4OTE2NX0"
//               },
//               "nodes": [
//                 {
//                   "address": "0x26a965f75a0bfde46e106e0d860fd656ce9ced5f61e6ad1dcfe80295a40d0a73",
//                   "idCreated": true,
//                   "idDeleted": false,
//                   "outputState": {
//                     "bcs": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg1keW5hbWljX2ZpZWxkBUZpZWxkAgIHt638CGj0NHfXLk46bwiSFJ2G+/VXM06NdPq66jFPqq4Lb3JkZXJlZF9tYXAETGVhZgEHt638CGj0NHfXLk46bwiSFJ2G+/VXM06NdPq66jFPqq4Jb3JkZXJib29rBU9yZGVyAADYBYEBAAAAADEmqWX3Wgv95G4Qbg2GD9ZWzpztX2HmrR3P6AKVpA0KcwEAAAAAAACAAAAAAAAAAAAAAdOoZhJMtfvS8ektsEaUhtssBIX2A5XXYxJTL4wvLIUjICMQNMtGMC8YCN5pxm7P87gUyUgRyx8umoFxNl/ygjMLIBgiAAAAAAA=",
//                     "asMoveObject": {
//                       "contents": {
//                         "type": {
//                           "repr": "0x0000000000000000000000000000000000000000000000000000000000000002::dynamic_field::Field<u64,0xb7adfc0868f43477d72e4e3a6f0892149d86fbf557334e8d74fabaea314faaae::ordered_map::Leaf<0xb7adfc0868f43477d72e4e3a6f0892149d86fbf557334e8d74fabaea314faaae::orderbook::Order>>"
//                         }
//                       }
//                     }
//                   }
//                 },
const BASE64_BCS: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg1keW5hbWljX2ZpZWxkBUZpZWxkAgIHt638CGj0NHfXLk46bwiSFJ2G+/VXM06NdPq66jFPqq4Lb3JkZXJlZF9tYXAETGVhZgEHt638CGj0NHfXLk46bwiSFJ2G+/VXM06NdPq66jFPqq4Jb3JkZXJib29rBU9yZGVyAADYBYEBAAAAADEmqWX3Wgv95G4Qbg2GD9ZWzpztX2HmrR3P6AKVpA0KcwEAAAAAAACAAAAAAAAAAAAAAdOoZhJMtfvS8ektsEaUhtssBIX2A5XXYxJTL4wvLIUjICMQNMtGMC8YCN5pxm7P87gUyUgRyx8umoFxNl/ygjMLIBgiAAAAAAA=";

#[test]
fn object_deser() {
    let bytes = decode_base64_default(BASE64_BCS).unwrap();
    let obj: Object = bcs::from_bytes(&bytes).unwrap();
    assert_eq!(
        obj.object_id(),
        ObjectId::new(hex_address_bytes(
            b"26a965f75a0bfde46e106e0d860fd656ce9ced5f61e6ad1dcfe80295a40d0a73"
        ))
    );
}
