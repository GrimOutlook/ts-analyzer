//! This module keeps track of all the information stored in the header of a
//! transport stream packet.

use crate::errors::invalid_first_byte::InvalidFirstByte;
use crate::AdaptationFieldControl::{AdaptationAndPayload, AdaptationField, Payload};
use crate::TransportScramblingControl::{EvenKey, NoScrambling, OddKey};
use crate::{AdaptationFieldControl, TransportScramblingControl};
use std::error::Error;
use std::fmt::{Display, Formatter};
use bitvec::macros::internal::funty::Fundamental;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::vec::BitVec;
#[cfg(feature = "log")]
use log::trace;

/// All transport stream packets start with a SYNC byte.
pub const SYNC_BYTE: u8 = 0x47;

/// All of this information is shamelessly stolen from wikipedia, my lord and savior.
/// This [article](https://en.wikipedia.org/wiki/MPEG_transport_stream) in particular. Please donate
/// to wikipedia if you have the means.
#[derive(Clone, Copy, Debug)]
pub struct TSHeader {
    /// TEI: Transport error indicator is true when a packet is set when a demodulator cannot
    /// correct invalid_first_byte and indicates that the packet is corrupt.
    tei: bool,
    /// PUSI: Payload unit start indicator indicates if this packet contains the first byte of a
    /// payload since they can be spread across multiple packets.
    pusi: bool,
    /// Transport priority, set when the current packet is higher priority than other packets of the
    /// same PID
    transport_priority: bool,
    /// PID: Packet identifier of the transport stream packet. Describes what the payload data is.
    pid: u16,
    /// TSC: Transport scrambling control indicates whether the payload is encrypted and with what
    /// key. Valid values are:
    /// - `0` for no scrambling.
    /// - `1` is reserved.
    /// - `2` Scrambled with even key.
    /// - `3` Scrambled with odd key.
    tsc: TransportScramblingControl,
    /// Adaptation field control describes if this packet contains adaptation field data,
    /// payload data, or both. Valid values are:
    /// - `0` is reserved.
    /// - `1` for payload only.
    /// - `2` for adaptation field only.
    /// - `3` for adaptation field followed by payload.
    adaptation_field_control: AdaptationFieldControl,
    /// Continuity counter is used for determining the sequence of data in each PID.
    continuity_counter: u8,
}

impl TSHeader {
    /// Create a new header
    pub fn new(
        tei: bool,
        pusi: bool,
        transport_priority: bool,
        pid: u16,
        tsc: u8,
        adaptation_field_control: u8,
        continuity_counter: u8,
    ) -> Self {
        #[cfg(feature = "log")]
        {
            trace!("pid: [{}]", pid);
            trace!("adaptation_field_control: [{}]", adaptation_field_control);
            trace!("continuity_counter: [{}]", continuity_counter);
        }

        TSHeader {
            tei,
            pusi,
            transport_priority,
            pid,
            tsc: match tsc {
                0 => NoScrambling,
                1 => TransportScramblingControl::Reserved,
                2 => EvenKey,
                3 => OddKey,
                _ => panic!("Invalid TSC value [{}]", tsc),
            },
            adaptation_field_control: match adaptation_field_control {
                0 => AdaptationFieldControl::Reserved,
                1 => Payload,
                2 => AdaptationField,
                3 => AdaptationAndPayload,
                _ => panic!(
                    "Invalid adaptation field control value [{}]",
                    adaptation_field_control
                ),
            },
            continuity_counter,
        }
    }

    /// Get the packet header from raw bytes.
    pub fn from_bytes(buf: &Box<[u8]>) -> Result<TSHeader, Box<dyn Error>> {
        let bytes: BitVec<u8, Msb0> = BitVec::from_slice(buf).to_bitvec();

        // Check if the first byte is SYNC byte.
        if bytes[0..8].load::<u8>() != SYNC_BYTE {
            return Err(Box::new(InvalidFirstByte { byte: buf[0] }));
        }

        println!("header bytes: {:b}", bytes);
        println!("tei: {:b}", bytes[9] as u8);
        
        
        // Get the header information from the header bytes
        let tei = bytes[9];
        let pusi = bytes[10];
        let transport_priority = bytes[11];
        let pid = bytes[12..24].to_bitvec().load();
        let transport_scrambling_control = bytes[24..26].to_bitvec().load();
        let adaptation_field_control = bytes[26..28].to_bitvec().load();
        let continuity_counter = bytes[28..32].load();

        let header = TSHeader {
            tei,
            pusi,
            transport_priority,
            pid,
            tsc: match transport_scrambling_control {
                0 => NoScrambling,
                1 => TransportScramblingControl::Reserved,
                2 => EvenKey,
                3 => OddKey,
                _ => panic!("Invalid TSC value [{}]", transport_scrambling_control),
            },
            adaptation_field_control: match adaptation_field_control {
                0 => AdaptationFieldControl::Reserved,
                1 => Payload,
                2 => AdaptationField,
                3 => AdaptationAndPayload,
                _ => panic!(
                    "Invalid adaptation field control value [{}]",
                    adaptation_field_control
                ),
            },
            continuity_counter,
        };
        
        Ok(header)
    }

    /// Return if the transport error indicator is set.
    pub fn tei(&self) -> bool {
        self.tei
    }

    /// Return if the payload unit start indicator is set.
    pub fn pusi(&self) -> bool {
        self.pusi
    }

    /// Return if the transport priority is set.
    pub fn transport_priority(&self) -> bool {
        self.transport_priority
    }

    /// Returns the packet identifier.
    pub fn pid(&self) -> u16 {
        self.pid
    }

    /// Return's the transport scrambling control of this packet.
    pub fn tsc(&self) -> TransportScramblingControl {
        self.tsc
    }

    /// Adaptation field control value.
    pub fn adaptation_field_control(&self) -> AdaptationFieldControl {
        self.adaptation_field_control
    }

    /// Return whether this packet has an adaptation field or not
    pub fn has_adaptation_field(&self) -> bool {
        match self.adaptation_field_control {
            AdaptationField | AdaptationAndPayload => true,
            _ => false
        }
    }

    /// Return whether this packet has a payload or not
    pub fn has_payload(&self) -> bool {
        match self.adaptation_field_control {
            Payload | AdaptationAndPayload => true,
            _ => false
        }
    }

    /// Returns the continuity counter.
    pub fn continuity_counter(&self) -> u8 {
        self.continuity_counter
    }

    
}

impl Display for TSHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = format!("\n\
            TEI: {}\n\
            PUSI: {}\n\
            Transport Priority: {}\n\
            PID: {}\n\
            Transport Scrambling Control: {:?}\n\
            Adaptation Field Control: {:?}\n\
            Continuity Counter: {}",
            self.tei,
            self.pusi,
            self.transport_priority,
            self.pid,
            self.tsc,
            self.adaptation_field_control,
            self.continuity_counter,
        );
        write!(f, "{}", msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn from_bytes() {
        let buf: Box<[u8]> = Box::new([0x47, 0x01, 0x00, 0x1A]);
        let header = TSHeader::from_bytes(&buf).unwrap();
        assert_eq!(header.tei(), false, "Transport Error Indicator is incorrect");
        assert_eq!(header.pusi(), false, "Payload Unit Start Indicator is incorrect");
        assert_eq!(header.transport_priority(), false, "Transport Priority is incorrect");
        assert_eq!(header.pid(), 256, "Transport Priority is incorrect");
        assert_eq!(header.adaptation_field_control(), Payload, "Transport Priority is incorrect");
    }
}