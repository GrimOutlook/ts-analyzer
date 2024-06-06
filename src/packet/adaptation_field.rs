//! This module keep track of all the information stored in the adaptation field of the
//! transport stream packet header.

use bitvec::{field::BitField, order::Msb0, vec::BitVec};

/// The PCR field and OPCR field are 6 bytes in size.
pub const PCR_SIZE: u8 = 6;

/// The splice countdown field is 1 byte in size.
pub const SPLICE_COUNTDOWN_SIZE: u8 = 1;

/// The length of the transport private data length field is 1 byte in size.
pub const TRANSPORT_PRIVATE_DATA_LENGTH_LENGTH: u8 = 1;

/// This is created because an adaptation field can either be full of metadata as expected ***OR***
/// it can be a single stuffing byte. I don't want operations that work on a real adaptation field
/// to work on a stuffing adaptation field but I don't want to make the adaptation field `None`
/// either because the the `adaptation_control_field` still says the adaptation field is present.

#[derive(Clone, Debug)]
pub enum AdaptationField {
    /// Data adaptation fields are what you think of when looking at an adaptation field and contain
    /// actual data
    Data(DataAdaptationField),
    /// Stuffing adaptation fields contain 1 byte of stuffing.
    Stuffing(StuffingAdaptationField)
}

/// All of this information is shamelessly stolen from wikipedia, my lord and savior.
/// This [article](https://en.wikipedia.org/wiki/MPEG_transport_stream) in particular. Please donate
/// to wikipedia if you have the means.
#[derive(Clone, Debug)]
pub struct DataAdaptationField {
    /// Number of bytes that make up the adaptation field.
    /// 
    /// This includes all the dynamic data such as the PCR fields as well as the transport
    /// private data.
    adaptation_field_length: u8,
    /// Set if current TS packet is in a discontinuity state with respect to either the continuity
    /// counter or the program clock reference
    discontinuity_indicator: bool,
    /// Set when the stream may be decoded without errors from this point
    random_access_indicator: bool,
    /// Set when this stream should be considered "high priority"
    elementary_stream_priority_indicator: bool,
    /// Set when PCR (Program Clock Reference) field is present
    pcr_flag: bool,
    /// Set when OPCR (Original Program Clock Reference) field is present
    opcr_flag: bool,
    /// Set when splice countdown field is present
    splicing_point_flag: bool,
    /// Set when transport private data is present
    transport_private_data_flag: bool,
    /// Set when adaptation extension data is present
    adaptation_field_extension_flag: bool,
    /// Program clock reference. The PCR indicates the intended time of arrival of the byte
    /// containing the last bit of the program_clock_reference_base at the input of the system
    /// target decoder
    ///
    /// Is `None` if the PCR Flag is `false`.
    pcr: Option<u64>,
    /// Original Program clock reference. Helps when one TS is copied into another
    ///
    /// Is `None` if the OPCR Flag is `false`.
    opcr: Option<u64>,
    /// Indicates how many TS packets from this one a splicing point occurs. May be negative.
    ///
    /// Is `None` if the Splicing Point Flag is `false`.
    splice_countdown: Option<i8>,
    /// Length of the Transport Private Data field.
    ///
    /// Is `None` if the Transport Private Data Flag is `false`.
    transport_private_data_length: Option<u8>,
    /// Transport private data. I tried to look into what this is and couldn't fina any
    /// documentation.
    ///
    /// Is `None` if the Transport Private Data Flag is `false`.
    transport_private_data: Option<Box<[u8]>>,
}

impl DataAdaptationField {
    /// Create a new adaptation field.
    pub fn new(
        adaptation_field_length: u8,
        discontinuity_indicator: bool,
        random_access_indicator: bool,
        elementary_stream_priority_indicator: bool,
        pcr_flag: bool,
        opcr_flag: bool,
        splicing_point_flag: bool,
        transport_private_data_flag: bool,
        adaptation_field_extension_flag: bool,
        pcr: Option<u64>,
        opcr: Option<u64>,
        splice_countdown: Option<i8>,
        transport_private_data_length: Option<u8>,
        transport_private_data: Option<Box<[u8]>>,
    ) -> Self {

        Self {
            adaptation_field_length,
            discontinuity_indicator,
            random_access_indicator,
            elementary_stream_priority_indicator,
            pcr_flag,
            opcr_flag,
            splicing_point_flag,
            transport_private_data_flag,
            adaptation_field_extension_flag,
            pcr,
            opcr,
            splice_countdown,
            transport_private_data_length,
            transport_private_data,
        }
    }

