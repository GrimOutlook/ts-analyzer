//! Error that is thrown when the payload pointer is larger than possible could possibly fit in
//! the remainder of the packet.
use std::fmt;

/// Error that is thrown when the payload pointer is larger than possible could possibly fit in
/// the remainder of the packet.
#[derive(Debug, Clone)]
pub struct InvalidPayloadPointer {
    pub pointer: u8,
    pub remainder: u8,
}

impl std::error::Error for InvalidPayloadPointer {}

impl fmt::Display for InvalidPayloadPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "payload pointer [{}] is too large for packet remainder [{}]", self.pointer,
               self.remainder)
    }
}