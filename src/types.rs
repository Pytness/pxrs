use byte::ctx::{Str, NULL};
use byte::*;
use std::fmt::Display;
use std::io::Read;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

use self::ctx::Endian;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u8);

impl Version {
    pub fn is_supported(&self) -> bool {
        matches!(self.0, 0x03..=0x0c)
    }
}

impl<'a> TryRead<'a, Endian> for Version {
    fn try_read(bytes: &'a [u8], ctx: Endian) -> Result<(Self, usize)> {
        let (version, size) = u8::try_read(bytes, ctx)?;
        Ok((Version(version), size))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version = match self.0 {
            0x03 => "3.0",
            0x04 => "3.5",
            0x05..=0x09 => "4.x",
            0x0a | 0x0b => "5.x",
            0x0c => "7.x",
            _ => "Unknown",
        };

        write!(f, "{} | {:02x}", version, self.0)
    }
}

#[derive(Debug, Clone)]
pub struct PxHeader {
    pub record_size: i16,             // 0x00: signed short
    pub header_size: i16,             // 0x02: signed short
    pub file_type: u8,                // 0x04: unsigned char
    pub max_table_size: u8,           // 0x05: unsigned char
    pub num_records: u32,             // 0x06: unsigned int
    pub used_blocks: u16,             // 0x0a: unsigned short
    pub file_blocks: u16,             // 0x0c: unsigned short
    pub first_block: u16,             // 0x0e: unsigned short
    pub last_block: u16,              // 0x10: unsigned short
    pub dummy_1: u16,                 // 0x12: unsigned short
    pub modified_flags1: u8,          // 0x14: unsigned char
    pub index_field_number: u8,       // 0x15: unsigned char
    pub primary_index_workspace: u32, // 0x16: unsigned int (void*)
    pub dummy_2: u32,                 // 0x1a: unsigned int (void*)
    pub index_root_block: u16,        // 0x1e: unsigned short
    pub index_levels: u8,             // 0x20: unsigned char
    pub num_fields: i16,              // 0x21: signed short
    pub primary_key_fields: i16,      // 0x23: signed short
    pub encryption1: u32,             // 0x25: unsigned int
    pub sort_order: u8,               // 0x29: unsigned char
    pub modified_flags2: u8,          // 0x2a: unsigned char
    pub dummy_5: u16,                 // 0x2b: unsigned short
    pub change_count1: u8,            // 0x2d: unsigned char
    pub change_count2: u8,            // 0x2e: unsigned char
    pub dummy_6: u8,                  // 0x2f: unsigned char
    pub table_name_ptr: u32,          // 0x30: unsigned int (char**)
    pub field_info: u32,              // 0x34: unsigned int (void*)
    pub write_protected: u8,          // 0x38: unsigned char
    pub file_version_id: Version,     // 0x39: unsigned char
    pub max_blocks: u16,              // 0x3a: unsigned short
    pub dummy_7: u8,                  // 0x3c: unsigned char
    pub aux_passwords: u8,            // 0x3d: unsigned char
    pub dummy_8: u16,                 // 0x3e: unsigned short
    pub crypt_info_start: u32,        // 0x40: unsigned int (void*)
    pub crypt_info_end: u32,          // 0x44: unsigned int (void*)
    pub dummy_9: u8,                  // 0x48: unsigned char
    pub auto_inc: u32,                // 0x49: unsigned int
    pub dummy_a: u16,                 // 0x4d: unsigned short
    pub index_update_required: u8,    // 0x4f: unsigned char
    pub dummy_b: u32,                 // 0x50: unsigned int
    pub dummy_c: u8,                  // 0x54: unsigned char
    pub ref_integrity: u8,            // 0x55: unsigned char
    pub dummy_d: u16,                 // 0x56: unsigned short
    pub file_version_id2: u16,        // 0x58: unsigned short
    pub file_version_id3: u16,        // 0x5a: unsigned short
    pub encryption2: u32,             // 0x5c: unsigned int
    pub file_update_time: u32,        // 0x60: unsigned int
    pub hi_field_id: u16,             // 0x64: unsigned short
    pub hi_field_id_info: u16,        // 0x66: unsigned short
    pub sometimes_num_fields: u16,    // 0x68: unsigned short
    pub dos_global_code_page: u16,    // 0x6a: unsigned short
    pub dummy_e: u32,                 // 0x6c: unsigned int
    pub change_count4: u16,           // 0x70: unsigned short
    pub dummy_f: u32,                 // 0x72: unsigned int
    pub dummy_10: u16,                // 0x76: unsigned short
    pub table_name: String,           // ----: char[79]
}

