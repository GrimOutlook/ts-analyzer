#![forbid(unsafe_code)]
// Use these checks when closer to complete. They're a bit too strict for early development.
// #![deny(future_incompatible, missing_docs, rust_2018_idioms, unused, warnings)]
#![deny(future_incompatible, missing_docs, rust_2018_idioms)]

//! This crate is used to read the payload data from a given transport stream.

// Include the README in the doc-tests.
#[doc = include_str!("../README.md")]

pub mod reader;

pub mod packet;

mod errors {
    pub mod invalid_first_byte;
    pub mod no_sync_byte_found;
    pub mod no_payload;
    pub mod payload_is_not_start;
    pub mod invalid_payload_pointer;
}

mod helpers {
    pub mod tracked_payload;
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TransportScramblingControl {
    NoScrambling = 0,
    Reserved = 1,
    EvenKey = 2,
    OddKey = 3,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum AdaptationFieldControl {
    Reserved = 0,
    Payload = 1,
    AdaptationField = 2,
    AdaptationAndPayload = 3,
}