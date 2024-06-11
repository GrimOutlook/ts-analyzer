# ts-analyzer

![Crates.io Total Downloads](https://img.shields.io/crates/d/ts-analyzer)
![docs.rs](https://img.shields.io/docsrs/ts-analyzer)
![Crates.io Version](https://img.shields.io/crates/v/ts-analyzer)
![GitHub Repo stars](https://img.shields.io/github/stars/GrimOutlook/ts-analyzer)
![Crates.io License](https://img.shields.io/crates/l/ts-analyzer)


A library used for analyzing MPEG/Transport Stream files. This library is not intended for encoding, decoding or multiplexing transport streams. It has mainly been created for KLV extraction using [klv-reader](https://github.com/GrimOutlook/klv-reader).

```rust
extern crate ts_reader;

use std::env;
use ts_reader::reader::TSReader;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let filename = env::var("TEST_FILE").unwrap();
    println!("Reading data from {}", filename);

    let f = File::open(filename).unwrap();
    let buf_reader = BufReader::new(f);
    // Reader must be mutable due to internal state changing to keep track of what packet is to be
    // read next.
    let mut reader = TSReader::new(buf_reader).unwrap();
    // Get the first packet's payload data.
    let payload_data = reader.read_next_packet() // Read the first TS packet from the file.
                             .expect("Error reading file") // Assume there was no error reading the file.
                             .expect("No valid TSPacket found") // Assume that a TSPacket was found in the file.
                             .payload() // Get the payload of the TSPacket.
                             .expect("No payload data in packet"); // Assume that there was payload data in the TSPacket.
    println!("Payload bytes: {:#?}", payload_data);
}
```

---

## Reference Material

A sample TS stream with KLV data can be found [here](https://www.arcgis.com/home/item.html?id=55ec6f32d5e342fcbfba376ca2cc409a).

- [Wikipedia: MPEG Transport Stream](https://en.wikipedia.org/wiki/MPEG_transport_stream)
- [MPEG Official Documentation](https://www.itu.int/rec/dologin_pub.asp?lang=e&id=T-REC-H.222.0-201703-S!!PDF-E&type=items)
