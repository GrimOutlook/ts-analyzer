use clap::Parser;
use ts_analyzer::reader::TSReader;
use std::{collections::HashSet, fs::File, io::BufReader, process::ExitCode};
use log::{debug, info};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Setup the verbose flag
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Get what video to scan
    #[arg(short, long)]
    path: String,
}

fn main() -> ExitCode {
    // Parse the arguments
    let args = Args::parse();
    let video = &args.path;
    
    // Initialize the logger
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!("Starting laser video sorter");

    // List of PIDs in the video
    let mut pids = HashSet::new();

    // Boilerplate to create a TSReader object
    let f = File::open(video).expect("Couldn't open file");
    let buf_reader = BufReader::new(f);
    let mut reader = TSReader::new(video, buf_reader).expect("Transport Stream file contains no SYNC bytes.");

    loop {
        // Check to see if any of the KLV data indicates that the laser is on
        let packet = match reader.next_packet() {
            Ok(packet) => packet,
            Err(e) => panic!("Could not get payload due to error: {}", e),
        };

        // If `None` is returned then we have finished reading the file.
        let Some(packet) = packet else {
            debug!("Finished reading file [{}]", video);
            break;
        };

        pids.insert(packet.header().pid());
    }

    let mut pids: Vec<u16> = Vec::from_iter(pids);
    pids.sort();

    println!("PIDs in video file [{}]:\n{:#?}", video, pids);

    return ExitCode::from(0);
}