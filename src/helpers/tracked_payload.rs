#[cfg(feature = "tracing")]
use tracing::trace;

use crate::ErrorKind;
use crate::packet::TsPacket;
use crate::packet::payload::PayloadBytes;
use crate::packet::payload::TsPayload;
use crate::packet::payload::TsPayloadData;

#[derive(Debug)]
pub(crate) struct TrackedPayload {
    /// PID of the packet that these payloads belong to.
    pid: u16,
    /// Payloads that we have parsed so far.
    ///
    /// Starts as `None` until the fist packet with the correct PID and the
    /// PUSI set in the header. It is then `Some` for the remainder of the
    /// execution.
    current_data: Option<PayloadBytes>,
}

impl TrackedPayload {
    /// Create a tracked payload object from a packet.
    ///
    /// This initializes the object with only the payload data of the packet
    /// that was passed in.
    pub fn from_packet(packet: TsPacket) -> Result<Self, ErrorKind> {
        let pid = &packet.header().pid();
        let Some(payload) = packet.to_payload() else {
            return Err(ErrorKind::NoPayload);
        };

        let TsPayloadData::StartData(_end, start) = payload.get_payload_data()
        else {
            return Err(ErrorKind::PayloadIsNotStart);
        };

        Ok(TrackedPayload { pid: *pid, current_data: Some(start) })
    }

    /// Adds raw payload bytes from a TSPayload object
    // # Panics
    // Panics if the `payloads` property is empty. This should never happen as
    // to instantiate a new tracked payload you need to include the first
    // payload which must contain the PUSI.
    pub fn add(&mut self, payload: TsPayload) -> Option<Vec<u8>> {
        match payload.get_payload_data() {
            TsPayloadData::Data(mut data) => {
                if let Some(ref mut current_data) = self.current_data {
                    current_data.append(&mut data);
                };
                None
            }
            TsPayloadData::StartData(mut end, start) => {
                match self.current_data {
                    Some(ref mut current_data) => {
                        // Append the end data to the payload
                        current_data.append(&mut end);
                        // Move the now-complete payload data out of the tracker
                        let output = current_data.to_owned();

                        // Make the new payload data what is listed at the end
                        *current_data = start.to_vec();

                        #[cfg(feature = "tracing")]
                        trace!("Completed payload data: {:2X?}", payload_data);

                        Some(output)
                    }
                    None => {
                        self.current_data = Some(start);
                        None
                    }
                }
            }
        }
    }

    /// Get the PID of the payload being tracked
    pub fn pid(&self) -> u16 {
        self.pid
    }
}

// #[cfg(test)]
// mod tests {
//     use test_case::test_case;
//
//     use super::*;
//
//     #[test_case(true, 1; "Payload contains start")]
//     #[test_case(false, 0; "Payload does not contain start")]
//     fn add_one(pusi: bool, expected_len: usize) {
//         let raw_data = [2, 1, 2, 3, 4];
//
//         let payload = TsPayload::from_bytes(pusi, 0, Box::new(raw_data));
//         let mut tp = TrackedPayload::new(0);
//         tp.add(&payload);
//
//         assert_eq!(
//             tp.payloads.len(),
//             expected_len,
//             "Tracked payloads is not the expected length"
//         );
//     }
//
//     #[test]
//     fn get_completed_2_packet() {
//         let mut tp = TrackedPayload::new(0);
//
//         let raw_data = [2, 1, 2, 3, 4];
//         let expected_data: Box<[u8]> = Box::new([3, 4, 1, 2]);
//         let payload1 = TsPayload::from_bytes(true, 0, Box::new(raw_data));
//         let payload2 = TsPayload::from_bytes(true, 0, Box::new(raw_data));
//
//         tp.add(&payload1);
//
//         assert!(
//             tp.get_completed().is_none(),
//             "Payload is completed when it shouldn't be"
//         );
//
//         tp.add(&payload2);
//
//         let completed_payload = tp.get_completed();
//
//         assert!(completed_payload.is_some(), "Payload is not completed");
//
//         let data = completed_payload.unwrap();
//         assert!(
//             data.iter().eq(expected_data.iter()),
//             "Completed packet data is incorrect: {:?}",
//             data
//         );
//
//         // Verify that only the last packet's payload remains in the tracked
//         // payload vector.
//         assert_eq!(
//             tp.payloads.len(),
//             1,
//             "Returned payloads are still being tracked"
//         );
//     }
//
//     #[test]
//     fn get_completed_3_packet() {
//         let mut tp = TrackedPayload::new(0);
//
//         let raw_data = [2, 1, 2, 3, 4];
//         let expected_data: Box<[u8]> = Box::new([3, 4, 2, 1, 2, 3, 4, 1, 2]);
//         let payload1 = TsPayload::from_bytes(true, 0, Box::new(raw_data));
//         let payload2 = TsPayload::from_bytes(false, 0, Box::new(raw_data));
//         let payload3 = TsPayload::from_bytes(true, 0, Box::new(raw_data));
//
//         tp.add(&payload1);
//         tp.add(&payload2);
//
//         assert!(
//             tp.get_completed().is_none(),
//             "Payload is completed when it shouldn't be"
//         );
//
//         tp.add(&payload3);
//
//         let completed_payload = tp.get_completed();
//
//         assert!(completed_payload.is_some(), "Payload is not completed");
//
//         let data = completed_payload.unwrap();
//         assert!(
//             data.iter().eq(expected_data.iter()),
//             "Completed packet data is incorrect: {:?}",
//             data
//         );
//
//         // Verify that only the last packet's payload remains in the tracked
//         // payload vector.
//         assert_eq!(
//             tp.payloads.len(),
//             1,
//             "Returned payloads are still being tracked"
//         );
//     }
// }
