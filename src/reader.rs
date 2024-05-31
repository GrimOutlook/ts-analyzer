//! A module for reading the transport stream.

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use crate::packet::TSPacket;

/// Struct used for holding information related to reading the transport stream.
pub struct TSReader {
    /// Buffered reader for the transport stream file.
    buf_reader: BufReader<File>,
}

impl TSReader {

    /// Create a new TSReader instance using the given file.
    pub fn new(buf_reader: BufReader<File>) -> Self {
        TSReader {
            buf_reader,
        }
    }

    /// Read the next packet from the transport stream file.
    /// # Returns
    /// `Ok(TSPacket)` if a transport stream packet could be parsed from the bytes.
    pub fn read_next_packet(&mut self) -> Result<TSPacket, Box<dyn Error>> {
        let mut buf = [0; 10];
        let _ = self.buf_reader.read(&mut buf);
        TSPacket::from_bytes(&mut buf)
    }
}