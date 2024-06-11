use std::error::Error;
use crate::errors::no_payload::NoPayload;
use crate::packet::payload::{self, TSPayload};
use crate::packet::TSPacket;

pub(crate) struct TrackedPayload {
    /// PID of the packet that these payloads belong to.
    pid: u16,
    /// Payloads that we have parsed so far.
    /// 
    /// These are stored in the order that they have been read from the file.
    payloads: Vec<TSPayload>,
}

impl TrackedPayload {
    /// Create a new tracked payload
    pub fn new(pid: u16) -> Self {
        TrackedPayload {
            pid,
            payloads: Vec::new(),
        }
    }

    /// Create a tracked payload object from a packet.
    ///
    /// This initializes the object with only the payload data of the packet that was passed in.
    pub fn from_packet(packet: &TSPacket) -> Result<Self, Box<dyn Error>> {
        let payload = match packet.payload() {
            Some(payload) => payload,
            None => return Err(Box::new(NoPayload))
        };

        Ok(TrackedPayload {
            pid: packet.header().pid(),
            payloads: vec!(payload)
        })
    }

    /// Adds raw payload bytes from a TSPayload object
    /// 
    /// If there are no payloads currently stored, and we are trying to add a payload that does not
    /// have the `PUSI` set, we do not add it as we will not be able to extract a full payload
    /// without the first payload that has the `PUSI` set.
    pub fn add(&mut self, payload: &TSPayload) {
        if ! payload.is_start() && self.payloads.is_empty() {
            return;
        }

        self.payloads.push(payload.clone());
    }

    /// Adds raw payload bytes from a TSPayload object and returns a completed payload if one exists
    pub fn add_and_get_complete(&mut self, payload: &TSPayload) -> Option<Box<[u8]>> {
        self.add(payload);

        return self.get_completed();
    }

    /// Check to see if there is a completed payload in the payloads vector and return the completed
    /// payload data if there is.
    pub fn get_completed(&mut self) -> Option<Box<[u8]>> {
        // Find the first payload with a start index.
        let Some(start_partial_payload) = self.payloads.iter().position(|payload| payload.is_start()) else {
            return None;
        };
        let Some(end_partial_payload) = self.payloads.iter().rposition(|payload| payload.is_start()) else {
            return None;
        };

        // If the indices are the same then we cannot determine if all the payload data has been
        // found.
        if start_partial_payload == end_partial_payload {
            return None;
        }

        let mut data_vec = vec![];
        
        // Retrieve the data after the start index from the first partial payload.
        data_vec.push(self.payloads[start_partial_payload].get_start_data().unwrap());
        for idx in start_partial_payload+1..=end_partial_payload {
            data_vec.push(self.payloads[idx].get_current_data());
        }

        // Remove all of the payloads that have just been read, except the last one. The last one
        // will have data that pertains to the next payload.
        for _ in start_partial_payload..end_partial_payload {
            self.payloads.remove(start_partial_payload);
        }

        // TODO: Investigate changing this `.into_vec()` call to something else. This is the only
        // way I could get it to work and it's likely that this has performance impacts.
        return Some(data_vec.iter().flat_map(|s| s.clone().into_vec()).collect())
    }

    /// Get the PID of the payload being tracked
    pub fn pid(&self) -> u16 {
        self.pid
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(true, 1; "Payload contains start")]
    #[test_case(false, 0; "Payload does not contain start")]
    fn add_one (pusi: bool, expected_len: usize) {
        let raw_data = [2, 1, 2, 3, 4];
        
        let payload = TSPayload::from_bytes(pusi, 0, Box::new(raw_data));
        let mut tp = TrackedPayload::new(0);
        tp.add(&payload);

        assert_eq!(tp.payloads.len(), expected_len, "Tracked payloads is not the expected length");
    }

    #[test]
    fn get_completed () {
        let mut tp = TrackedPayload::new(0);
        
        let raw_data = [2, 1, 2, 3, 4];
        let expected_data: Box<[u8]> = Box::new([3, 4, 2, 1, 2, 3, 4, 1, 2]);
        let payload1 = TSPayload::from_bytes(true, 0, Box::new(raw_data));
        let payload2 = TSPayload::from_bytes(false, 0, Box::new(raw_data));
        let payload3 = TSPayload::from_bytes(true, 0, Box::new(raw_data));

        tp.add(&payload1);
        tp.add(&payload2);

        assert!(tp.get_completed().is_none(), "Payload is completed when it shouldn't be");

        tp.add(&payload3);

        let completed_payload = tp.get_completed();

        assert!(completed_payload.is_some(), "Payload is not completed");

        let data = completed_payload.unwrap();
        assert!(data.iter().eq(expected_data.iter()), "Completed packet data is incorrect: {:?}", data);

        // Verify that only the last packet's payload remains in the tracked payload vector.
        assert_eq!(tp.payloads.len(), 1, "Returned payloads are still being tracked");
    }
}