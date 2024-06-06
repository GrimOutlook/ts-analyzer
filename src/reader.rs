//! A module for reading the transport stream.
use std::io::{BufReader, ErrorKind, Read, Seek, SeekFrom};
use crate::packet::{TSPacket, PACKET_SIZE};
use crate::packet::header::SYNC_BYTE;
use crate::helpers::tracked_payload::TrackedPayload;
use crate::TSError;

#[cfg(feature = "log")]
use log::{info,debug,trace};

/// Struct used for holding information related to reading the transport stream.
pub struct TSReader<R: Read + Seek> {
    /// Buffered reader for the transport stream file.
    buf_reader: BufReader<R>,
    /// Sync byte alignment. A Sync byte should be found every `PACKET_SIZE` away.
    sync_alignment: u64,
    /// Counter of the number of packets read
    packets_read: u64,
    /// PIDs that should be tracked when querying for packets or payloads.
    /// 
    /// If empty, all PIDs are tracked. This will use more memory as there are more
    /// incomplete payloads to keep track of.
    tracked_pids: Vec<u16>,
    /// Payloads that are currently being tracked by the reader.
    tracked_payloads: Vec<TrackedPayload>,
}

impl<R> TSReader<R> where R: Read + Seek{

    /// Create a new TSReader instance using the given file.
    ///
    /// This function also finds the first SYNC byte, so we can determine the alignment of the
    /// transport packets.
    /// # Parameters
    /// - `buf_reader`: a buffered reader that contains transport stream data.
    pub fn new(mut buf_reader: BufReader<R>) -> Result<Self, TSError> {
        // Find the first sync byte, so we can search easier by doing simple `PACKET_SIZE` buffer
        // reads.
        let mut read_buf = [0];
        let sync_alignment: u64;

        loop {
            let count = match buf_reader.read(&mut read_buf) {
                Ok(count) => count,
                Err(e) => return Err(TSError::ReaderError(e))
            };

            // Return a `NoSyncByte` error if no SYNC byte could be found in the reader.
            if count == 0 {
                #[cfg(feature = "log")]
                trace!("No data read from reader. SYNC byte could not be found.");
                return Err(TSError::NoSyncByte);
            }

            // Run through this loop until we find a sync byte.
            if read_buf[0] != SYNC_BYTE {
                continue
            }

            // Note the location of this SYNC byte for later
            let sync_pos = buf_reader.stream_position().expect("Couldn't get stream position from BufReader");

            #[cfg(feature = "log")]
            trace!("SYNC found at position {} for reader", sync_pos);

            // If we think this is the correct alignment because we have found a SYNC byte we need
            // to verify that this is correct by seeking 1 `PACKET_SIZE` away and verifying a SYNC
            // byte is there. If there isn't one there then this is simply the same data as a SYNC
            // byte by coincidence, and we need to keep looking.
            //
            // There is always the possibility that we hit a `0x47` in the payload, seek 1
            // `PACKET_SIZE` further, and find another `0x47` but I don't have a way of accounting
            // for that, so we're going with blind hope that this case doesn't get seen.
            match buf_reader.seek_relative(PACKET_SIZE as i64 - 1) {
                Ok(_) => (),
                Err(e) => return Err(TSError::ReaderError(e))
            };
            let count = match buf_reader.read(&mut read_buf){
                Ok(count) => count,
                Err(e) => return Err(TSError::ReaderError(e))
            };

            // If we run out of data to read while trying to verify that the SYNC byte is actually a
            // SYNC byte and isn't part of a payload then we'll simply assume that it really is a
            // SYNC byte as we have nothing else to go off of.
            if count != 0 {
                // If the byte 1 `PACKET_SIZE` away is also a SYNC byte we can be relatively sure that
                // this alignment is correct.
                if read_buf[0] != SYNC_BYTE {
                    continue
                }
            }

            // Seek back to the original location for later reading.
            match buf_reader.seek(SeekFrom::Start(sync_pos - 1)) {
                Ok(_) => (),
                Err(e) => return Err(TSError::ReaderError(e))
            };
            
            sync_alignment = sync_pos;
            break
        }

        Ok(TSReader {
            buf_reader,
            sync_alignment,
            packets_read: 0,
            tracked_pids: Vec::new(),
            tracked_payloads: Vec::new(),
        })
    }

