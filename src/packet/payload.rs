//! TSPayload keeps track of the payload data.

use std::error::Error;

use crate::errors::payload_is_not_start::PayloadIsNotStart;

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
    /// The continuity counter keeps track of the order in which packets get created for a specific
    /// PID.
    /// 
    /// This is a clone of the continuity counter found in the header. I have chosen to duplicate it
    /// here due to the fact that it is useful to have when attempting to stitch payloads together.
    /// By including it the user does not have to store every `TSPacket` and can instead just store
    /// `TSPayload` objects.
    /// 
    /// This is stored as a u8 but should actually be a u4 as it is only made up of 4 bits in the
    /// header.
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

    /// Get the start index of this payload
    pub fn start_index(&self) -> Option<u8> {
        self.start_index
    }
    
    /// Returns if this payload contains the start of a new payload in it's data.
    pub fn is_start(&self) -> bool {
        self.start_index.is_some()
    }

    /// Returns the current payload data. This is the data before the start index, if one exists.
    pub fn get_current_data(&self) -> Box<[u8]> {
        if let Some(index) = self.start_index {
            // TODO: Investigate changing this `.to_vec()` call to something else. This is the only
            // way I could get it to work and it's likely that this has performance impacts.
            return Box::from(self.data[0..index as usize].to_vec())
        }

        return self.data.clone();
    }

    /// Returns the new payload data. This is the data after the start index, if one exists.
    pub fn get_start_data(&self) -> Result<Box<[u8]>, Box<dyn Error>> {
        let Some(index) = self.start_index else {
            return Err(Box::new(PayloadIsNotStart))
        };

        Ok(Box::from(self.data[index as usize..self.data.len()].to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;
    use test_case::test_case;

    #[test_case(true, Some(2), 2; "Payload contains start")]
    #[test_case(false, None, 10; "Payload does not contain start")]
    fn from_bytes(pusi: bool, start_index: Option<u8>, continuity_counter: u8) {
        let raw_data = [2, 1, 2, 3, 4];
        let expected_data: Box<[u8]> = match pusi {
            true => Box::from(raw_data[1..raw_data.len()].to_vec()),
            false => Box::from(raw_data),
        };

        let payload = TSPayload::from_bytes(pusi, continuity_counter, Box::new(raw_data));
        assert!(payload.data().iter().eq(expected_data.iter()), "Data is not the same");
        assert_eq!(payload.start_index(), start_index, "Start index is not the same");
        assert_eq!(payload.continuity_counter(), continuity_counter, "Continuity counter is not the same");
    }

    #[test_case(true; "Payload contains start")]
    #[test_case(false; "Payload does not contain start")]
    fn is_start(is_start: bool) {
        let payload = TSPayload::from_bytes(is_start, 0, Box::new([2, 1, 2, 3, 4]));
        assert_eq!(payload.is_start(), is_start, "Start payload is incorrect");
    }

    #[test_case(true; "Payload contains start")]
    #[test_case(false; "Payload does not contain start")]
    fn get_current_data(pusi: bool) {
        let raw_data = [2, 1, 2, 3, 4];
        let expected_data: Box<[u8]> = match pusi {
            true => {
                // We add 1 because in the actual function we remove the first item when the PUSI is
                // true
                let idx = raw_data[0] + 1;
                Box::from(raw_data[1..idx as usize].to_vec())
            },
            false => Box::from(raw_data),
        };

        let payload = TSPayload::from_bytes(pusi, 0, Box::new(raw_data));
        assert!(payload.get_current_data().iter().eq(expected_data.iter()), "Current data is not the same");
    }

    #[test_case(true; "Payload contains start")]
    #[test_case(false; "Payload does not contain start")]
    fn get_start_data(pusi: bool) {
        let raw_data = [2, 1, 2, 3, 4];
        let expected_data: Result<Box<[u8]>, Box<dyn Error>> = match pusi {
            true => {
                // We add 1 because in the actual function we remove the first item when the PUSI is
                // true
                let idx = raw_data[0] + 1;
                Ok(Box::from(raw_data[idx as usize..raw_data.len()].to_vec()))
            },
            false => Err(Box::new(PayloadIsNotStart)),
        };
        let payload = TSPayload::from_bytes(pusi, 0, Box::new(raw_data));

        match payload.get_start_data() {
            Ok(data) => assert!(data.iter().eq(expected_data.unwrap().iter()), "Start data is incorrect"),
            Err(data) => assert_eq!(data.type_id(), expected_data.unwrap_err().type_id(), "Incorrect error type"),
        };
    }
}