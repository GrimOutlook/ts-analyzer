//! A module for reading the transport stream.
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read, Seek, SeekFrom};
use crate::errors::no_sync_byte_found::NoSyncByteFound;
use crate::packet::{TSPacket, PACKET_SIZE};
use crate::packet::header::SYNC_BYTE;
use crate::packet::payload::TSPayload;
use crate::helpers::tracked_payload::{self, TrackedPayload};

#[cfg(feature = "log")]
use log::{info,debug,trace};
use memmem::{Searcher, TwoWaySearcher};

/// Struct used for holding information related to reading the transport stream.
pub struct TSReader {
    /// Filename for the file being read. Only really used for logging.
    filename: String,
    /// Buffered reader for the transport stream file.
    buf_reader: BufReader<File>,
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

impl TSReader {

    /// Create a new TSReader instance using the given file.
    ///
    /// This function also finds the first SYNC byte, so we can determine the alignment of the
    /// transport packets.
    /// # Parameters
    /// - `buf_reader`: a buffered reader that contains transport stream data.
    pub fn new(filename: &str, mut buf_reader: BufReader<File>) -> Result<Self, Box<dyn Error>> {
        // Find the first sync byte, so we can search easier by doing simple `PACKET_SIZE` buffer
        // reads.
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
            let sync_pos = buf_reader.stream_position().expect("Couldn't get stream position from BufReader");

            #[cfg(feature = "log")]
            trace!("SYNC found at position {} for file {}", sync_pos, filename);

            // If we think this is the correct alignment because we have found a SYNC byte we need
            // to verify that this is correct by seeking 1 `PACKET_SIZE` away and verifying a SYNC
            // byte is there. If there isn't one there then this is simply the same data as a SYNC
            // byte by coincidence, and we need to keep looking.
            //
            // There is always the possibility that we hit a `0x47` in the payload, seek 1
            // `PACKET_SIZE` further, and find another `0x47` but I don't have a way of accounting
            // for that, so we're going with blind hope that this case doesn't get seen.
            buf_reader.seek_relative(PACKET_SIZE as i64 - 1)?;
            let count = buf_reader.read(&mut read_buf)?;

            // If we run out of data to read while trying to verify that the SYNC byte is actually a
            // SYNC byte and isn't part of a payload then we'll simply assume that it really is a
            // SYNC byte as we have nothing else to go off of.
            if count == 0 {
                #[cfg(feature = "log")]
                debug!("Could not find SYNC byte in file {}", filename);
                return Err(Box::new(NoSyncByteFound));
            }

            // Seek back to the original location for later reading.
            buf_reader.seek(SeekFrom::Start(sync_pos - 1))?;

            // If the byte 1 `PACKET_SIZE` away is also a SYNC byte we can be relatively sure that
            // this alignment is correct.
            if read_buf[0] == SYNC_BYTE {
                sync_alignment = sync_pos;
                break
            }
        }

        Ok(TSReader {
            filename: filename.to_string(),
            buf_reader,
            sync_alignment,
            packets_read: 0,
            tracked_pids: Vec::new(),
            tracked_payloads: Vec::new(),
        })
    }

    /// Read the next packet from the transport stream file.
    ///
    /// This function returns `None` for any `Err` in order to prevent the need for `.unwrap()`
    /// calls in more concise code.
    /// # Returns
    /// `Some(TSPacket)` if the next transport stream packet could be parsed from the file.
    /// `None` if the next transport stream packet could not be parsed from the file for any
    /// reason. This includes if the entire file has been fully read.
    pub fn next_packet_unchecked(&mut self) -> Option<TSPacket> {
        self.next_packet().unwrap_or_else(|_| None)
    }

    /// Read the next packet from the transport stream file.
    /// # Returns
    /// `Ok(Some(TSPacket))` if the next transport stream packet could be parsed from the file.
    /// `Ok(None)` if there was no issue reading the file and no more TS packets can be read.
    pub fn next_packet(&mut self) -> Result<Option<TSPacket>, Box<dyn Error>> {
        let mut packet_buf = [0; PACKET_SIZE];
        loop {
            match self.buf_reader.read_exact(&mut packet_buf) {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        #[cfg(feature = "log")]
                        {
                            info!("Finished reading file {}", self.filename);
                        }
                        return Ok(None);
                    }

                    return Err(Box::new(e));
                },
            }

            #[cfg(feature = "log")]
            {
                if let Ok(position) = self.buf_reader.stream_position() {
                    trace!("Seek position in file {}: {}", self.filename, position)
                }
            }

            self.packets_read += 1;
            #[cfg(feature = "log")]
            trace!("Packets read in file {}: {}", self.filename, self.packets_read);

            let packet = match TSPacket::from_bytes(&mut packet_buf) {
                Ok(packet) => packet,
                Err(e) => {
                    #[cfg(feature = "log")]
                    debug!("Got error from {} when trying to parse next packet from bytes {:2X?}",
                        self.filename, packet_buf);
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

    /// Read the next payload from the transport stream file.
    ///
    /// This function returns `None` for any `Err` in order to prevent the need for `.unwrap()`
    /// calls in more concise code.
    /// # Returns
    /// `Some(TSPayload)` if the next transport stream packet could be parsed from the file.
    /// `None` if the next transport stream payload could not be parsed from the file for any
    /// reason. This includes if the entire file has been fully read.
    pub fn next_payload_unchecked(&mut self) -> Option<Box<[u8]>> {
        self.next_payload().unwrap_or_else(|_| None)
    }

    /// Read the next full payload from the file.
    ///
    /// This function parses through all transport stream packets, stores them in a buffer and
    /// concatenates their payloads together once a payload has been complete.
    pub fn next_payload(&mut self) -> Result<Option<Box<[u8]>>, Box<dyn Error>> {
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
        let payload = packet.payload()?;

        // Check to see if we already have an TrackedPayload object for this item PID
        let pid = packet.header().pid();
        
        if let Some(index) = self.tracked_payloads.iter().position(|tp| tp.pid() == pid) {
            let tracked_payload = &mut self.tracked_payloads[index];
            return tracked_payload.add_and_get_complete(&payload);
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