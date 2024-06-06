extern crate ts_analyzer;

use std::{env, io::Cursor};
use ts_analyzer::reader::TSReader;

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

    let mut packets: usize = 0;
    loop {
        // Get the next payload. Panic if we hit an unexpected error.
        let possible_packet = match reader.next_packet() {
            Ok(packet) => packet,
            Err(e) => panic!("An error was hit!: {}", e)
        };

        // When `next_payload` returns `Ok(None)` that means the reader has been fully read
        // through. 
        let Some(_) = possible_packet else {
            break;
        };

        packets += 1;
    }

    println!("Packets in file: {}", packets);
}