    /// Parse the adaptation field from the passed in buffer
    pub fn from_bytes(buf: &mut [u8]) -> Self {
        // This is just used to track where we are reading each portion of the field.
        let mut read_idx = 0;
        
        // Get the length of the adaptation field.
        //
        // TODO: Determine if this takes into account the `Transport private data length` field or
        // not. If it doesn't then that field will need to be parsed as well. For the current moment
        // I'm assuming it takes it into account
        let adaptation_field_length: u8 =
            BitVec::<u8, Msb0>::from_slice(&buf[read_idx..read_idx + 1]).load_be();

        // Increment the read index since we just read a byte
        read_idx += 1;

        // Check if any of the dynamic fields are set. If these pop during testing I'll have to
        // implement them, but otherwise I'll leave them until necessary.
        let adaptation_field_required: BitVec<u8, Msb0> =
            BitVec::from_slice(&buf[read_idx..read_idx + 1]);

        // Increment the read index since we just read a byte
        read_idx += 1;

        let pcr_flag = adaptation_field_required[3];
        let opcr_flag = adaptation_field_required[4];
        let splicing_point_flag = adaptation_field_required[5];
        let transport_private_data_flag = adaptation_field_required[6];
        let adaptation_field_extension_flag = adaptation_field_required[7];

        let pcr = Self::read_pcr_data(&pcr_flag, buf, &mut read_idx);
        let opcr = Self::read_pcr_data(&opcr_flag, buf, &mut read_idx);

        let splice_countdown = match Self::read_data_conditionally(
            &splicing_point_flag,
            buf,
            &mut read_idx,
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
            &mut read_idx,
            TRANSPORT_PRIVATE_DATA_LENGTH_LENGTH as usize,
        ) {
            Some(bits) => {
                let length: u8 = bits.load();

                transport_private_data = Some(Box::from(
                    Self::read_data(buf, &mut read_idx, length as usize).as_raw_slice(),
                ));

                Some(length)
            }
            None => {
                transport_private_data = None;

                None
            }
        };

        DataAdaptationField {
            adaptation_field_length,
            discontinuity_indicator: adaptation_field_required[0],
            random_access_indicator: adaptation_field_required[1],
            elementary_stream_priority_indicator: adaptation_field_required[2],
            pcr_flag,
            opcr_flag,
            splicing_point_flag,
            transport_private_data_flag,
            adaptation_field_extension_flag,
            pcr,
            opcr,
            splice_countdown,
            transport_private_data_length,
            transport_private_data,
        }
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

    /// Returns the number of bytes that make up the adaptation field length
    pub fn adaptation_field_length(&self) -> u8 {
        self.adaptation_field_length
    }

    /// Returns if the discontinuity indicator is set or not
    pub fn discontinuity_indicator(&self) -> bool {
        self.discontinuity_indicator
    }

    /// Returns if the random access indicator is set or not
    pub fn random_access_indicator(&self) -> bool {
        self.random_access_indicator
    }

    /// Returns if the elementary stream priority indicator is set or not
    pub fn elementary_stream_priority_indicator(&self) -> bool {
        self.elementary_stream_priority_indicator
    }

    /// Returns if the PCR flag is set or not
    pub fn pcr_flag(&self) -> bool {
        self.pcr_flag
    }

    /// Returns if the OPCR flag is set or not
    pub fn opcr_flag(&self) -> bool {
        self.opcr_flag
    }

    /// Returns if the splicing point flag is set or not
    pub fn splicing_point_flag(&self) -> bool {
        self.splicing_point_flag
    }

    /// Returns if the transport private data flag is set or not
    pub fn transport_private_data_flag(&self) -> bool {
        self.transport_private_data_flag
    }

    /// Returns if the adaptation extension field flag is set or not
    pub fn adaptation_extension_field_flag(&self) -> bool {
        self.adaptation_field_extension_flag
    }

    /// Returns the PCR in the packet if one exists
    pub fn pcr(&self) -> Option<u64> {
        self.pcr
    }

    /// Returns the OPCR in the packet if one exists
    pub fn opcr(&self) -> Option<u64> {
        self.opcr
    }

    /// Returns the splice countdown in the packet if one exists
    pub fn splice_countdown(&self) -> Option<i8> {
        self.splice_countdown
    }

    /// Returns the transport private data length in the packet if one exists
    pub fn transport_private_data_length(&self) -> Option<u8> {
        self.transport_private_data_length
    }

    /// Returns the splice countdown in the packet if one exists
    pub fn transport_private_data(&self) -> &Option<Box<[u8]>> {
        &self.transport_private_data
    }
}

/// How many stuffing bytes exist in an adaptation field with a length field of `0`
pub const STUFFING_ADAPTATION_FIELD_LENGTH: u8 = 1;

#[derive(Clone, Debug)]
/// An adaptation field with a length of `0` is a StuffingAdaptationField. It contains 1 byte of
/// stuffing per the standard.
pub struct StuffingAdaptationField {
    /// This value will always be 1. This object gets created when the adaptation_field_length field
    /// is `0`. This really means that the the `adaptation_field` is really just 1 stuffing byte.
    adaptation_field_length: u8
}

impl StuffingAdaptationField {
    /// Create a new stuffing adaptation field.
    pub fn new() -> StuffingAdaptationField {
        return StuffingAdaptationField {
            adaptation_field_length: STUFFING_ADAPTATION_FIELD_LENGTH
        }
    }

    /// Return the number of stuffing bytes in the stuffing adaptation field.
    /// 
    /// # Hint
    /// It's `1`.
    pub fn adaptation_field_length(&self) -> u8 {
        self.adaptation_field_length
    }
}