#[derive(Clone, Debug)]
pub struct TSPayload {
    /// The raw bytes contained in the payload (excluding the Payload Pointer if one exists)
    data: Box<[u8]>,
    /// Indicates where the new payload starts in the data section.
    ///
    /// This field will be `None` when the `PUSI` (Payload Unit Start Indicator) flag is `0` in the
    /// header.
    start_index: Option<u8>,
}

impl TSPayload {
    pub fn from_bytes(pusi: bool, payload_data: Box<[u8]>) -> TSPayload {
        let (start_index, data) = if (pusi) {
            (Some(payload_data[0]), Box::from(&payload_data[1..payload_data.len()]))
        } else {
            (None, payload_data)
        };

        TSPayload {
            data,
            start_index,
        }
    }

    pub fn data(&self) -> Box<[u8]> {
        return self.data.clone()
    }
}