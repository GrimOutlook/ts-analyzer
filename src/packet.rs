//! Holds all the information regarding a given packet from the transport stream.
pub mod payload;
pub mod header;
pub mod adaptation_field;

use crate::{packet::adaptation_field::DataAdaptationField, TSError};
use crate::packet::header::TSHeader;
use adaptation_field::{AdaptationField, StuffingAdaptationField};
use bitvec::prelude::*;
use std::error::Error;

use crate::packet::payload::TSPayload;
#[cfg(feature = "log")]
use log::trace;

pub(crate) const PACKET_SIZE: usize = 188;

/// The length of a transport stream packet is 4 bytes in size.
pub const HEADER_SIZE: u8 = 4;

/// All of this information is shamelessly stolen from wikipedia, my lord and savior.
/// This [article](https://en.wikipedia.org/wiki/MPEG_transport_stream) in particular. Please donate
/// to wikipedia if you have the means.
pub struct TSPacket {
    /// Header object which tracks header attributes of the packet
    header: TSHeader,
    /// Adaptation field data. This field will be `None` when the adaptation field control field has
    /// a `0` in the MSB place.
    adaptation_field: Option<AdaptationField>,
    /// Payload field data. This field will be `None` when the adaptation field control field has
    /// a `0` in the LSB place.
    payload: Option<TSPayload>,
}

impl TSPacket {
    /// Create a TSPacket from a byte array.
    pub fn from_bytes(buf: &mut [u8]) -> Result<TSPacket, TSError> {
        let buffer_length = buf.len();
        let header_bytes = Box::from(buf[0..HEADER_SIZE as usize].to_vec());

        #[cfg(feature = "log")]
        trace!("Parsing TSPacket from raw bytes: {:02X?}", buf);

        let header = match TSHeader::from_bytes(&header_bytes) {
            Ok(header) => header,
            Err(e) => return Err(e),
        };

        #[cfg(feature = "log")]
        trace!("Header for TSPacket: {}", header);

        // This number comes from the fact that the TS header is always 4 bytes wide and the
        // adaptation field always comes directly after the header if it is present.
        let mut read_idx: usize = 4;

        // If the adaptation field is present we need to determine it's size as we want to ignore
        // it entirely.
        let adaptation_field = if header.has_adaptation_field() {
            #[cfg(feature = "log")]
            trace!("Adaptation field exists for TSPacket");

            // Get the length of the adaptation field. If it's `0` then this is a stuffing
            // adaptation field.
            let length = buf[read_idx];

            if length != 0 {
                let af = DataAdaptationField::from_bytes(&mut buf[read_idx..buffer_length]);

                read_idx += af.adaptation_field_length() as usize;
    
                // We currently do nothing with the adaptation extension field.
                // TODO: Add support for adaptation extension field.
                #[cfg(feature = "log")]
                trace!("Packet has adaptation extension field {}", header);
    
                Some(AdaptationField::Data(af))
            } else {
                let af = StuffingAdaptationField::new();

                read_idx += af.adaptation_field_length() as usize;

                Some(AdaptationField::Stuffing(af))
            }
        } else {
            None
        };

        let payload = if header.has_payload() {
            #[cfg(feature = "log")]
            trace!("Payload exists for TSPacket");

            let payload_bytes: Box<[u8]> = Box::from(
                BitVec::<u8, Msb0>::from_slice(&buf[read_idx..buf.len()]).as_raw_slice()
            );

            let remainder = (PACKET_SIZE - read_idx) as u8;
            if header.pusi() && payload_bytes[0] > remainder {
                return Err(TSError::InvalidPayloadPointer(payload_bytes[0], remainder))
            }

            Some(TSPayload::from_bytes(header.pusi(), header.continuity_counter(), payload_bytes))
        } else {
            None
        };

        // Payload data should now start at the read_idx.
        let packet = TSPacket {
            header,
            adaptation_field,
            payload,
        };

        Ok(packet)
    }

    /// Returns the header object of this packet
    pub fn header(&self) -> TSHeader {
        self.header.clone()
    }

    /// Returns if the packet has adaptation field data.
    pub fn has_adaptation_field(&self) -> bool {
        self.header.has_adaptation_field()
    }

    /// Returns if the packet has payload field data.
    pub fn has_payload(&self) -> bool {
        self.header.has_payload()
    }

    /// Return the adaptation field data.
    pub fn adaptation_field(&self) -> Option<AdaptationField> {
        self.adaptation_field.clone()
    }

    /// Return the payload data
    pub fn payload(&self) -> Option<TSPayload> {
        self.payload.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::AdaptationFieldControl;

    use super::*;
    use test_case::test_case;

    // The original error I got from this packet was: `range end index 224 out of range for slice of
    // length 24`. Want to keep it as a historical test case.
    fn packet_1() -> (Box<[u8]>, AdaptationFieldControl) {
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
        return (Box::new(packet), crate::AdaptationFieldControl::AdaptationAndPayload);
    }

    #[test_case(packet_1)]
    fn from_bytes(p: fn() -> (Box<[u8]>, AdaptationFieldControl)) {
        let (mut buf, adaptation_field_control) = p();
        let packet = TSPacket::from_bytes(&mut buf).unwrap();
        assert_eq!(packet.header().adaptation_field_control(), adaptation_field_control, "Transport Error Indicator is incorrect");
    }
}