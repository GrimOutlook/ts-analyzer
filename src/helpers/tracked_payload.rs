use crate::packet::payload::TSPayload;

pub(crate) struct TrackedPayload {
    /// PID of the packet that these payloads belong to.
    pid: u16,
    /// Payloads that we have parsed so far.
    /// 
    /// These are stored in the order that they have been read from the file.
    payloads: Vec<TSPayload>,
}

impl TrackedPayload {
    /// Adds raw payload bytes from a 
    pub fn add_payload(&mut self, payload: TSPayload) -> bool {
        self.payloads.push(payload);

        false
    }
}