//! This showcases how to obtain a DOF's object ID from its wrapper `Field`'s BCS bytes.
use af_sui_types::{ObjectId, decode_base64_default, hex_address_bytes};
use sui_sdk_types::Object;

// Field wrapper:
// {
//   "data": {
//     "object": {
//       "address": "0x000001d479708e6d43aa5c24e9123874b84f5dd53cd538b377a06cef5e366d10",
//       "bcs": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg1keW5hbWljX2ZpZWxkBUZpZWxkAgcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAhRkeW5hbWljX29iamVjdF9maWVsZAdXcmFwcGVyAQcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgVraW9zawRJdGVtAAcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgZvYmplY3QCSUQAADEpDwEAAAAAYAAAAdR5cI5tQ6pcJOkSOHS4T13VPNU4s3egbO9eNm0QvhD2eKyMAk5FvxFwAKIwssfDany6CNLg5m+8uhx9U5m+EPZ4rIwCTkW/EXAAojCyx8NqfLoI0uDmb7y6HH1TmQHy9nUHfgKdnelfUVVLaXhnO9c3TRsbhJYZWZty27S08SD8V9YtW3Kj2PSMciD4PhTecIJB9EbuaZ+NB4i8B6YCC9BXLQAAAAAA",
//       "asMoveObject": {
//         "contents": {
//           "type": {
//             "repr": "0x0000000000000000000000000000000000000000000000000000000000000002::dynamic_field::Field<0x0000000000000000000000000000000000000000000000000000000000000002::dynamic_object_field::Wrapper<0x0000000000000000000000000000000000000000000000000000000000000002::kiosk::Item>,0x0000000000000000000000000000000000000000000000000000000000000002::object::ID>"
//           }
//         }
//       }
//     }
//   }
// }
const BASE64_BCS: &str = "\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAg1keW5hbWljX2ZpZWxkBUZpZWxkAgcAAAAAAAAAAAAAAAAAAAAAAAA\
AAAAAAAAAAAAAAAAAAhRkeW5hbWljX29iamVjdF9maWVsZAdXcmFwcGVyAQcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
AAAgVraW9zawRJdGVtAAcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgZvYmplY3QCSUQAADEpDwEAAAAAYAAAAdR5c\
I5tQ6pcJOkSOHS4T13VPNU4s3egbO9eNm0QvhD2eKyMAk5FvxFwAKIwssfDany6CNLg5m+8uhx9U5m+EPZ4rIwCTkW/EXAAojCy\
x8NqfLoI0uDmb7y6HH1TmQHy9nUHfgKdnelfUVVLaXhnO9c3TRsbhJYZWZty27S08SD8V9YtW3Kj2PSMciD4PhTecIJB9EbuaZ+\
NB4i8B6YCC9BXLQAAAAAA";

#[test]
fn dof_object_id() {
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

// Dynamic object:
// {
//   "data": {
//     "object": {
//       "address": "0xbe10f678ac8c024e45bf117000a230b2c7c36a7cba08d2e0e66fbcba1c7d5399",
//       "bcs": "AABbjFTpXjps01+uUTWFif0lkYWuKhHoJpkvFSAnnw8iVgtjb2xsZWN0aWJsZRJSZWRCdWxsQ29sbGVjdGlibGUAATEpDwEAAAAAjwa+EPZ4rIwCTkW/EXAAojCyx8NqfLoI0uDmb7y6HH1TmSFMaWdodG5pbmcgVG9yb3MgQ29sbGVjdGlibGUgIzAwMDH1BFRoZSBSZWQgQnVsbCBSYWNpbmcgUkIyMCwgYSBwb3dlcmhvdXNlIG9uIHRoZSBGb3JtdWxhIDEgY2lyY3VpdCwgaXMgYSB0cnVlIG1hcnZlbCBvZiBlbmdpbmVlcmluZyBhbmQgcGVyZm9ybWFuY2UuIFRoaXMgc3RhdGUtb2YtdGhlLWFydCByYWNpbmcgY2FyIGhhcyBiZWVuIHRoZSBiYWNrYm9uZSBvZiBSZWQgQnVsbCBSYWNpbmcncyBjb21wZXRpdGl2ZSBwcm93ZXNzLCBjb25zaXN0ZW50bHkgcHVzaGluZyB0aGUgYm91bmRhcmllcyBvZiBzcGVlZCBhbmQgcHJlY2lzaW9uLiBDcmFmdGVkIHdpdGggcHJlY2lzaW9uIGFuZCBwb3dlcmVkIGJ5IGlubm92YXRpb24sIHRoZSBSQjE5IGlzIGEgZnVzaW9uIG9mIGN1dHRpbmctZWRnZSB0ZWNobm9sb2d5IGFuZCByYXcgc3BlZWQuIFdpdGggaXRzIHNsZWVrIGRlc2lnbiBhbmQgdW5taXN0YWthYmxlIFJlZCBCdWxsIGxpdmVyeSwgaXQgZW1ib2RpZXMgdGhlIGVzc2VuY2Ugb2YgdGhlICJHaXZlcyBZb3UgV2luZ3MiIG1hbnRyYS4gVGhpcyBORlQgY2FyZCBjb21tZW1vcmF0ZXMgdGhlIFJCMjAsIGFuIGVtYmxlbSBvZiByZWxlbnRsZXNzIGRldGVybWluYXRpb24gYW5kIHRoZSBwdXJzdWl0IG9mIGV4Y2VsbGVuY2UgaW4gdGhlIHdvcmxkIG9mIG1vdG9yc3BvcnQuBkNvbW1vbg0xNzAxODc4NTk3MTExAD9odHRwczovL3N0b3JhZ2UuZ29vZ2xlYXBpcy5jb20vcmVkYnVsbC1yYWNpbmctZHJvcC9yYnJfdGVzdC5wbmcBAAAB1Hlwjm1Dqlwk6RI4dLhPXdU81Tizd6Bs7142bRAg/FfWLVtyo9j0jHIg+D4U3nCCQfRG7mmfjQeIvAemAgswqWwAAAAAAA==",
//       "asMoveObject": {
//         "contents": {
//           "type": {
//             "repr": "0x5b8c54e95e3a6cd35fae51358589fd259185ae2a11e826992f1520279f0f2256::collectible::RedBullCollectible"
//           }
//         }
//       }
//     }
//   }
// }
