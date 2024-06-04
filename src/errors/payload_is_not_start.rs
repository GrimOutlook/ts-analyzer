//! Error that is when trying to read the start of a new payload when no new payload is present in
//! a packet.
use std::fmt;

/// Error that is when trying to read the start of a new payload when no new payload is present in
/// a packet.
#[derive(Debug, Clone)]
pub struct PayloadIsNotStart;

impl std::error::Error for PayloadIsNotStart {}

impl fmt::Display for PayloadIsNotStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "payload does not contain the start of a new partial payload")
    }
}