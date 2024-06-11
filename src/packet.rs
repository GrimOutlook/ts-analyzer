//! Holds all the information regarding a given packet from the transport stream.
use std::error::Error;
use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use crate::errors::invalid_first_byte::InvalidFirstByte;

#[cfg(feature="log")]
use log::trace;

/// All transport stream packets start with a SYNC byte.
pub const SYNC_BYTE: u8 = 0x47;

/// All of this information is shamelessly stolen from wikipedia, my lord and savior.
/// This [article](https://en.wikipedia.org/wiki/MPEG_transport_stream) in particular. Please donate
/// to wikipedia if you have the means.
pub struct TSPacket {
    /// TEI: Transport error indicator is true when a packet is set when a demodulator cannot
    /// correct invalid_first_byte and indicates that the packet is corrupt.
    tei: bool,
    /// PUSI: Payload unit start indicator indicates if this packet contains the first byte of a
    /// payload since they can be spread across multiple packets.
    pusi: bool,
    /// Transport priority, set when the current packet is higher priority than other packets of the
    /// same PID
    transport_priority: bool,
    /// PID: Packet identifier of the transport stream packet. Describes what the payload data is.
    pid: u16,
    /// TSC: Transport scrambling control indicates whether the payload is encrypted and with what
    /// key. Valid values are:
    /// - `00` for no scrambling.
    /// - `01` is reserved.
    /// - `10` Scrambled with even key.
    /// - `11` Scrambled with odd key.
    tsc: BitVec<u8, Msb0>,
    /// Adaptation field control describes if this packet contains adaptation field data,
    /// payload data, or both. Valid values are:
    /// - `00` is reserved.
    /// - `01` for payload only.
    /// - `10` for adaptation field only.
    /// - `11` for adaptation field followed by payload.
    adaptation_field_control: BitVec<u8, Msb0>,
    /// Continuity counter is used for determining the sequence of data in each PID.
    continuity_counter: u8,
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
            return Err(Box::new(InvalidFirstByte { byte: buf[0] }));
        }

        #[cfg(feature="log")]
        trace!("Parsing TSPacket from raw bytes: [{:#?}]", buf);

        let header: BitVec<u8, Msb0> = BitVec::from_slice(&buf[1..=3]);

        // Get some of the necessary information for further processing as some of this data is
        // dynamic.
        let adaptation_field_control = header[18..=19].to_bitvec();
        let adaptation_field_exists = adaptation_field_control.get(0).unwrap().as_bool();
        let payload_exists = adaptation_field_control.get(1).unwrap().as_bool();

        // If the adaptation field is present we need to determine it's size as we want to ignore
        // it entirely.
        if adaptation_field_exists {
            // This number comes from the fact that the TS header is always 4 bytes wide and the
            // adaptation field always comes directly after the header if it is present.
            let adaptation_field_start = 4;

            // Get the length of the adaptation field.
            //
            // TODO: Determine if this takes into account the `Transport private data length` field
            // or not. If it doesn't then that field will need to be parsed as well. For the current
            // moment I'm assuming it takes it into account
            let adaptation_field_len: u8 = BitVec::<u8, Msb0>::from_slice(&buf[adaptation_field_start..adaptation_field_start+1]).load();

            // Check if any of the dynamic fields are set. If these pop during testing I'll have to
            // implement them, but otherwise I'll leave them until necessary.
            let adaptation_field_required: BitVec<u8, Msb0> = BitVec::from_slice(&buf[adaptation_field_start+1..adaptation_field_start+2]);
            let pcr_flag = adaptation_field_required.get(3).unwrap().as_bool();
            let opcr_flag = adaptation_field_required.get(4).unwrap().as_bool();
            let transport_private_data_flag = adaptation_field_required.get(6).unwrap().as_bool();
            let adaptation_field_extension_flag = adaptation_field_required.get(7).unwrap().as_bool();

            if pcr_flag || opcr_flag || transport_private_data_flag || adaptation_field_extension_flag {
                todo!("Implement dynamic adaptation field sizes");
            }
        }

        let packet = TSPacket {
            tei: header.get(0).unwrap().as_bool(),
            pusi: header.get(1).unwrap().as_bool(),
            transport_priority: header.get(2).unwrap().as_bool(),
            pid: header[3..=15].load(),
            tsc: header[16..=17].to_bitvec(),
            adaptation_field_control: header[18..=19].to_bitvec(),
            continuity_counter: header[20..=23].load(),
            adaptation_field: None,
            payload_data: None,
        };

        Ok (packet)
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
    pub fn pid(&self) -> u16 {
        self.pid
    }

    /// Return's the packet identifier.
    pub fn tsc_odd(&self) -> u8 {
        self.tsc.load()
    }

    /// Adaptation field control value.
    pub fn adaptation_field_control(&self) -> u8 {
        self.adaptation_field_control.load()
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
        self.continuity_counter
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