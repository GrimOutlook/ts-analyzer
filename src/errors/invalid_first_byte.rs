//! Define invalid_first_byte used in the parsing of the transport stream.
use std::fmt;

/// Error that is thrown when trying to parse a byte array as a transport stream packet, but it
/// doesn't start with a `SYNC_BYTE`.
#[derive(Debug, Clone)]
pub struct InvalidFirstByte {
    /// Invalid first byte.
    pub byte: u8,
}

impl std::error::Error for InvalidFirstByte {}

impl fmt::Display for InvalidFirstByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid first byte for packet: [{}]", self.byte)
    }
}