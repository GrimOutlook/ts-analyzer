#![forbid(unsafe_code)]
// Use these checks when closer to complete. They're a bit too strict for early
// development. #![deny(future_incompatible, missing_docs, rust_2018_idioms,
// unused, warnings)]
#![deny(future_incompatible, rust_2018_idioms)]

//! This crate is used to read the payload data from a given transport stream.

use std::error::Error;
use std::fmt::Display;

// Include the README in the doc-tests.
#[doc = include_str!("../README.md")]
pub mod reader;

pub mod packet;

mod helpers {
    pub mod tracked_payload;
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    /// Error that is thrown when trying to parse a byte array as a transport
    /// stream packet, but it doesn't start with a `SYNC_BYTE`.
    #[error("Invalid first byte for transport stream packet `{byte}`")]
    InvalidFirstByte {
        /// Invalid first byte.
        byte: u8,
    },

    /// Error that is thrown when the payload pointer is larger than possible
    /// could possibly fit in the remainder of the packet.
    #[error(
        "Invalid payload pointer `{pointer}` for payload with `{remainder}` bytes remaining"
    )]
    InvalidPayloadPointer { pointer: u8, remainder: u8 },

    /// Error that is thrown when trying read a payload from a packet without a
    /// payload.
    #[error("Cannot read payload from packet that has none")]
    NoPayload,

    /// Error that is thrown when trying to read a transport stream file and no
    /// SYNC byte can be found.
    #[error("Stream contains no SYNC byte")]
    NoSyncByteFound,

    /// Error that is thrown when trying to read the start of a new payload and
    /// no new payload is present in a packet.
    #[error("Continuation payload cannot be used as a starting payload")]
    PayloadIsNotStart,

    #[error(transparent)]
    Unknown(#[from] std::io::Error),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransportScramblingControl {
    NoScrambling = 0,
    Reserved     = 1,
    EvenKey      = 2,
    OddKey       = 3,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AdaptationFieldControl {
    Reserved             = 0,
    Payload              = 1,
    AdaptationField      = 2,
    AdaptationAndPayload = 3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Errors {
    InvalidFirstByte(u8),
}

impl Error for Errors {}

impl Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::InvalidFirstByte(invalid_byte) => {
                write!(f, "invalid first byte for packet: [{}]", invalid_byte)
            }
        }
    }
}
