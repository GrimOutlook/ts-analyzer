use crate::{AdaptationFieldControl, TransportScramblingControl};
use crate::AdaptationFieldControl::{AdaptationAndPayload, AdaptationField, Payload};
use crate::TransportScramblingControl::{EvenKey, NoScrambling, OddKey};


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
    pub fn new(
        tei: bool,
        pusi: bool,
        transport_priority: bool,
        pid: u16,
        tsc: u8,
        adaptation_field_control: u8,
        continuity_counter: u8,
    ) -> Self {
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
                _ => panic!("Invalid TSC value [{}]", tsc)
            },
            adaptation_field_control: match adaptation_field_control {
                0 => AdaptationFieldControl::Reserved,
                1 => Payload,
                2 => AdaptationField,
                3 => AdaptationAndPayload,
                _ => panic!("Invalid adaptation field control value [{}]", adaptation_field_control)
            },
            continuity_counter
        }
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

    /// Returns the continuity counter.
    pub fn continuity_counter(&self) -> u8 {
        self.continuity_counter
    }
}
