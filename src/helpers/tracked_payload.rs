use std::error::Error;
use crate::errors::no_payload_error::NoPayloadError;
use crate::packet::payload::TSPayload;
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
            None => return Err(Box::new(NoPayloadError))
        };

        Ok(TrackedPayload {
            pid: packet.header().pid(),
            payloads: vec!(payload)
        })
    }

    /// Adds raw payload bytes from a 
    pub fn add_payload(&mut self, payload: &TSPayload) -> bool {
        self.payloads.push(payload);

        false
    }

    /// Get the PID of the payload being tracked
    pub fn pid(&self) -> u16 {
        self.pid
    }
}