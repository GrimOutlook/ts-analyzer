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