    /// Read the next packet from the transport stream reader.
    ///
    /// This function returns `None` for any `Err` in order to prevent the need for `.unwrap()`
    /// calls in more concise code.
    /// # Returns
    /// `Some(TSPacket)` if the next transport stream packet could be parsed from the reader.
    /// `None` if the next transport stream packet could not be parsed from the reader for any
    /// reason. This includes if the entire reader has been fully read.
    pub fn next_packet_unchecked(&mut self) -> Option<TSPacket> {
        self.next_packet().unwrap_or_else(|_| None)
    }

    /// Read the next packet from the transport stream reader.
    /// # Returns
    /// `Ok(Some(TSPacket))` if the next transport stream packet could be parsed from the reader.
    /// `Ok(None)` if there was no issue reading the reader and no more TS packets can be read.
    pub fn next_packet(&mut self) -> Result<Option<TSPacket>, TSError> {
        let mut packet_buf = [0; PACKET_SIZE];
        loop {
            match self.buf_reader.read_exact(&mut packet_buf) {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        #[cfg(feature = "log")]
                        {
                            info!("Finished reading all data from reader");
                        }
                        return Ok(None);
                    }

                    return Err(TSError::ReaderError(e));
                },
            }

            #[cfg(feature = "log")]
            {
                if let Ok(position) = self.buf_reader.stream_position() {
                    trace!("Seek position in reader: {}", position)
                }
            }

            self.packets_read += 1;
            #[cfg(feature = "log")]
            trace!("Packets read from reader: {}", self.packets_read);

            let packet = match TSPacket::from_bytes(&mut packet_buf) {
                Ok(packet) => packet,
                Err(e) => {
                    #[cfg(feature = "log")]
                    debug!("Got error when trying to parse next packet from bytes {:2X?}", packet_buf);
                    return Err(e)
                },
            };

            // We should only return a packet if it is in the tracked PIDs (or there are no tracked
            // PIDs)
            if ! self.tracked_pids.is_empty() && ! self.tracked_pids.contains(&packet.header().pid()) {
                continue
            }

