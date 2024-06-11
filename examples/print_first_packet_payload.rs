extern crate ts_reader;

use std::env;
use ts_reader::reader::TSReader;
use std::fs::File;
use std::io::BufReader;

fn main() {
    env_logger::init();
    let filename = env::var("TEST_FILE").unwrap();
    println!("Reading data from {}", filename);

    let f = File::open(filename).unwrap();
    let buf_reader = BufReader::new(f);
    // Reader must be mutable due to internal state changing to keep track of what packet is to be
    // read next.
    let mut reader = TSReader::new(buf_reader).unwrap();

    let mut packet;
    loop {
        // Run through packets until we get to one with a payload.
        packet = reader.read_next_packet_unchecked() // Read the first TS packet from the file.
                             .expect("No valid TSPacket found"); // Assume that a TSPacket was found in the file.

        if packet.has_payload()  { // Check if this packet has a payload.
            break
        }
    }
    println!("Payload bytes: {:02X?}", packet.payload().expect("No payload in this packet").data());
}