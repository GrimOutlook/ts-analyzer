
/// All of this information is shamelessly stolen from wikipedia, my lord and savior.
/// This [article](https://en.wikipedia.org/wiki/MPEG_transport_stream) in particular. Please donate
/// to wikipedia if you have the means.
#[derive(Clone, Copy, Debug)]
pub struct TSAdaptationField {
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
    /// Program clock reference
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
    transport_private_data_length: Option<bool>,
}