//! A module for reading the transport stream.

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use crate::errors::no_sync_byte_found::NoSyncByteFound;
use crate::packet::{SYNC_BYTE, TSPacket};

const PACKET_SIZE: usize = 188;

/// Struct used for holding information related to reading the transport stream.
pub struct TSReader {
    /// Buffered reader for the transport stream file.
    buf_reader: BufReader<File>,
    /// Sync byte alignment. A Sync byte should be found every `PACKET_SIZE` away.
    sync_alignment: u64,
}

impl TSReader {

    /// Create a new TSReader instance using the given file.
    ///
    /// This function also finds the first SYNC byte, so we can determine the alignment of the
    /// transport packets.
    pub fn new(mut buf_reader: BufReader<File>) -> Result<Self, Box<dyn Error>> {
        // Find the first sync byte, so we can search easier by doing simple `PACKET_SIZE` buffer
        // reads
        let mut read_buf = [0];
        let sync_alignment: u64;

        loop {
            let count = buf_reader.read(&mut read_buf)?;

            // Return a `NoSyncByteFound` error if no SYNC byte could be found in the reader.
            if count == 0 {
                return Err(Box::new(NoSyncByteFound));
            }

            // Run through this loop until we find a sync byte.
            if read_buf[0] != SYNC_BYTE {
                continue
            }

            // Note the location of this SYNC byte for later
            let sync_pos = buf_reader.stream_position().unwrap();

            // If we think this is the correct alignment because we have found a SYNC byte we need
            // to verify that this is correct by seeking 1 `PACKET_SIZE` away and verifying a SYNC
            // byte is there. If there isn't one there then this is simply the same data as a SYNC
            // byte by coincidence, and we need to keep looking.
            //
            // There is always the possibility that we hit a `0x47` in the payload, seek 1
            // `PACKET_SIZE` further, and find another `0x47` but I don't have a way of accounting
            // for that, so we're going with blind hope that this case doesn't get seen.
            buf_reader.seek_relative(PACKET_SIZE as i64)?;
            let count = buf_reader.read(&mut read_buf)?;

            // If we run out of data to read while trying to verify that the SYNC byte is actually a
            // SYNC byte and isn't part of a payload then we'll simply assume that it really is a
            // SYNC byte as we have nothing else to go off of.
            if count == 0 {
                return Err(Box::new(NoSyncByteFound));
            }

            // Seek back to the original location for later reading.
            buf_reader.seek(SeekFrom::Start(sync_pos))?;

            // If the byte 1 `PACKET_SIZE` away is also a SYNC byte we can be relatively sure that
            // this alignment is correct.
            if read_buf[0] == SYNC_BYTE {
                sync_alignment = sync_pos;
                break
            }
        }

        Ok(TSReader {
            buf_reader,
            sync_alignment,
        })
    }

    /// Read the next packet from the transport stream file.
    /// # Returns
    /// `Ok(Some(TSPacket))` if a transport stream packet could be parsed from the bytes.
    /// `Ok(None)` if there was no issue reading the file and no more TS packets can be read.
    pub fn read_next_packet(&mut self) -> Result<Option<TSPacket>, Box<dyn Error>> {
        let mut packet_buf = [0; PACKET_SIZE];
        let count = self.buf_reader.read(&mut packet_buf)?;

        if count < PACKET_SIZE {
            return Ok(None);
        }

        match TSPacket::from_bytes(&mut packet_buf) {
            Ok(packet) => Ok(Some(packet)),
            Err(e) => Err(e),
        }
    }

    /// Return the alignment of the SYNC bytes in this reader.
    pub fn sync_byte_alignment(&self) -> u64 {
        self.sync_alignment
    }
}