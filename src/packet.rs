//! Holds all the information regarding a given packet from the transport stream.
use std::error::Error;
use bitvec::macros::internal::funty::Fundamental;
use log::trace;
use bitvec::prelude::*;
use crate::errors::InvalidFirstByteError;

/// All transport stream packets start with a SYNC byte.
const SYNC_BYTE: u8 = 0x47;

/// All of this information is shamelessly stolen from wikipedia, my lord and savior.
/// This [article](https://en.wikipedia.org/wiki/MPEG_transport_stream) in particular. Please donate
/// to wikipedia if you have the means.
pub struct TSPacket {
    /// TEI: Transport error indicator is true when a packet is set when a demodulator cannot
    /// correct errors and indicates that the packet is corrupt.
    tei: bool,
    /// PUSI: Payload unit start indicator indicates if this packet contains the first byte of a
    /// payload since they can be spread across multiple packets.
    pusi: bool,
    /// Transport priority, set when the current packet is higher priority than other packets of the
    /// same PID
    transport_priority: bool,
    /// PID: Packet identifier of the transport stream packet. Describes what the payload data is.
    pid: BitVec,
    /// TSC: Transport scrambling control indicates whether the payload is encrypted and with what
    /// key. Valid values are:
    /// - `00` for no scrambling.
    /// - `01` is reserved.
    /// - `10` Scrambled with even key.
    /// - `11` Scrambled with odd key.
    tsc: BitVec,
    /// Adaptation field control describes if this packet contains adaptation field data,
    /// payload data, or both. Valid values are:
    /// - `00` is reserved.
    /// - `01` for payload only.
    /// - `10` for adaptation field only.
    /// - `11` for adaptation field followed by payload.
    adaptation_field_control: BitVec,
    /// Continuity counter is used for determining the sequence of data in each PID.
    continuity_counter: BitVec,
    /// Adaptation field data. This field will be `None` when the adaptation field control field has
    /// a `0` in the MSB place.
    adaptation_field: Option<Box<[u8]>>,
    /// Payload field data. This field will be `None` when the adaptation field control field has
    /// a `0` in the LSB place.
    payload_data: Option<Box<[u8]>>,

}

impl TSPacket {

    /// Create a TSPacket from a byte array.
    pub fn from_bytes(buf: &mut [u8]) -> Result<TSPacket, Box<dyn Error>> {
        // Check if the first byte is SYNC byte.
        if buf[0] != SYNC_BYTE {
            return Err(Box::new(InvalidFirstByteError { byte: buf[0] }));
        }

        trace!("Parsing TSPacket from raw bytes: [{:#?}]", buf);
        Ok (TSPacket {
            tei: false,
            pusi: false,
            transport_priority: false,
            pid: BitVec::new(),
            tsc: BitVec::new(),
            adaptation_field_control: BitVec::new(),
            continuity_counter: BitVec::new(),
            adaptation_field: None,
            payload_data: None,
        })

    }

    /// Return if the transport error indicator is set.
    pub fn tei(&self) -> bool {
        self.tei
    }

    /// Return if the payload unit start indicator is set.
    pub fn pusi(&self) -> bool {
        self.pusi
    }

    /// Return if the transport priority is set.
    pub fn transport_priority(&self) -> bool {
        self.transport_priority
    }

    /// Returns the packet identifier.
    pub fn pid(&self) -> u8 {
        self.pid.load()
    }

    /// Return's the packet identifier.
    pub fn tsc_odd(&self) -> u8 {
        self.tsc.load()
    }

    /// Adaptation field control value.
    pub fn adaptation_field_control(&self) -> u8 {
        self.continuity_counter.load()
    }

    /// Returns if the packet has adaptation field data.
    pub fn has_adaptation_field(&self) -> bool {
        self.adaptation_field_control.get(0).unwrap().as_bool()
    }

    /// Returns if the packet has payload field data.
    pub fn has_payload(&self) -> bool {
        self.adaptation_field_control.get(1).unwrap().as_bool()
    }

    /// Returns the continuity counter.
    pub fn continuity_counter(&self) -> u8 {
        self.continuity_counter.load()
    }

    /// Return the adaptation field data.
    pub fn adaptation_field(&self) -> Option<Box<[u8]>> {
        self.adaptation_field.clone()
    }

    /// Return the payload data
    pub fn payload(&self) -> Option<Box<[u8]>> {
        self.payload_data.clone()
    }
}