            return Ok(Some(packet));
        }
    }

    /// Read the next payload from the transport stream reader.
    ///
    /// This function returns `None` for any `Err` in order to prevent the need for `.unwrap()`
    /// calls in more concise code.
    /// # Returns
    /// `Some(TSPayload)` if the next transport stream packet could be parsed from the reader.
    /// `None` if the next transport stream payload could not be parsed from the reader for any
    /// reason. This includes if the entire reader has been fully read.
    pub fn next_payload_unchecked(&mut self) -> Option<Box<[u8]>> {
        self.next_payload().unwrap_or_else(|_| None)
    }

    /// Read the next full payload from the reader.
    ///
    /// This function parses through all transport stream packets, stores them in a buffer and
    /// concatenates their payloads together once a payload has been complete.
    pub fn next_payload(&mut self) -> Result<Option<Box<[u8]>>, TSError> {
        loop {
            let possible_packet = match self.next_packet() {
                Ok(packet) => packet,
                Err(e) => return Err(e),
            };
            
            let Some(packet) = possible_packet else {
                return Ok(None);
            };

            // Add this packet's payload to the tracked payload and retrieve the completed payload
            // if it exists.
            let payload = self.add_tracked_payload(&packet);
            if payload.is_some() {
                return Ok(payload)
            }
        }
    }

    /// Return the alignment of the SYNC bytes in this reader.
    pub fn sync_byte_alignment(&self) -> u64 {
        self.sync_alignment
    }

    /// Add a PID to the tracking list.
    ///
    /// Only tracked PIDs are returned when running methods that gather packets or payloads. If no
    /// PID is specified then all PIDs are returned.
    pub fn add_tracked_pid(&mut self, pid: u16) {
        self.tracked_pids.push(pid);
    }

    /// Remove this PID from being tracked.
    ///
    /// Only tracked PIDs are returned when running methods that gather packets or payloads. If no
    /// PID is specified then all PIDs are returned.
    pub fn remove_tracked_pid(&mut self, pid: u16) {
        self.tracked_pids.retain(|vec_pid| *vec_pid != pid);
    }

    /// Add payload data from a packet to the tracked payloads list.
    fn add_tracked_payload(&mut self, packet: &TSPacket) -> Option<Box<[u8]>> {
        let payload = match packet.payload() {
            Some(payload) => payload,
            None => return None
        };

        // Check to see if we already have an TrackedPayload object for this item PID
        let pid = packet.header().pid();
        match self.tracked_payloads.iter().position(|tp| tp.pid() == pid) {
            Some(index) => {
                let tracked_payload = &mut self.tracked_payloads[index];
                return tracked_payload.add_and_get_complete(&payload);
            }
            None => ()
        }

        // We cannot possibly know that a payload is complete from the first packet. In order to
        // know that a payload is fully contained in 1 packet we need to see the `PUSI` flag set in
        // the next packet so there is no reason to check if the packet is complete when creating a
        // new TrackedPayload.

        match TrackedPayload::from_packet(packet) {
            Ok(tp) => {
                self.tracked_payloads.push(tp);
            }
            Err(_) => {}
        };

        return None;
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use test_case::test_case;

    // The original error I got from this packet was: `range end index 224 out of range for slice of
    // length 24`. Want to keep it as a historical test case as it should be parsed correctly.
    fn packet_reader_1() -> BufReader<Cursor<Box<[u8]>>> {
        let packet = [
            0x47, 0x01, 0x01, 0x3C, 0x00, 0x77, 0xE5, 0x90, 0x91, 0xC9, 0x60, 0x9E, 0x19, 0xD2,
            0x4A, 0x42, 0x50, 0x0E, 0x42, 0xDA, 0xED, 0xA4, 0x3E, 0xD8, 0x4F, 0xE5, 0x25, 0x24,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xBE, 0x8D, 0xCB, 0x97, 0x4B, 0x77, 0xBA, 0xBA,
            0xEC, 0xFC, 0xD8, 0x6C, 0xD2, 0x5A, 0x1F, 0x36, 0x86, 0xF7, 0x50, 0xEC, 0x78, 0xAA,
            0x45, 0xAE, 0xCF, 0x8E, 0x4D, 0x3E, 0x2E, 0x1D, 0xDD, 0x9B, 0xEE, 0xBA, 0x38, 0xB2,
            0x61, 0x56, 0x6E, 0xBD, 0xC5, 0x6D, 0x4C, 0xDD, 0x67, 0x57, 0x3E, 0x0E, 0x9B, 0x83,
            0x75, 0x34, 0xF6, 0x54, 0x85, 0xB6, 0xA2, 0x87, 0x5D, 0xB9, 0xBE, 0x1C, 0x1D, 0x9E,
            0xBD, 0x35, 0x3A, 0xFD, 0xBA, 0x63, 0xC8, 0xCC, 0xC3, 0x15, 0xE2, 0xDA, 0x96, 0xCE,
            0xA7, 0x6B, 0x05, 0xE5, 0x0D, 0x58, 0xE9, 0x21, 0xB3, 0x74, 0x6B, 0xD9, 0xD6, 0xBA,
            0x8B, 0x47, 0x45, 0x4A, 0x21, 0x53, 0x56, 0x92, 0xBF, 0x61, 0x7F, 0x91, 0x4E, 0x00,
            0x48, 0x14, 0xB1, 0xBA, 0x75, 0x10, 0x15, 0x9F, 0xB3, 0xD3, 0xD5, 0xBD, 0x90, 0x5A,
            0x7A, 0x7F, 0x2B, 0xC1, 0xF2, 0x5A, 0xFA, 0x49, 0x88, 0x08, 0x11, 0xE5, 0xC5, 0x67,
            0x18, 0x2A, 0x24, 0x2D, 0x60, 0xEB, 0x40, 0x28, 0xEC, 0x0A, 0x51, 0x0D, 0xA0, 0x55,
            0xC2, 0x70, 0xB0, 0x44, 0x00, 0x3F
        ];
        return BufReader::new(Cursor::new(Box::new(packet)));
    }

    #[test_case(packet_reader_1, 257, true; "PID successfully tracked")]
    #[test_case(packet_reader_1, 0, false; "PID unsuccessfully tracked")]
    fn from_bytes(p: fn() -> BufReader<Cursor<Box<[u8]>>>, pid: u16, tracked: bool) {
        let buf_reader = p();
        let mut ts= match TSReader::new(buf_reader) {
            Ok(ts) => ts,
            Err(e) =>
                panic!("Could not create reader TS reader for test due to error: {}", e)
        };
        
        // Add the PID we want to track for this test
        ts.add_tracked_pid(pid);

        // Get the next packet.
        let packet = ts.next_packet_unchecked();
        // Verify we get the value we expect.
        assert_eq!(packet.is_some(), tracked, "Packet tracking behavior was incorrect")
    }
}