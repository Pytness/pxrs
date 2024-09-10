use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::mem::size_of;
use std::ptr;

use crate::types::PxBlocks;
use crate::types::{PxFieldInfo, PxHeader};

// Assume PxHeader, PxFieldInfo, px_records, PxBlocks, etc. are defined similarly to the previous structs in Rust.

// Helper function to read little-endian values from a byte slice.
fn copy_from_le<T: Default + Copy>(dst: &mut T, src: &[u8], size: usize) {
    let mut buf: [u8; 8] = [0; 8]; // Adjust for the largest expected type (like u64)
    buf[..size].copy_from_slice(&src[..size]);
    *dst = unsafe { ptr::read(buf.as_ptr() as *const T) };
}

// Parses the header from unp_head into the PxHeader struct.
fn parse_header(unp_head: &[u8], header: &mut PxHeader) {
    let mut i = 0;

    macro_rules! head_copy {
        ($x:ident) => {
            copy_from_le(&mut header.$x, &unp_head[i..], size_of::<PxHeader>());
            i += size_of::<PxHeader>();
        };
    }

    head_copy!(record_size);
    head_copy!(header_size);
    head_copy!(file_type);
    head_copy!(max_table_size);
    head_copy!(num_records);
    head_copy!(used_blocks);
    head_copy!(file_blocks);
    head_copy!(first_block);
    head_copy!(last_block);
    head_copy!(dummy_1);
    head_copy!(modified_flags1);
    head_copy!(index_field_number);
    head_copy!(primary_index_workspace);
    head_copy!(dummy_2);
    head_copy!(index_root_block);
    head_copy!(index_levels);
    head_copy!(num_fields);
    head_copy!(primary_key_fields);
    head_copy!(encryption1);
    head_copy!(sort_order);
    head_copy!(modified_flags2);
    head_copy!(dummy_5);
    head_copy!(change_count1);
    head_copy!(change_count2);
    head_copy!(dummy_6);
    head_copy!(table_name_ptr);
    head_copy!(field_info);
    head_copy!(write_protected);
    head_copy!(file_version_id);
    head_copy!(max_blocks);
    head_copy!(dummy_7);
    head_copy!(aux_passwords);
    head_copy!(dummy_8);
    head_copy!(crypt_info_start);
    head_copy!(crypt_info_end);
    head_copy!(dummy_9);
    head_copy!(auto_inc);
    head_copy!(dummy_a);
    head_copy!(index_update_required);
    head_copy!(dummy_b);
    head_copy!(dummy_c);
    head_copy!(ref_integrity);
    head_copy!(dummy_d);
}

fn parse_header_v4(unp_head: &[u8], header: &mut PxHeader) {
    let mut i = 0;

    macro_rules! head_copy {
        ($x:ident) => {
            copy_from_le(&mut header.$x, &unp_head[i..], size_of::<PxHeader>());
            i += size_of::<PxHeader>();
        };
    }

    head_copy!(file_version_id2);
    head_copy!(file_version_id3);
    head_copy!(encryption2);
    head_copy!(file_update_time);
    head_copy!(hi_field_id);
    head_copy!(hi_field_id_info);
    head_copy!(sometimes_num_fields);
    head_copy!(dos_global_code_page);
    head_copy!(dummy_e);
    head_copy!(change_count4);
    head_copy!(dummy_f);
    head_copy!(dummy_10);
}

// Check if the header is supported based on fileVersionID and fileType
fn is_header_supported(header: &PxHeader) -> bool {
    match header.file_version_id {
        0x03..=0x0c => (),
        _ => {
            eprintln!("Unknown Fileversion ID");
            return false;
        }
    }

    match header.file_type {
        0x00..=0x08 => (),
        _ => {
            eprintln!("Unknown FileType ID");
            return false;
        }
    }

    if header.num_records > 0 && header.first_block != 1 {
        eprintln!(
            "Warning: numRecords > 0 ({}) && firstBlock != 1 ({})",
            header.num_records, header.first_block
        );
    }

    true
}

// Parses the entire header from the file.
pub fn parse_complete_header(fd: &mut File, header: &mut PxHeader) -> Result<Vec<PxFieldInfo>> {
    let mut unp_head = [0u8; 0x58];
    fd.read_exact(&mut unp_head)?;
    parse_header(&unp_head, header);

    if !is_header_supported(header) {
        eprintln!("Unsupported header");
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unsupported header",
        ));
    }

    if header.file_version_id >= 0x05
        && header.file_type != 0x01
        && header.file_type != 0x04
        && header.file_type != 0x07
    {
        let mut unp_head4 = [0u8; 0x20];
        fd.read_exact(&mut unp_head4)?;
        parse_header_v4(&unp_head4, header);
    }

    let mut fields = vec![];

    for _ in 0..header.num_fields {
        let mut d = [0u8; 2];
        fd.read_exact(&mut d)?;

        let field_info: PxFieldInfo = PxFieldInfo {
            name: "asdf".to_string().into_bytes().try_into().unwrap(),
            field_type: d[0] as i32,
            size: d[1] as i32,
        };
        fields.push(field_info);
    }

    // Read the table name
    let mut table_name = [0u8; 79];
    fd.read_exact(&mut table_name)?;
    header.table_name.copy_from_slice(&table_name);

    Ok(fields)
}

// Assuming PxBlocks and PxFieldInfo structs exist
pub fn parse_blocks(fd: &mut File, header: &PxHeader) -> Result<Vec<PxBlocks>> {
    let mut blocks = Vec::with_capacity(header.file_blocks as usize);
    let mut block_buf = vec![0u8; 0x400 * header.max_table_size as usize];

    while fd.read(&mut block_buf)? > 0 {
        let mut block_index = 0;

        let mut next_block: u32 = 0;
        let mut prev_block: u32 = 0;
        let num_recs_in_block: u32 = 0;

        copy_from_le(&mut next_block, &block_buf[block_index..], size_of::<u16>());
        block_index += size_of::<u16>();
        copy_from_le(&mut prev_block, &block_buf[block_index..], size_of::<u16>());

        let block = PxBlocks {
            prev_block: prev_block as i32,
            next_block: next_block as i32,
            num_recs_in_block: num_recs_in_block as i32,
            records: ptr::null_mut(),
        };

        // Continue parsing the block...
        blocks.push(block);
    }

    Ok(blocks)
}
