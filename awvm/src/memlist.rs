//! Reader for `memlist.bin` — the per-release table that catalogues
//! every resource in the `bank<NN>` files.
//!
//! Format mirrors `releases/common_data/banks2resources.py`:
//! 20 bytes per entry, big-endian fields, list ends when an entry's
//! `bankOffset` reads as `0xFFFFFFFF`.

use std::io::{self, Read, Seek, SeekFrom};

/// One row in `memlist.bin`. Field layout follows the upstream
/// Python reference; a few "unknown" 16-bit fields are preserved
/// in case a future analysis discovers a use for them.
#[derive(Debug, Clone)]
pub struct MemEntry {
    pub state: u8,
    pub typ: u8,
    pub unknown_0x02: u16,
    pub unknown_0x04: u16,
    pub rank_num: u8,
    pub bank_id: u8,
    pub bank_offset: u32,
    pub unknown_0x0c: u16,
    pub packed_size: u16,
    pub unknown_0x10: u16,
    pub size: u16,
}

/// `0xFFFFFFFF` in `bank_offset` marks the end of the list. Matches
/// `banks2resources.py:load_memlist`.
const SENTINEL_BANK_OFFSET: u32 = 0xFFFF_FFFF;

/// Parse all entries from a `memlist.bin` payload.
pub fn parse(bytes: &[u8]) -> io::Result<Vec<MemEntry>> {
    let mut cursor = io::Cursor::new(bytes);
    let mut out = Vec::new();
    let mut i = 0u64;
    loop {
        cursor.seek(SeekFrom::Start(20 * i))?;
        let mut entry_buf = [0u8; 20];
        if cursor.read(&mut entry_buf)? != 20 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("memlist truncated mid-entry at index {i}"),
            ));
        }
        let entry = decode_entry(&entry_buf);
        if entry.bank_offset == SENTINEL_BANK_OFFSET {
            return Ok(out);
        }
        out.push(entry);
        i += 1;
    }
}

fn decode_entry(b: &[u8; 20]) -> MemEntry {
    MemEntry {
        state: b[0],
        typ: b[1],
        unknown_0x02: u16::from_be_bytes([b[2], b[3]]),
        unknown_0x04: u16::from_be_bytes([b[4], b[5]]),
        rank_num: b[6],
        bank_id: b[7],
        bank_offset: u32::from_be_bytes([b[8], b[9], b[10], b[11]]),
        unknown_0x0c: u16::from_be_bytes([b[12], b[13]]),
        packed_size: u16::from_be_bytes([b[14], b[15]]),
        unknown_0x10: u16::from_be_bytes([b[16], b[17]]),
        size: u16::from_be_bytes([b[18], b[19]]),
    }
}

/// The string form of an `entry.typ` used by the upstream Python
/// `banks2resources.py:get_type`. Matches verbatim including the
/// `POLY_CINEM` truncation (only six known types; everything else is
/// rendered as `UNKNOWN(<n>)`).
pub fn type_name(typ: u8) -> String {
    match typ {
        0 => "SOUND".to_owned(),
        1 => "MUSIC".to_owned(),
        2 => "POLY_ANIM".to_owned(),
        3 => "PALETTE".to_owned(),
        4 => "BYTECODE".to_owned(),
        5 => "POLY_CINEM".to_owned(),
        _ => format!("UNKNOWN({typ})"),
    }
}
