# ts-analyzer

[![Crates.io Total Downloads](https://img.shields.io/crates/d/ts-analyzer)](https://crates.io/crates/ts-analyzer)
[![docs.rs](https://img.shields.io/docsrs/ts-analyzer)](https://docs.rs/ts-analyzer)
[![Crates.io Version](https://img.shields.io/crates/v/ts-analyzer)](https://crates.io/crates/ts-analyzer/versions)
[![GitHub Repo stars](https://img.shields.io/github/stars/GrimOutlook/ts-analyzer)](https://github.com/GrimOutlook/ts-analyzer)
[![Crates.io License](https://img.shields.io/crates/l/ts-analyzer)](LICENSE)


A library used for analyzing MPEG/Transport Stream files. This library is not intended for encoding, decoding or multiplexing transport streams. It has mainly been created for KLV extraction using [klv-reader](https://github.com/GrimOutlook/klv-reader).

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
    println!("Reading data from {}", filename);

    let f = File::open(filename.clone()).expect("Couldn't open file");
    let buf_reader = BufReader::new(f);
    // Reader must be mutable due to internal state changing to keep track of what packet is to be
    // read next.
    let mut reader = TSReader::new(&filename, buf_reader).expect("Transport Stream file contains no SYNC bytes.");

    let mut packet;
    loop {
        println!("Reading packet");
        // Run through packets until we get to one with a payload.
        packet = reader.next_packet_unchecked() // Read the first TS packet from the file.
                       .expect("No valid TSPacket found"); // Assume that a TSPacket was found in the file.

        if packet.has_payload()  {
            break
        }
    }

    let payload = packet.payload();
    assert!(payload.is_some(), "No payload in packet");
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
- [ ] Parse complete payloads from multiple packets
    - [ ] Track packets based on PID
    - [ ] Concatenate payloads of the same PID based on continuity counter

---

## Reference Material

A sample TS stream with KLV data can be found [here](https://www.arcgis.com/home/item.html?id=55ec6f32d5e342fcbfba376ca2cc409a).

- [Wikipedia: MPEG Transport Stream](https://en.wikipedia.org/wiki/MPEG_transport_stream)
- [MPEG Official Documentation](https://www.itu.int/rec/dologin_pub.asp?lang=e&id=T-REC-H.222.0-201703-S!!PDF-E&type=items)
