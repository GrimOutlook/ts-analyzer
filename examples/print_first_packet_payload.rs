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
    let mut reader = TSReader::new(buf_reader);
    // Get the first packet's payload data.
    let payload_data = reader.read_next_packet().unwrap().payload().unwrap();
    println!("Payload bytes: {:#?}", payload_data);
}