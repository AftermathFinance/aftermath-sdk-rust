use af_sui_types::decode_base64_default;
use sui_sdk_types::types::Transaction;

// epoch = 594 (testnet)
//
// query epochLastTransaction($epoch: UInt53) {
//   epoch(id: $epoch) {
//     epochId
//     checkpoints(last: 1) {
//       nodes {
//         transactionBlocks(last: 1) {
//           nodes {
//             digest
//             kind {
//               __typename
//             }
//             bcs
//           }
//         }
//       }
//     }
//   }
// }
const BASE64_BCS: &str = "AAUCAlICAAAAAAAAdACtAAAAAAAAUwIAAAAAAABGAAAAAAAAAIDnfFEREwAAGB4InlYEAAAgoP6XPhAAAGAJiQEqAAAAU49m/5MBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAA=";

#[test]
fn transaction_deser() {
    let bytes = decode_base64_default(BASE64_BCS).unwrap();
    let _: Transaction = bcs::from_bytes(&bytes).unwrap();
}