impl Default for PxHeader {
    fn default() -> Self {
        unsafe {
            let mut header = MaybeUninit::<PxHeader>::uninit();
            let ptr: *mut PxHeader = header.as_mut_ptr();

            addr_of_mut!((*ptr).table_name).write(String::new());

            header.assume_init()
        }
    }
}

impl PxHeader {
    pub fn from_reader(reader: &mut dyn Read) -> std::io::Result<Self> {
        let mut offset = 0;

        let mut buffer = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buffer)?;

        let mut header = PxHeader::default();

        macro_rules! read_field {
            ($field:ident) => {
                header.$field = buffer.read_with(&mut offset, BE).unwrap();
            };
        }

        read_field!(record_size);
        read_field!(header_size);
        read_field!(file_type);
        read_field!(max_table_size);
        read_field!(num_records);
        read_field!(used_blocks);
        read_field!(file_blocks);
        read_field!(first_block);
        read_field!(last_block);
        read_field!(dummy_1);
        read_field!(modified_flags1);
        read_field!(index_field_number);
        read_field!(primary_index_workspace);
        read_field!(dummy_2);
        read_field!(index_root_block);
        read_field!(index_levels);
        read_field!(num_fields);
        read_field!(primary_key_fields);
        read_field!(encryption1);
        read_field!(sort_order);
        read_field!(modified_flags2);
        read_field!(dummy_5);
        read_field!(change_count1);
        read_field!(change_count2);
        read_field!(dummy_6);
        read_field!(table_name_ptr);
        read_field!(field_info);
        read_field!(write_protected);
        read_field!(file_version_id);
        read_field!(max_blocks);
        read_field!(dummy_7);
        read_field!(aux_passwords);
        read_field!(dummy_8);
        read_field!(crypt_info_start);
        read_field!(crypt_info_end);
        read_field!(dummy_9);
        read_field!(auto_inc);
        read_field!(dummy_a);
        read_field!(index_update_required);
        read_field!(dummy_b);
        read_field!(dummy_c);
        read_field!(ref_integrity);
        read_field!(dummy_d);
        read_field!(file_version_id2);
        read_field!(file_version_id3);
        read_field!(encryption2);
        read_field!(file_update_time);
        read_field!(hi_field_id);
        read_field!(hi_field_id_info);
        read_field!(sometimes_num_fields);
        read_field!(dos_global_code_page);
        read_field!(dummy_e);
        read_field!(change_count4);
        read_field!(dummy_f);
        read_field!(dummy_10);

        header.table_name = buffer
            .read_with::<&str>(&mut offset, Str::Delimiter(NULL))
            .unwrap_or_default()
            .to_string();

        Ok(header)
    }
}

