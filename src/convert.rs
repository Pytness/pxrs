use crate::types::*;
use std::ffi::CStr;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::mem;
use std::ptr;
use std::time::SystemTime;

// Helper functions for endian conversions
fn copy_from_be<T: Default + Copy>(dst: &mut T, src: &[u8], len: usize) {
    let mut buf: [u8; 8] = [0; 8];
    buf[..len].copy_from_slice(&src[..len]);
    *dst = unsafe { ptr::read(buf.as_ptr() as *const T) };
}

fn copy_from_le<T: Default + Copy>(dst: &mut T, src: &[u8], len: usize) {
    let mut buf: [u8; 8] = [0; 8];
    buf[..len].copy_from_slice(&src[..len]);
    *dst = unsafe { ptr::read(buf.as_ptr() as *const T) };
}

// Sign manipulation functions
fn fix_sign(dst: &mut [u8], len: usize) {
    dst[len - 1] &= 0x7f;
}

fn set_sign(dst: &mut [u8], len: usize) {
    dst[len - 1] |= 0x80;
}

// Convert PX number to long
fn px_to_long(number: u64, ret: &mut u64, field_type: u8) -> Result<(), &'static str> {
    let mut retval = 0u64;
    let s = number.to_le_bytes();
    let d = retval.to_le_bytes();

    match field_type {
        PX_FIELD_TYPE_LOGICAL => {
            copy_from_be(&mut retval, &s, 1);
            if s[0] & 0x80 != 0 {
                fix_sign(&mut retval.to_le_bytes(), 1);
            } else if retval == 0 {
                return Err("Value is null");
            } else {
                set_sign(&mut retval.to_le_bytes(), 1);
            }
        }
        PX_FIELD_TYPE_SHORT_INT => {
            copy_from_be(&mut retval, &s, 2);
            if s[0] & 0x80 != 0 {
                fix_sign(&mut retval.to_le_bytes(), 2);
            } else if retval == 0 {
                return Err("Value is null");
            } else {
                set_sign(&mut retval.to_le_bytes(), 2);
            }
        }
        PX_FIELD_TYPE_LONG_INT | PX_FIELD_TYPE_INCREMENTAL => {
            copy_from_be(&mut retval, &s, 4);
            if s[0] & 0x80 != 0 {
                fix_sign(&mut retval.to_le_bytes(), 4);
            } else if retval == 0 {
                return Err("Value is null");
            } else {
                set_sign(&mut retval.to_le_bytes(), 4);
            }
        }
        _ => return Err("Unsupported type"),
    }

    *ret = retval;
    Ok(())
}

// Convert PX number to double
fn px_to_double(number: u64, ret: &mut f64, field_type: u8) -> Result<(), &'static str> {
    let mut retval = 0f64;
    let s = number.to_le_bytes();
    let mut d = retval.to_le_bytes();

    match field_type {
        PX_FIELD_TYPE_CURRENCY | PX_FIELD_TYPE_NUMBER => {
            copy_from_be(&mut retval, &s, 8);
            if s[0] & 0x80 != 0 {
                fix_sign(&mut retval.to_le_bytes(), 8);
            } else if retval == 0.0 {
                return Err("Value is null");
            } else {
                // Apply fix for negative values
                d.iter_mut().for_each(|x| *x ^= 0xff);

                retval = unsafe { mem::transmute::<[u8; 8], f64>(d) };
            }
        }
        _ => return Err("Unsupported type"),
    }

    *ret = retval;
    Ok(())
}

