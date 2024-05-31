#![forbid(unsafe_code)]
// Use these checks when closer to complete. They're a bit too strict for early development.
// #![deny(future_incompatible, missing_docs, rust_2018_idioms, unused, warnings)]
#![deny(future_incompatible, missing_docs, rust_2018_idioms)]

//! This crate is used to read the payload data from a given transport stream.

pub mod reader;
pub mod packet;

mod errors {
    pub mod invalid_first_byte;
    pub mod no_sync_byte_found;
}