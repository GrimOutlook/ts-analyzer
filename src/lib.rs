#![forbid(unsafe_code)]
#![deny(future_incompatible, missing_docs, rust_2018_idioms, unused, warnings)]

//! This crate is used to read the payload data from a given transport stream.

pub mod reader;
pub mod packet;

mod errors {
    pub mod invalid_first_byte;
    pub mod no_sync_byte_found;
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
