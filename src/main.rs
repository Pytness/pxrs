mod convert;
mod parse;
mod types;

use clap::Parser;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

use types::{PxFieldInfo, PxHeader};

fn show_header_info(header: &PxHeader) {
    println!(
        "{:<20}Paradox {}",
        "File-Version:",
        match header.file_version_id {
            0x03 => "3.0",
            0x04 => "3.5",
            0x05..=0x09 => "4.x",
            0x0a | 0x0b => "5.x",
            0x0c => "7.x",
            _ => "Unknown",
        }
    );
    println!(
        "{:<20}{}",
        "Filetype:",
        match header.file_type {
            0x00 => "indexed .DB",
            0x01 => "primary index .PX",
            0x02 => "non indexed .DB",
            0x03 => "non-incrementing secondary index .Xnn",
            0x04 => "secondary index .Ynn (inc/non-inc)",
            0x05 => "incrementing secondary index .Xnn",
            0x06 => "non-incrementing secondary index .XGn",
            0x07 => "secondary index .YGn (inc/non-inc)",
            0x08 => "incrementing secondary index .XGn",
            _ => "Unknown",
        }
    );
    println!(
        "{:<20}{}",
        "Tablename:",
        String::from_utf8_lossy(&header.table_name)
    );
    println!(
        "{:<20}{}",
        "Sort-Order:",
        match header.sort_order {
            0x00 => "ASCII",
            0xb7 => "International",
            0x82 | 0xe6 => "Norwegian/Danish",
            0x0b => "Swedish/Finnish",
            0x5d => "Spanish",
            0x62 => "PDX ANSI intl",
            _ => "Unknown",
        }
    );
    println!(
        "{:<20}{}",
        "Write-Protection:",
        match header.write_protected {
            0x00 => "off",
            0x01 => "on",
            _ => "Unknown",
        }
    );

    if header.file_version_id >= 0x05
        && header.file_type != 0x01
        && header.file_type != 0x04
        && header.file_type != 0x07
    {
        println!(
            "{:<20}{}",
            "Codepage:",
            match header.dos_global_code_page {
                0x01b5 => "United States",
                0x04e4 => "Spain",
                _ => "Unknown",
            }
        );
    }

    println!("{:<20}{}", "Number of Blocks:", header.file_blocks);
    println!("{:<20}{}", "Used Blocks:", header.used_blocks);
    println!("{:<20}{}", "First Block:", header.first_block);
    println!("{:<20}{}", "Number of Records:", header.num_records);
    println!("{:<20}{}", "Max. Tablesize:", header.max_table_size);
    println!("{:<20}{}", "Recordsize:", header.record_size);

    if header.file_type == 0x01 {
        println!("{:<20}{}", "Index-root:", header.index_root_block);
        println!("{:<20}{}", "Index-levels:", header.index_levels);
    }
}

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

    show_header_info(&header);

    Ok(())
}
