//! Error that is thrown when trying to read a transport stream file and no SYNC byte can be found.
use std::fmt;

/// Error that is thrown when trying to read a transport stream file and no SYNC byte can be found.
#[derive(Debug, Clone)]
pub struct NoSyncByteFound;

impl std::error::Error for NoSyncByteFound {}

impl fmt::Display for NoSyncByteFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no sync byte found in reader")
    }
}