// Convert PX number to time (tm structure)
fn px_to_tm(number: u64, tm: &mut libc::tm, field_type: u8) -> Result<(), &'static str> {
    let mut retval = 0u64;
    let s = number.to_le_bytes();
    let d = retval.to_le_bytes();

    match field_type {
        PX_FIELD_TYPE_DATE => {
            copy_from_be(&mut retval, &s, 4);
            if s[0] & 0x80 != 0 {
                fix_sign(&mut retval.to_le_bytes(), 4);
            } else if retval == 0 {
                return Err("Value is null");
            }
            // Date conversion logic (Y2K workaround)
            let jd = 719528 + retval - 1;
            let (y, m, d) = gdate(jd);
            tm.tm_year = y as i32 - 1900;
            tm.tm_mon = m as i32 - 1;
            tm.tm_mday = d as i32;
        }
        PX_FIELD_TYPE_TIME => {
            copy_from_be(&mut retval, &s, 4);
            if s[0] & 0x80 != 0 {
                fix_sign(&mut retval.to_le_bytes(), 4);
                retval /= 1000; // discard milliseconds
                tm.tm_sec = (retval % 60) as i32;
                retval /= 60;
                tm.tm_min = (retval % 60) as i32;
                tm.tm_hour = (retval / 60) as i32;
            } else if retval == 0 {
                return Err("Value is null");
            }
        }
        PX_FIELD_TYPE_TIMESTAMP => {
            copy_from_be(&mut retval, &s, 8);
            if s[0] & 0x80 != 0 {
                fix_sign(&mut retval.to_le_bytes(), 8);
                retval >>= 8;
                retval /= 500; // resolution of 1/500s
                let t = retval as i64 - 37603860709183;
                *tm = unsafe { *libc::gmtime(&t) };
            } else if retval == 0 {
                return Err("Value is null");
            }
        }
        _ => return Err("Unsupported type"),
    }

    Ok(())
}

// Memo handling - this function retrieves a memo blob from a file
fn px_memo_to_string(
    blob: &[u8],
    size: usize,
    blobname: Option<&str>,
) -> io::Result<Option<String>> {
    if size < 10 {
        return Ok(None);
    }

    let mut offset: u32 = 0;
    let mut length: u32 = 0;
    let mut mod_number: u16 = 0;
    let mut index: u8 = 0;

    copy_from_le(&mut offset, &blob[size - 10..], 4);
    copy_from_le(&mut length, &blob[size - 6..], 4);
    copy_from_le(&mut mod_number, &blob[size - 2..], 2);
    copy_from_le(&mut index, &blob[size - 10..], 1);

    offset &= 0xffffff00;

    if index == 0x00 {
        return Ok(None);
    }

    if let Some(blobname) = blobname {
        let mut file = File::open(blobname)?;

        if index == 0xff {
            // Type 02 block
            let mut header = [0u8; 9];
            file.seek(SeekFrom::Start(offset as u64))?;
            file.read_exact(&mut header)?;

            let mut idx = MbType2Pointer {
                type_: 0,
                size_div_4k: 0,
                length: 0,
                mod_count: 0,
            };
            copy_from_le(&mut idx.type_, &header[0..], 1);
            copy_from_le(&mut idx.size_div_4k, &header[1..], 2);
            copy_from_le(&mut idx.length, &header[3..], 4);
            copy_from_le(&mut idx.mod_count, &header[7..], 2);

            if idx.type_ != 0x02 || idx.length != length {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Type 02 blob length mismatch",
                ));
            }

            let mut string = vec![0u8; length as usize];
            file.read_exact(&mut string)?;

            return Ok(Some(String::from_utf8(string).unwrap_or_default()));
        } else {
            // Handle type 03 block here (similar to type 02 but with different logic)
            // Implement as per your specific logic needs
        }
    }

    Ok(None)
}

// Helper function for Julian date to Gregorian date conversion
fn gdate(jd: u64) -> (i32, i32, i32) {
    let mut jd = jd as i64 - 1721119;
    let j = (4 * jd - 1) / 146097;
    jd = (4 * jd - 1) % 146097;
    let t = jd / 4;

    jd = (4 * t + 3) / 1461;
    let t = (4 * t + 3) % 1461;
    let t = (t + 4) / 4;

    let m = (5 * t - 3) / 153;
    let t = (5 * t - 3) % 153;
    let t = (t + 5) / 5;

    let y = 100 * j + jd;

    if m < 10 {
        (y as i32, m as i32 + 3, t as i32)
    } else {
        (y as i32 + 1, m as i32 - 9, t as i32)
    }
}
