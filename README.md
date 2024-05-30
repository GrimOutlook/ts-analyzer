# ts-reader

A library used for reading MPEG/Transport Stream files. Mainly created for KLV extraction using [klv-reader](https://github.com/GrimOutlook/klv-reader).

```rust
use ts_reader::TSReader;
use std::fs::File;

fn main() {
    let f = File::open("test_video.ts").unwrap();
    // Reader must be mutable due to internal state changing to keep track of what packet is to be read next.
    let mut reader = TSReader::new(f);
    // Get the first packet's payload data.
    let payload_data = reader.read_packet().unwrap().payload.unwrap();
    prinln!("Payload bytes: {:#?}", payload_data);
}
```

---

## Reference Material

A sample TS stream with KLV data can be found [here](https://www.arcgis.com/home/item.html?id=55ec6f32d5e342fcbfba376ca2cc409a).

- [Wikipedia: MPEG Transport Stream](https://en.wikipedia.org/wiki/MPEG_transport_stream)
