//! Define errors used in the parsing of the transport stream.
use std::fmt;

/// Error that is thrown when trying to parse a byte array as a transport stream packet, but it
/// doesn't start with a `SYNC_BYTE`.
#[derive(Debug, Clone)]
pub struct InvalidFirstByteError {
    /// Invalid first byte.
    pub byte: u8,
}

impl std::error::Error for InvalidFirstByteError {}

impl fmt::Display for InvalidFirstByteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid first byte for packet: [{}]", self.byte)
    }
}