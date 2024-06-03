//! TSPayload keeps track of the payload data.

#[derive(Clone, Debug)]
/// Payload of a transport stream object.
pub struct TSPayload {
    /// The raw bytes contained in the payload (excluding the Payload Pointer if one exists)
    data: Box<[u8]>,
    /// Indicates where the new payload starts in the data section.
    ///
    /// This field will be `None` when the `PUSI` (Payload Unit Start Indicator) flag is `0` in the
    /// header.
    start_index: Option<u8>,
    /// The continuity counter keeps track of the order in which packets get created for a specific PID.
    /// 
    /// This is a clone of the continuity counter found in the header. I have chosen to duplicate it here
    /// due to the fact that it is useful to have when attempting to stitch payloads together. By including
    /// it the user does not have to store every `TSPacket` and can instead just store `TSPayload` objects.
    /// 
    /// This is stored as a u8 but should actually be a u4 as it is only made up of 4 bits in the header.
    continuity_counter: u8,
}

impl TSPayload {
    /// Parse the payload data and `pusi` from the raw payload bytes.
    pub fn from_bytes(pusi: bool, continuity_counter: u8, payload_data: Box<[u8]>) -> TSPayload {
        let (start_index, data) = if pusi {
            (Some(payload_data[0]), Box::from(&payload_data[1..payload_data.len()]))
        } else {
            (None, payload_data)
        };

        TSPayload {
            data,
            start_index,
            continuity_counter,
        }
    }

    /// Return a reference to the raw data stored in the payload.
    pub fn data(&self) -> &Box<[u8]> {
        &self.data
    }

    /// Return the continuity counter of this payload.
    pub fn continuity_counter(&self) -> u8 {
        self.continuity_counter
    }
}