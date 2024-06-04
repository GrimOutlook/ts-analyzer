//! Holds all the information regarding a given packet from the transport stream.
pub mod payload;
pub mod header;
pub mod adaptation_field;

use crate::errors::invalid_first_byte::InvalidFirstByte;
use crate::packet::adaptation_field::TSAdaptationField;
use crate::packet::header::TSHeader;
use crate::AdaptationFieldControl::{AdaptationAndPayload, AdaptationField, Payload};
use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;
use std::error::Error;

use crate::packet::payload::TSPayload;
#[cfg(feature = "log")]
use log::trace;

/// All transport stream packets start with a SYNC byte.
pub const SYNC_BYTE: u8 = 0x47;

/// The PCR field and OPCR field are 6 bytes in size.
pub const PCR_SIZE: u8 = 6;

/// The splice countdown field is 1 byte in size.
pub const SPLICE_COUNTDOWN_SIZE: u8 = 1;

/// The length of the transport private data length field is 1 byte in size.
pub const TRANSPORT_PRIVATE_DATA_LENGTH_LENGTH: u8 = 1;

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
    adaptation_field: Option<TSAdaptationField>,
    /// Payload field data. This field will be `None` when the adaptation field control field has
    /// a `0` in the LSB place.
    payload: Option<TSPayload>,
}

