mod convert;
mod parse;
mod types;

use clap::Parser;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

use types::{PxFieldInfo, PxHeader};

fn show_field_info(field_info: &PxFieldInfo) {
    println!(
        "Name: {:<20}Type: {:<15}Size: {}",
        String::from_utf8_lossy(&field_info.name),
        match field_info.field_type {
            0x01 => "Alpha",
            0x02 => "Date",
            0x03 => "Short Integer",
            0x04 => "Long Integer",
            0x05 => "Currency",
            0x06 => "Number",
            0x0c => "Memo BLOB",
            0x10 => "Graphic",
            0x0d => "BLOB",
            0x09 => "Logical",
            0x14 => "Time",
            0x15 => "Timestamp",
            0x16 => "Incremental",
            _ => "Unknown",
        },
        field_info.size
    );
}

#[derive(Parser)]
#[command(name = "PXInfo")]
#[command(version = "1.0")]
#[command(about = "Displays header information of a Paradox database file")]
struct Cli {
    #[arg(short, long, value_name = "FILE", help = "Sets the input file to use")]
    filename: String,
}

fn main() -> io::Result<()> {
    let matches = Cli::parse();

    let filename = matches.filename;
    let path = Path::new(&filename);

    if !path.exists() {
        eprintln!("File '{}' does not exist", filename);
        std::process::exit(1);
    }

    let file = File::open(path)?;

    let mut reader = BufReader::new(file);

    // Dummy header for demonstration purposes.
    let header = PxHeader::from_reader(&mut reader).expect("Failed to read header");

    println!("{}", header);

    Ok(())
}
