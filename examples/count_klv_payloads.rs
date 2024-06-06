extern crate ts_analyzer;

use std::{env, io::Cursor};
use ts_analyzer::reader::TSReader;
use memmem::{Searcher, TwoWaySearcher};

// Define the KLV header to search for. This can be found in the MISB standards documentation.
const KLV_HEADER: &[u8; 16] = b"\x06\x0E\x2B\x34\x02\x0B\x01\x01\x0E\x01\x03\x01\x01\x00\x00\x00";

fn main() {
    env_logger::init();
    let filename = env::var("TEST_FILE").expect("Environment variable not set");
    println!("Reading data from {}", filename);

    let f = std::fs::read(filename).expect("Couldn't open file");
    // Load the file into memory. This is memory expensive but lowers the I/O wait-time greatly.
    // TODO: This actually does nothing to the running speed. This points to unoptimized
    // code.
    let c = Cursor::new(f);
    // Reader must be mutable due to internal state changing to keep track of what packet is to be
    // read next and what payloads are being tracked.
    let mut reader = TSReader::new( c).expect("Transport Stream file contains no SYNC bytes.");
    let search = TwoWaySearcher::new(KLV_HEADER);

    let mut payloads: usize = 0;
    loop {
        // Get the next payload. Panic if we hit an unexpected error.
        let possible_payload = match reader.next_payload() {
            Ok(payload) => payload,
            Err(e) => panic!("An error was hit!: {}", e)
        };

        // When `next_payload` returns `Ok(None)` that means the reader has been fully read
        // through. 
        let Some(payload) = possible_payload else {
            break;
        };

        // Check to see if the KLV header is present in the payload that was found
        if search.search_in(&payload).is_some() {
            payloads += 1;
        }
    }

    println!("KLV payloads found: {}", payloads);
}