impl TSPacket {
    /// Create a TSPacket from a byte array.
    pub fn from_bytes(buf: &mut [u8]) -> Result<TSPacket, Box<dyn Error>> {
        let header_bytes: BitVec<u8, Msb0> = BitVec::from_slice(&buf[0..HEADER_SIZE as usize]);

        // Check if the first byte is SYNC byte.
        if header_bytes[0..8].load::<u8>() != SYNC_BYTE {
            return Err(Box::new(InvalidFirstByte { byte: buf[0] }));
        }

        #[cfg(feature = "log")]
        trace!("Parsing TSPacket from raw bytes: {:02X?}", buf);

        // Get the header information from the header bytes
        let tei = header_bytes[9].as_bool();
        let pusi = header_bytes[10].as_bool();
        let transport_priority = header_bytes[11].as_bool();
        let pid = header_bytes[12..24].load();
        let transport_scrambling_control = header_bytes[24..26].to_bitvec().load();
        let adaptation_field_control = header_bytes[26..28].to_bitvec().load();
        let continuity_counter = header_bytes[28..32].load();

        let (adaptation_field_exists, payload_exists) = match adaptation_field_control {
            1 => (false, true),
            2 => (true, false),
            3 => (true, true),
            _ => {
                panic!("Unknown adaptation field control byte: [{}]", adaptation_field_control)
            }
        };

        let header = TSHeader::new(
            tei,
            pusi,
            transport_priority,
            pid,
            transport_scrambling_control,
            adaptation_field_control,
            continuity_counter,
        );
        #[cfg(feature = "log")]
        trace!("Header for TSPacket: {}", header);

        // This number comes from the fact that the TS header is always 4 bytes wide and the
        // adaptation field always comes directly after the header if it is present.
        let mut read_idx: usize = 4;

        // If the adaptation field is present we need to determine it's size as we want to ignore
        // it entirely.
        let adaptation_field = if adaptation_field_exists {
            #[cfg(feature = "log")]
            trace!("Adaptation field exists for TSPacket");
            Some(Self::parse_adaptation_field(buf, &mut read_idx))
        } else {
            None
        };

        // Handle the adaptation extension field.
        if adaptation_field.is_some()
            && adaptation_field.as_ref()
            .expect("Expected adaptation field to not be None. I even checked!")
            .has_adaptation_extension_field()
        {
            // We currently do nothing with the adaptation extension field.
            // TODO: Add support for adaptation extension field.
        };

        let payload = if payload_exists {
            #[cfg(feature = "log")]
            trace!("Payload exists for TSPacket");

            let payload_bytes: Box<[u8]> = Box::from(
                BitVec::<u8, Msb0>::from_slice(&buf[read_idx..buf.len()]).as_raw_slice()
            );
            Some(TSPayload::from_bytes(header.pusi(), continuity_counter, payload_bytes))
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
        match self.header().adaptation_field_control() {
            AdaptationField | AdaptationAndPayload => true,
            _ => false,
        }
    }

    /// Returns if the packet has payload field data.
    pub fn has_payload(&self) -> bool {
        match self.header().adaptation_field_control() {
            Payload | AdaptationAndPayload => true,
            _ => false,
        }
    }

    /// Return the adaptation field data.
    pub fn adaptation_field(&self) -> Option<TSAdaptationField> {
        self.adaptation_field.clone()
    }

    /// Return the payload data
    pub fn payload(&self) -> Option<TSPayload> {
        self.payload.clone()
    }

    fn read_data_conditionally(
        flag: &bool,
        buf: &mut [u8],
        read_idx: &mut usize,
        read_size: usize,
    ) -> Option<BitVec<u8, Msb0>> {
        if !flag {
            return None;
        }

        Some(Self::read_data(buf, read_idx, read_size))
    }

    fn read_data(buf: &mut [u8], read_idx: &mut usize, read_size: usize) -> BitVec<u8, Msb0> {
        // Read the  data from the given buffer location
        let bits: BitVec<u8, Msb0> = BitVec::from_slice(&buf[*read_idx..*read_idx + read_size]);

        // Increment the read index since we just read a `PCR_SIZE` amount of bytes.
        *read_idx += read_size;

        return bits;
    }

    /// Read the PCR (or OPCR) data from a starting index
    fn read_pcr_data(flag: &bool, buf: &mut [u8], read_idx: &mut usize) -> Option<u64> {
        let pcr_bits = match Self::read_data_conditionally(flag, buf, read_idx, PCR_SIZE as usize) {
            Some(bits) => bits,
            None => {
                // Return early if there is no field to be read, as seen by reading the flag.
                return None;
            }
        };

        // The first 33 bits are the "base" value which gets multiplied by `300`. This is defined in
        // the MPEG/TS standard.
        let base: u64 = pcr_bits[0..34].load();
        // The next 6 bits are reserved, so we will ignore them and the last 9 bits are the
        // "extension" which get added to the multiplied base.
        let extension: u64 = pcr_bits[39..48].load();

        return Some(base * 300 + extension);
    }

    fn parse_adaptation_field(buf: &mut [u8], read_idx: &mut usize) -> TSAdaptationField {
        // Get the length of the adaptation field.
        //
        // TODO: Determine if this takes into account the `Transport private data length` field or
        // not. If it doesn't then that field will need to be parsed as well. For the current moment
        // I'm assuming it takes it into account
        let adaptation_field_len: u8 =
            BitVec::<u8, Msb0>::from_slice(&buf[*read_idx..*read_idx + 1]).load();

        // Increment the read index since we just read a byte
        *read_idx += 1;

        // Check if any of the dynamic fields are set. If these pop during testing I'll have to
        // implement them, but otherwise I'll leave them until necessary.
        let adaptation_field_required: BitVec<u8, Msb0> =
            BitVec::from_slice(&buf[*read_idx..*read_idx + 1]);

        // Increment the read index since we just read a byte
        *read_idx += 1;

        // Create a little lambda function to reduce code duplication
        let read_bool = |bits: &BitVec<u8, Msb0>, index: usize| bits.get(index).unwrap().as_bool();

        let discontinuity_indicator = read_bool(&adaptation_field_required, 0);
        let random_access_indicator = read_bool(&adaptation_field_required, 1);
        let elementary_stream_priority_indicator = read_bool(&adaptation_field_required, 2);
        let pcr_flag = read_bool(&adaptation_field_required, 3);
        let opcr_flag = read_bool(&adaptation_field_required, 4);
        let splicing_point_flag = read_bool(&adaptation_field_required, 5);
        let transport_private_data_flag = read_bool(&adaptation_field_required, 6);
        let adaptation_field_extension_flag = read_bool(&adaptation_field_required, 7);

        let pcr_data = Self::read_pcr_data(&pcr_flag, buf, read_idx);
        let opcr_data = Self::read_pcr_data(&opcr_flag, buf, read_idx);

        let splice_countdown = match Self::read_data_conditionally(
            &splicing_point_flag,
            buf,
            read_idx,
            SPLICE_COUNTDOWN_SIZE as usize,
        ) {
            Some(bits) => Some(bits.load()),
            None => None,
        };

        // Putting this in the outer scope, so we can use the value in the TSAdapterField
        // constructor below.
        let transport_private_data: Option<Box<[u8]>>;

        let transport_private_data_length = match Self::read_data_conditionally(
            &transport_private_data_flag,
            buf,
            read_idx,
            TRANSPORT_PRIVATE_DATA_LENGTH_LENGTH as usize,
        ) {
            Some(bits) => {
                let length: u8 = bits.load();

                transport_private_data = Some(Box::from(
                    Self::read_data(buf, read_idx, length as usize).as_raw_slice(),
                ));

                Some(length)
            }
            None => {
                transport_private_data = None;

                None
            }
        };

        TSAdaptationField::new(
            adaptation_field_len,
            discontinuity_indicator,
            random_access_indicator,
            elementary_stream_priority_indicator,
            pcr_flag,
            opcr_flag,
            splicing_point_flag,
            transport_private_data_flag,
            adaptation_field_extension_flag,
            pcr_data,
            opcr_data,
            splice_countdown,
            transport_private_data_length,
            transport_private_data,
        )
    }
}
