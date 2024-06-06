# ts-analyzer

[![Crates.io Total Downloads](https://img.shields.io/crates/d/ts-analyzer)](https://crates.io/crates/ts-analyzer)
[![docs.rs](https://img.shields.io/docsrs/ts-analyzer)](https://docs.rs/ts-analyzer)
[![Crates.io Version](https://img.shields.io/crates/v/ts-analyzer)](https://crates.io/crates/ts-analyzer/versions)
[![GitHub Repo stars](https://img.shields.io/github/stars/GrimOutlook/ts-analyzer)](https://github.com/GrimOutlook/ts-analyzer)
[![Crates.io License](https://img.shields.io/crates/l/ts-analyzer)](../LICENSE)


A library used for analyzing MPEG/Transport Stream files. This library is not intended for encoding,
decoding or multiplexing transport streams. It has mainly been created for payload extraction and
packet analysis of transport stream packets. Specifically in the case of KLV extraction using
[klv-reader](https://github.com/GrimOutlook/klv-reader).

## Example

```rust
extern crate ts_analyzer;

use std::env;
use ts_analyzer::reader::TSReader;
use std::fs::File;
use std::io::BufReader;

fn main() {
    env_logger::init();
    let filename = env::var("TEST_FILE").expect("Environment variable not set");

    // We need to open the file first before we can create a TSReader with it. TSReader accepts
    // anything that implements `Read + Seek` traits so `Cursor` and `BufReader` are other common
    // classes that can be used. 
    let f = File::open(filename.clone()).expect("Couldn't open file");
    // Reader must be mutable due to internal state changing to keep track of what packet is to be
    // read next. `.new()` returns a result because if a SYNC bytes (`0x047`) cannot be found in
    // the stream or SYNC bytes are not found in a repeating pattern 188 bytes apart then this is
    // not a valid transport stream.
    let mut reader = TSReader::new(f).expect("Transport Stream file contains no SYNC bytes.");

    let mut packet;
    loop {
        // Get a packet from the reader. The `unchecked` in the method name means that if an error
        // is hit then `Some(packet)` is returned rather than `Ok(Some(packet))` in order to reduce
        // `.unwrap()` (or other) calls.
        packet = reader.next_packet_unchecked()
                       // Assume that a TSPacket was found in the file and was successfully parsed.
                       .expect("No valid TSPacket found");

        // Return once we have found a packet that has a payload.
        if packet.has_payload()  {
            break
        }
    }

    // This is only the payload data from this packet. This is not all of the data from a full
    // payload which can be split among many packets. If that is what is desired look into using
    // the `.next_payload()` and `.next_payload_unchecked()` methods instead.
    let payload = packet.payload();
    // We have to unwrap the `payload()` call since some packets may not have a payload.
    println!("Payload bytes: {:02X?}", payload.unwrap().data());
}
```

---

## Goals

- [ ] Parse transport stream packets
    - [x] Parse transport stream packet header
    - [x] Parse transport stream packet adaptation field
    - [ ] Parse transport stream packet adaptation extension field
    - [x] Be able to dump raw payload bytes from packet
- [x] Parse complete payloads from multiple packets
    - [x] Track packets based on PID
    - [x] Concatenate payloads of the same PID based on continuity counter
- [ ] Improve throughput of packet and payload reading.
    - Current speeds are around 15MB/s for payload reading and 22MB/s for packet reading 
    even with data directly in memory.

---

## Reference Material

A sample TS stream with KLV data can be found [here](https://www.arcgis.com/home/item.html?id=55ec6f32d5e342fcbfba376ca2cc409a).

- [Wikipedia: MPEG Transport Stream](https://en.wikipedia.org/wiki/MPEG_transport_stream)
- [MPEG Official Documentation](https://www.itu.int/rec/dologin_pub.asp?lang=e&id=T-REC-H.222.0-201703-S!!PDF-E&type=items)
