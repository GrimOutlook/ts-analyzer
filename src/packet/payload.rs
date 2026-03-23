//! TSPayload keeps track of the payload data.

use crate::ErrorKind;

pub type PayloadBytes = Vec<u8>;

#[derive(Clone, Debug, PartialEq)]
pub enum TsPayloadData {
    StartData(PayloadBytes, PayloadBytes),
    Data(PayloadBytes),
}

#[derive(Clone, Debug, PartialEq)]
/// Payload of a transport stream object.
pub struct TsPayload {
    /// The raw bytes contained in the payload (excluding the Payload Pointer
    /// if one exists)
    data: TsPayloadData,
    /// The continuity counter keeps track of the order in which packets get
    /// created for a specific PID.
    ///
    /// This is a clone of the continuity counter found in the header. I have
    /// chosen to duplicate it here due to the fact that it is useful to
    /// have when attempting to stitch payloads together. By including it
    /// the user does not have to store every `TSPacket` and can instead just
    /// store `TSPayload` objects.
    ///
    /// This is stored as a u8 but should actually be a u4 as it is only made
    /// up of 4 bits in the header.
    continuity_counter: u8,
}

impl TsPayload {
    /// Parse the payload data and `pusi` from the raw payload bytes.
    pub fn from_bytes(
        pusi: bool,
        continuity_counter: u8,
        payload_data: &[u8],
    ) -> TsPayload {
        assert!(!payload_data.is_empty(), "Payload data is empty");
        let data = if pusi {
            let (start_index, full_data) = payload_data.split_first().unwrap();
            let (end_data, start_data) =
                full_data.split_at_checked(*start_index as usize).unwrap();
            assert!(!start_data.is_empty(), "Starting payload data is empty");
            TsPayloadData::StartData(end_data.to_vec(), start_data.to_vec())
        } else {
            TsPayloadData::Data(payload_data.to_vec())
        };

        TsPayload { data, continuity_counter }
    }

    /// Return the continuity counter of this payload.
    pub fn continuity_counter(&self) -> u8 {
        self.continuity_counter
    }

    /// Returns if this payload contains the start of a new payload in it's
    /// data.
    pub fn is_start(&self) -> bool {
        matches!(self.data, TsPayloadData::StartData(_, _))
    }

    /// Returns the current payload data. This is the data before the start
    /// index, if one exists.
    pub fn get_payload_data(self) -> TsPayloadData {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    // #[test_case(true, Some(2), 2; "Payload contains start")]
    // #[test_case(false, None, 10; "Payload does not contain start")]
    // fn from_bytes(pusi: bool, start_index: Option<u8>, continuity_counter:
    // u8) {     let raw_data = [2, 1, 2, 3, 4];
    //     let expected_data: Box<[u8]> = match pusi {
    //         true => Box::from(raw_data[1..raw_data.len()].to_vec()),
    //         false => Box::from(raw_data),
    //     };
    //
    //     let payload =
    //         TsPayload::from_bytes(pusi, continuity_counter, &raw_data);
    //     assert_eq!(payload, expected_data, "Data is not the same");
    //     assert_eq!(
    //         payload.start_index(),
    //         start_index,
    //         "Start index is not the same"
    //     );
    //     assert_eq!(
    //         payload.continuity_counter(),
    //         continuity_counter,
    //         "Continuity counter is not the same"
    //     );
    // }

    #[test_case(true; "Payload contains start")]
    #[test_case(false; "Payload does not contain start")]
    fn is_start(is_start: bool) {
        let payload = TsPayload::from_bytes(is_start, 0, &[2, 1, 2, 3, 4]);
        assert_eq!(payload.is_start(), is_start, "Start payload is incorrect");
    }

    // #[test_case(true; "Payload contains start")]
    // #[test_case(false; "Payload does not contain start")]
    // fn get_current_data(pusi: bool) {
    //     let raw_data = [2, 1, 2, 3, 4];
    //     let expected_data: Box<[u8]> = match pusi {
    //         true => {
    //             // We add 1 because in the actual function we remove the
    // first             // item when the PUSI is true
    //             let idx = raw_data[0] + 1;
    //             Box::from(raw_data[1..idx as usize].to_vec())
    //         }
    //         false => Box::from(raw_data),
    //     };
    //
    //     let payload = TsPayload::from_bytes(pusi, 0, &raw_data);
    //     assert_eq!(payload, expected_data, "Current data is not the same");
    // }

    // #[test_case(true; "Payload contains start")]
    // #[test_case(false; "Payload does not contain start")]
    // fn get_start_data(pusi: bool) {
    //     let raw_data = [2, 1, 2, 3, 4];
    //     let expected_data: Result<Box<[u8]>, ErrorKind> = match pusi {
    //         true => {
    //             // We add 1 because in the actual function we remove the
    // first             // item when the PUSI is true
    //             let idx = raw_data[0] + 1;
    //             Ok(Box::from(raw_data[idx as
    // usize..raw_data.len()].to_vec()))         }
    //         false => Err(ErrorKind::PayloadIsNotStart),
    //     };
    //     let payload = TsPayload::from_bytes(pusi, 0, &raw_data);
    //
    //     match payload.get_start_data() {
    //         Ok(data) => assert!(
    //             data.iter().eq(expected_data.unwrap().iter()),
    //             "Start data is incorrect"
    //         ),
    //         Err(data) => assert_eq!(
    //             data.to_string(),
    //             expected_data.unwrap_err().to_string(),
    //             "Incorrect error type"
    //         ),
    //     };
    // }
}
