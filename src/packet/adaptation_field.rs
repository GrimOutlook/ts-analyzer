/// All of this information is shamelessly stolen from wikipedia, my lord and savior.
/// This [article](https://en.wikipedia.org/wiki/MPEG_transport_stream) in particular. Please donate
/// to wikipedia if you have the means.
#[derive(Clone, Debug)]
pub struct TSAdaptationField {
    /// Number of bytes that make up the adaptation field.
    ///
    /// TODO: Determine what this length includes. The documentation isn't super clear but seems to
    /// imply that it includes everything in the adaptation field.
    /// Not entirely sure if this includes the dynamic data that has its own length field such as
    /// `Transport Private Data` or `Adaptation Field Extension`.
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

impl TSAdaptationField {
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
}
