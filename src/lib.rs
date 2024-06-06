#![forbid(unsafe_code)]
// Use these checks when closer to complete. They're a bit too strict for early development.
// #![deny(future_incompatible, missing_docs, rust_2018_idioms, unused, warnings)]
#![deny(future_incompatible, missing_docs, rust_2018_idioms)]

//! This crate is used to read the payload data from a given transport stream.

// Include the README in the doc-tests.
#[doc = include_str!("../README.md")]

pub mod reader;
pub mod packet;

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

/// Errors that can be created by this application
#[derive(Debug)]
pub enum TSError {
    /// Errors generated by reading data from a file or buffer
    ReaderError(std::io::Error),
    /// Error generated when trying to parse a TS packet with an invalid first byte.
    InvalidFirstByte(u8 /* invalid byte */),
    /// Error generated when trying to parse the payload of a TS packet and the payload pointer
    /// points past the end of the packet.
    InvalidPayloadPointer(u8 /* pointer */, u8 /* remainder */),
    /// Error generated when trying to do operations on the payload of a packet but no payload is
    /// present
    NoPayload,
    /// Error generated when no SYNC byte can be found in a reader object
    NoSyncByte,
    /// Error generated when trying to do operations on a TS packet that is expected to have a new
    /// payload in it but the `PUSI` flag isn't set.
    PUSIsNotSet,
}

impl std::error::Error for TSError {}

impl std::fmt::Display for TSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TSError::InvalidFirstByte(invalid_byte) => 
                write!(f, "invalid first byte for packet: [{}]", invalid_byte),
            TSError::InvalidPayloadPointer(pointer, remainder) =>
                write!(f, "payload pointer [{}] is too large for packet remainder [{}]",
                    pointer, remainder),
            TSError::NoPayload => write!(f, "no payload found in packet"),
            TSError::NoSyncByte => write!(f, "no sync byte found in reader"),
            TSError::PUSIsNotSet =>
                write!(f, "payload does not contain the start of a new partial payload"),
            TSError::ReaderError(error) => write!(f, "reader error: {}", error),
        }
    }
}

/// TSError equivalence checking only checks to see if the variants are the same and ignores the
/// the stored data.
impl std::cmp::PartialEq for TSError {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::discriminant;
        discriminant(self) == discriminant(other)
    }
}
