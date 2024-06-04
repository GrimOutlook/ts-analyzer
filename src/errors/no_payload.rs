//! Error that is thrown when trying read a payload from a packet without a payload.
use std::fmt;

/// Error that is thrown when trying read a payload from a packet without a payload.
#[derive(Debug, Clone)]
pub struct NoPayload;

impl std::error::Error for NoPayload {}

impl fmt::Display for NoPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no payload found in packet")
    }
}