//! Holds all the information regarding a given packet from the transport stream.
pub mod payload;
pub mod header;
pub mod adaptation_field;

use crate::errors::invalid_payload_pointer::InvalidPayloadPointer;
use crate::packet::adaptation_field::DataAdaptationField;
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
    pub fn from_bytes(buf: &mut [u8]) -> Result<TSPacket, Box<dyn Error>> {
        let buffer_length = buf.len();
        let header_bytes = Box::from(buf[0..HEADER_SIZE as usize].to_vec());

        #[cfg(feature = "log")]
        trace!("Parsing TSPacket from raw bytes: {:02X?}", buf);

        let header = match TSHeader::from_bytes(&header_bytes) {
            Ok(header) => header,
            Err(e) => return Err(e),
        };

        // This number comes from the fact that the TS header is always 4 bytes wide and the
        // adaptation field always comes directly after the header if it is present.
        let mut read_idx: usize = HEADER_SIZE as usize;

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

                // Add 1 because the adaptation field length is 1 byte long
                read_idx += af.adaptation_field_length() as usize + 1;
    
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
                return Err(Box::new(InvalidPayloadPointer { pointer: payload_bytes[0], remainder }))
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
    fn packet_1() -> (Box<[u8]>, AdaptationFieldControl, Box<[u8]>) {
        let packet = [
            0x47, 0x41, 0x02, 0x10, // Header
            0x00, 0x00, 0x01, 0xFC, 0x01, 0x10, 0x84, 0x80, 0x05, 0x21, 0x02, 0x95, 0x32, 0x6F,
            0x00, 0x00, 0xDF, 0x01, 0x03, 0x06, 0x0E, 0x2B, 0x34, 0x02, 0x0B, 0x01, 0x01, 0x0E,
            0x01, 0x03, 0x01, 0x01, 0x00, 0x00, 0x00, 0x81, 0xF1, 0x02, 0x08, 0x00, 0x04, 0xCA,
            0x14, 0x28, 0x06, 0x0E, 0xEA, 0x03, 0x15, 0x45, 0x53, 0x52, 0x49, 0x5F, 0x4D, 0x65,
            0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x5F, 0x43, 0x6F, 0x6C, 0x6C, 0x65, 0x63, 0x74,
            0x04, 0x06, 0x4E, 0x39, 0x37, 0x38, 0x32, 0x36, 0x05, 0x02, 0x70, 0x12, 0x06, 0x02,
            0x15, 0xB3, 0x07, 0x02, 0xEF, 0x62, 0x0A, 0x05, 0x43, 0x32, 0x30, 0x38, 0x42, 0x0B,
            0x00, 0x0C, 0x00, 0x0D, 0x04, 0x3A, 0x72, 0x80, 0x98, 0x0E, 0x04, 0xB5, 0x6C, 0xF5,
            0xB9, 0x0F, 0x02, 0x31, 0x4F, 0x10, 0x02, 0x04, 0x5C, 0x11, 0x02, 0x02, 0x73, 0x12,
            0x04, 0xB4, 0xCC, 0xCC, 0xCE, 0x13, 0x04, 0xF1, 0x81, 0x6C, 0x17, 0x14, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x15, 0x04, 0x00, 0x1E, 0x0A, 0x4F, 0x16, 0x02, 0x00, 0x00, 0x17,
            0x04, 0x3A, 0x76, 0x87, 0xAF, 0x18, 0x04, 0xB5, 0x70, 0x74, 0xF2, 0x19, 0x02, 0x23,
            0x99, 0x1A, 0x02, 0x01, 0x7B, 0x1B, 0x02, 0x00, 0x75, 0x1C, 0x02, 0xFF, 0xF1, 0x1D,
            0x02, 0x02 // Payload
        ];
        return (Box::new(packet), crate::AdaptationFieldControl::Payload, Box::new([0x00, 0x00, 0x01, 0xFC]));
    }

    fn packet_2() -> (Box<[u8]>, AdaptationFieldControl, Box<[u8]>) {
        let packet = [
            0x47, 0x01, 0x02, 0x31, // Header
            0x59, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Adaptation field
            0x03, 0x1E, 0x02, 0xFE, 0x9B, 0x1F, 0x02, 0xFF, 0x93, 0x20, 0x02, 0x00, 0x0F, 0x21, 
            0x02, 0xFE, 0x1B, 0x2F, 0x01, 0x00, 0x30, 0x2A, 0x01, 0x01, 0x01, 0x02, 0x01, 0x01, 
            0x03, 0x04, 0x2F, 0x2F, 0x43, 0x41, 0x04, 0x00, 0x05, 0x00, 0x06, 0x02, 0x43, 0x41, 
            0x15, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
            0x00, 0x00, 0x00, 0x00, 0x16, 0x02, 0x00, 0x05, 0x38, 0x01, 0x00, 0x3B, 0x08, 0x46, 
            0x69, 0x72, 0x65, 0x62, 0x69, 0x72, 0x64, 0x41, 0x01, 0x01, 0x48, 0x08, 0x00, 0x00, 
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0xF9, 0x05 // Payload
        ];

        return (Box::new(packet), crate::AdaptationFieldControl::AdaptationAndPayload, Box::new([0x03, 0x1E, 0x02, 0xFE]));
    }

    #[test_case(packet_2)]
    #[test_case(packet_1)]
    fn from_bytes(packet: fn() -> (Box<[u8]>, AdaptationFieldControl, Box<[u8]>)) {
        let (mut buf, adaptation_field_control, first_packet_bytes) = packet();
        let packet = TSPacket::from_bytes(&mut buf).unwrap();
        
        assert_eq!(packet.header().adaptation_field_control(), adaptation_field_control, "Transport Error Indicator is incorrect");

        let real_first_bytes: Box<[u8]> = packet.payload().unwrap().data()[0..first_packet_bytes.len()].into();
        assert!(real_first_bytes.iter().eq(first_packet_bytes.iter()), "First payload bytes are incorrect: {:02X?}", real_first_bytes);
    }
}