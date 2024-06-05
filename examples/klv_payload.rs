extern crate ts_analyzer;

use std::env;
use ts_analyzer::reader::TSReader;
use std::fs::File;
use std::io::BufReader;
use memmem::{Searcher, TwoWaySearcher};

const KLV_HEADER: &[u8; 16] = b"\x06\x0E\x2B\x34\x02\x0B\x01\x01\x0E\x01\x03\x01\x01\x00\x00\x00";

fn main() {
    env_logger::init();
    let filename = env::var("TEST_FILE").expect("Environment variable not set");
    println!("Reading data from {}", filename);

    let f = File::open(filename.clone()).expect("Couldn't open file");
    let buf_reader = BufReader::new(f);
    // Reader must be mutable due to internal state changing to keep track of what packet is to be
    // read next and what payloads are being tracked.
    let mut reader = TSReader::new(&filename, buf_reader).expect("Transport Stream file contains no SYNC bytes.");
    let search = TwoWaySearcher::new(KLV_HEADER);

    let mut data: Box<[u8]> = Box::from([0]);
    loop {
        println!("Reading packet");
        // Run through packets until we get to one with a payload.
        let packet = match reader.next_packet() {
            Ok(payload) => payload.expect("No valid complete TS payload found"),
            Err(e) => {
                panic!("An error was hit!: {}", e);
            }
        };

        let Some(payload) = packet.payload() else {
            continue
        };

        data = payload.data().clone();

        
        println!("Payload bytes: {:02X?}", data);

        if search.search_in(&data).is_some() {
            break
        }
    }

    println!("Payload bytes: {:02X?}", data);
}