impl Display for PxHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "File-Version: {}", self.file_version_id)?;
        writeln!(
            f,
            "Filetype: {}",
            match self.file_type {
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
        )?;
        writeln!(f, "Tablename: {}", &self.table_name)?;
        writeln!(
            f,
            "Sort-Order: {}",
            match self.sort_order {
                0x00 => "ASCII",
                0xb7 => "International",
                0x82 | 0xe6 => "Norwegian/Danish",
                0x0b => "Swedish/Finnish",
                0x5d => "Spanish",
                0x62 => "PDX ANSI intl",
                _ => "Unknown",
            }
        )?;
        writeln!(
            f,
            "Write-Protection: {}",
            match self.write_protected {
                0x00 => "off",
                0x01 => "on",
                _ => "Unknown",
            }
        )?;

        if self.file_version_id.0 >= 0x05
            && self.file_type != 0x01
            && self.file_type != 0x04
            && self.file_type != 0x07
        {
            writeln!(
                f,
                "Codepage: {}",
                match self.dos_global_code_page {
                    0x01b5 => "United States",
                    0x04e4 => "Spain",
                    _ => "Unknown",
                }
            )?;
        }

        writeln!(f, "Number of Blocks: {}", self.file_blocks)?;
        writeln!(f, "Used Blocks: {}", self.used_blocks)?;
        writeln!(f, "First Block: {}", self.first_block)?;
        writeln!(f, "Number of Records: {}", self.num_records)?;
        writeln!(f, "Max. Tablesize: {}", self.max_table_size)?;
        writeln!(f, "Recordsize: {}", self.record_size)?;
        writeln!(f, "Number of fields: {}", self.num_fields)?;

        if self.file_type == 0x01 {
            writeln!(f, "Index-root: {}", self.index_root_block)?;
            writeln!(f, "Index-levels: {}", self.index_levels)?;
        }

        Ok(())
    }
}

// Field information structure

#[repr(C)]
#[derive(Debug)]
pub struct PxFieldInfo {
    pub name: [u8; 80],  // char[80]
    pub field_type: i32, // int
    pub size: i32,       // int
}

type PxRecords = *const u8;

#[repr(C)]
pub struct PxBlocks {
    pub prev_block: i32,         // int
    pub next_block: i32,         // int
    pub num_recs_in_block: i32,  // int
    pub records: *mut PxRecords, // px_records*
}

#[repr(C)]

pub struct MbType2Pointer {
    pub type_: u8,
    pub size_div_4k: u16,
    pub length: u32,
    pub mod_count: u16,
}

// Field types constants
pub const PX_FIELD_TYPE_ALPHA: u8 = 0x01;
pub const PX_FIELD_TYPE_DATE: u8 = 0x02;
pub const PX_FIELD_TYPE_SHORT_INT: u8 = 0x03;
pub const PX_FIELD_TYPE_LONG_INT: u8 = 0x04;
pub const PX_FIELD_TYPE_CURRENCY: u8 = 0x05;
pub const PX_FIELD_TYPE_NUMBER: u8 = 0x06;
pub const PX_FIELD_TYPE_LOGICAL: u8 = 0x09;
pub const PX_FIELD_TYPE_MEMO_BLOB: u8 = 0x0c;
pub const PX_FIELD_TYPE_BIN_BLOB: u8 = 0x0d;
pub const PX_FIELD_TYPE_DUNNO: u8 = 0x0e;
pub const PX_FIELD_TYPE_GRAPHIC: u8 = 0x10;
pub const PX_FIELD_TYPE_TIME: u8 = 0x14;
pub const PX_FIELD_TYPE_TIMESTAMP: u8 = 0x15;
pub const PX_FIELD_TYPE_INCREMENTAL: u8 = 0x16;
pub const PX_FIELD_TYPE_BCD: u8 = 0x17;

// File types constants
pub const PX_FILETYPE_DB_INDEXED: u8 = 0x00;
pub const PX_FILETYPE_PX: u8 = 0x01;
pub const PX_FILETYPE_DB_NOT_INDEXED: u8 = 0x02;
pub const PX_FILETYPE_XNN_NON_INC: u8 = 0x03;
pub const PX_FILETYPE_YNN: u8 = 0x04;
pub const PX_FILETYPE_XNN_INC: u8 = 0x05;
pub const PX_FILETYPE_XGN_NON_INC: u8 = 0x06;
pub const PX_FILETYPE_YGN: u8 = 0x07;
pub const PX_FILETYPE_XGN_INC: u8 = 0x08;
