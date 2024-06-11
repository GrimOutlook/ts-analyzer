//! Holds all the information regarding a given packet from the transport stream.
mod adaptation_extension;
mod header;
mod adaptation_field;

use crate::errors::invalid_first_byte::InvalidFirstByte;
use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use std::error::Error;

#[cfg(feature = "log")]
use log::trace;
use crate::packet::adaptation_field::TSAdaptationField;
use crate::AdaptationFieldControl::{Payload, AdaptationAndPayload, AdaptationField};
use crate::packet::header::TSHeader;

/// All transport stream packets start with a SYNC byte.
pub const SYNC_BYTE: u8 = 0x47;

/// All of this information is shamelessly stolen from wikipedia, my lord and savior.
/// This [article](https://en.wikipedia.org/wiki/MPEG_transport_stream) in particular. Please donate
/// to wikipedia if you have the means.
pub struct TSPacket {
    /// Header object which tracks header attributes of the packet
    header: TSHeader,
    /// Adaptation field data. This field will be `None` when the adaptation field control field has
    /// a `0` in the MSB place.
    adaptation_field: Option<TSAdaptationField>,
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

        #[cfg(feature = "log")]
        trace!("Parsing TSPacket from raw bytes: [{:#?}]", buf);

        let header_bytes: BitVec<u8, Msb0> = BitVec::from_slice(&buf[1..=3]);

        // Get some of the necessary information for further processing as some of this data is
        // dynamic.
        let adaptation_field_control = header_bytes[18..=19].to_bitvec();
        let adaptation_field_exists = adaptation_field_control.get(0).unwrap().as_bool();
        let payload_exists = adaptation_field_control.get(1).unwrap().as_bool();

        let header = TSHeader::new(
            header_bytes.get(0).unwrap().as_bool(),
            header_bytes.get(1).unwrap().as_bool(),
            header_bytes.get(2).unwrap().as_bool(),
            header_bytes[3..=15].load(),
            header_bytes[16..=17].to_bitvec().load(),
            header_bytes[18..=19].to_bitvec().load(),
            header_bytes[20..=23].load(),
        );

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
            let adaptation_field_len: u8 = BitVec::<u8, Msb0>::from_slice(
                &buf[adaptation_field_start..adaptation_field_start + 1],
            )
            .load();

            // Check if any of the dynamic fields are set. If these pop during testing I'll have to
            // implement them, but otherwise I'll leave them until necessary.
            let adaptation_field_required: BitVec<u8, Msb0> =
                BitVec::from_slice(&buf[adaptation_field_start + 1..adaptation_field_start + 2]);
            let pcr_flag = adaptation_field_required.get(3).unwrap().as_bool();
            let opcr_flag = adaptation_field_required.get(4).unwrap().as_bool();
            let transport_private_data_flag = adaptation_field_required.get(6).unwrap().as_bool();
            let adaptation_field_extension_flag =
                adaptation_field_required.get(7).unwrap().as_bool();
        }

        let packet = TSPacket {
            header,
            adaptation_field: None,
            payload_data: None,
        };

        Ok(packet)
    }

    /// Returns the header object of this packet
    pub fn header(&self) -> TSHeader {
        self.header.clone()
    }

    /// Returns if the packet has adaptation field data.
    pub fn has_adaptation_field(&self) -> bool {
        match self.header().adaptation_field_control() {
            AdaptationField | AdaptationAndPayload => true,
            _ => false
        }
    }

    /// Returns if the packet has payload field data.
    pub fn has_payload(&self) -> bool {
        match self.header().adaptation_field_control() {
            Payload | AdaptationAndPayload => true,
            _ => false
        }
    }

    /// Return the adaptation field data.
    pub fn adaptation_field(&self) -> Option<TSAdaptationField> {
        self.adaptation_field.clone()
    }

    /// Return the payload data
    pub fn payload(&self) -> Option<Box<[u8]>> {
        self.payload_data.clone()
    }
}
