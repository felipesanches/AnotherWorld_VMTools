//! Reader for the 3DO Opera filesystem (the proprietary
//! filesystem 3DO CD-ROMs use).
//!
//! Reads from a CD-ROM Mode 1 `.bin` image (2352 bytes per sector,
//! of which 2048 are usable data after the 12-byte sync + 4-byte
//! header). The filesystem itself is well-described in the 3DO
//! Portfolio docs and various RE writeups.
//!
//! Layout summary:
//!
//! - Sector 0 holds the volume header: `recordType=0x01`, sync
//!   `5A 5A 5A 5A 5A`, then volume label, block_size, root-dir
//!   pointer, plus `last_avatar`+1 redundant copies of the root
//!   directory's first block (avatars are mirror copies on the
//!   physical media for reliability — we only need one).
//!
//! - Each directory block starts with a 20-byte header (linkage +
//!   first-free-byte / first-entry-byte offsets) followed by
//!   variable-length entries. Each entry has a 68-byte fixed
//!   prefix (flags, id, type, block_size, byte_count, block_count,
//!   burst, gap, 32-byte name, last_avatar) followed by the avatar
//!   list (`last_avatar + 1` u32 LBAs).
//!
//! - File data is read by walking the entry's avatar list — each
//!   avatar is the LBA of one extent of the file (the same data
//!   replicated for redundancy). For our purposes, reading
//!   avatar 0 is enough.
//!
//! No write support; no support for "blessed" volume markers,
//! catapult areas, or anything beyond plain file listing + read.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SECTOR_RAW: usize = 2352;
const SECTOR_DATA: usize = 2048;
/// Mode 1 sector data starts after 12-byte sync + 4-byte header.
const SECTOR_DATA_OFFSET: usize = 16;

const VOL_RECORD_TYPE: u8 = 0x01;
const VOL_SYNC: &[u8; 5] = &[0x5A, 0x5A, 0x5A, 0x5A, 0x5A];

/// Directory entry "type" codes from the Opera spec. The four-byte
/// code is stored big-endian.
const TYPE_DIRECTORY: &[u8; 4] = b"*dir";

#[derive(Debug)]
pub enum OperaError {
    Io(io::Error),
    BadStructure(String),
}

impl From<io::Error> for OperaError {
    fn from(e: io::Error) -> Self {
        OperaError::Io(e)
    }
}

impl std::fmt::Display for OperaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperaError::Io(e) => write!(f, "io: {e}"),
            OperaError::BadStructure(s) => write!(f, "bad Opera structure: {s}"),
        }
    }
}

impl std::error::Error for OperaError {}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub typ: [u8; 4],
    pub byte_count: u32,
    pub block_size: u32,
    /// LBA of the first avatar (redundant copy) for this entry.
    pub first_block: u32,
    pub flags: u32,
    pub id: u32,
}

impl DirEntry {
    pub fn is_directory(&self) -> bool {
        &self.typ == TYPE_DIRECTORY
    }
    pub fn type_str(&self) -> String {
        String::from_utf8_lossy(&self.typ).into_owned()
    }
}

pub struct OperaImage {
    raw: Vec<u8>,
}

impl OperaImage {
    pub fn from_bytes(raw: Vec<u8>) -> Result<Self, OperaError> {
        if raw.len() < SECTOR_RAW {
            return Err(OperaError::BadStructure(format!(
                "image is shorter than one CD-ROM Mode 1 sector ({} bytes)",
                raw.len()
            )));
        }
        let img = Self { raw };
        let vh = img.sector(0)?;
        if vh[0] != VOL_RECORD_TYPE {
            return Err(OperaError::BadStructure(format!(
                "sector 0 record_type = 0x{:02x}, expected 0x01",
                vh[0]
            )));
        }
        if &vh[1..6] != VOL_SYNC {
            return Err(OperaError::BadStructure(format!(
                "sector 0 sync bytes = {:02x?}, expected 5A 5A 5A 5A 5A",
                &vh[1..6]
            )));
        }
        Ok(img)
    }

    pub fn from_path(path: &Path) -> Result<Self, OperaError> {
        Ok(Self::from_bytes(fs::read(path)?)?)
    }

    /// Returns the 2048 bytes of usable data from CD-ROM sector `n`.
    pub fn sector(&self, n: u32) -> Result<&[u8], OperaError> {
        let off = (n as usize) * SECTOR_RAW + SECTOR_DATA_OFFSET;
        if off + SECTOR_DATA > self.raw.len() {
            return Err(OperaError::BadStructure(format!(
                "sector {n} is past end of image"
            )));
        }
        Ok(&self.raw[off..off + SECTOR_DATA])
    }

    /// Volume label (max 32 chars).
    pub fn label(&self) -> Result<String, OperaError> {
        let vh = self.sector(0)?;
        Ok(read_cstr(&vh[40..40 + 32]))
    }

    /// LBA of the root directory's first block.
    pub fn root_dir_block(&self) -> Result<u32, OperaError> {
        let vh = self.sector(0)?;
        // Avatar list starts at offset 100 (after volume_id .. root_dir_last_avatar).
        Ok(read_be_u32(&vh[100..104]))
    }

    /// List directory entries in the directory whose first block is
    /// at LBA `dir_block` (the start of avatar 0).
    pub fn list_dir(&self, dir_block: u32) -> Result<Vec<DirEntry>, OperaError> {
        let avatar_start = dir_block;
        let mut block = dir_block;
        let mut out = Vec::new();
        loop {
            let buf = self.sector(block)?;
            // Directory block header (20 bytes):
            //   0..3   next_block — logical block index of next dir block within
            //          this directory (0xFFFFFFFF = no more). Physical LBA is
            //          `avatar_start + next_block`.
            //   4..7   prev_block — logical block index of previous dir block,
            //          or 0xFFFFFFFF for the first block.
            //   8..11  flags
            //   12..15 first_free (offset of first free byte in this block)
            //   16..19 first_entry (offset of first entry in this block, usually 0x14)
            let next_block = read_be_u32(&buf[0..4]);
            let first_free = read_be_u32(&buf[12..16]) as usize;
            let first_entry = read_be_u32(&buf[16..20]) as usize;

            let mut pos = first_entry;
            while pos + 68 <= first_free.min(SECTOR_DATA) {
                let entry_start = pos;
                let flags = read_be_u32(&buf[entry_start..entry_start + 4]);
                let id = read_be_u32(&buf[entry_start + 4..entry_start + 8]);
                let mut typ = [0u8; 4];
                typ.copy_from_slice(&buf[entry_start + 8..entry_start + 12]);
                let block_size = read_be_u32(&buf[entry_start + 12..entry_start + 16]);
                let byte_count = read_be_u32(&buf[entry_start + 16..entry_start + 20]);
                let _block_count = read_be_u32(&buf[entry_start + 20..entry_start + 24]);
                let _burst = read_be_u32(&buf[entry_start + 24..entry_start + 28]);
                let _gap = read_be_u32(&buf[entry_start + 28..entry_start + 32]);
                let name = read_cstr(&buf[entry_start + 32..entry_start + 32 + 32]);
                let last_avatar = read_be_u32(&buf[entry_start + 64..entry_start + 68]) as usize;
                let avatar_list_off = entry_start + 68;
                let first_block = if avatar_list_off + 4 <= buf.len() {
                    read_be_u32(&buf[avatar_list_off..avatar_list_off + 4])
                } else {
                    0
                };
                let entry_size = 68 + 4 * (last_avatar + 1);
                if name.is_empty() && byte_count == 0 {
                    break;
                }
                out.push(DirEntry {
                    name,
                    typ,
                    byte_count,
                    block_size,
                    first_block,
                    flags,
                    id,
                });
                pos = entry_start + entry_size;
            }

            if next_block == 0xFFFF_FFFF {
                break;
            }
            block = avatar_start + next_block;
        }
        Ok(out)
    }

    /// Read the raw bytes of a file entry, walking its avatar(s).
    /// We only follow avatar 0 — Opera replicates file data across
    /// avatars for reliability; one copy is enough.
    pub fn read_file(&self, entry: &DirEntry) -> Result<Vec<u8>, OperaError> {
        if entry.is_directory() {
            return Err(OperaError::BadStructure(format!(
                "{:?} is a directory, not a file",
                entry.name
            )));
        }
        let mut out = Vec::with_capacity(entry.byte_count as usize);
        let mut block = entry.first_block;
        let block_size = entry.block_size.max(1);
        let mut remaining = entry.byte_count as usize;
        while remaining > 0 {
            let buf = self.sector(block)?;
            let take = remaining.min(buf.len()).min(block_size as usize);
            out.extend_from_slice(&buf[..take]);
            remaining -= take;
            block += 1;
        }
        Ok(out)
    }
}

/// Recursively extract every file under the root directory of
/// `bin_path` into `out_dir`, mirroring the on-disc directory tree.
pub fn extract_root_to_dir(bin_path: &Path, out_dir: &Path) -> Result<Vec<PathBuf>, OperaError> {
    let img = OperaImage::from_path(bin_path)?;
    let root_block = img.root_dir_block()?;
    let mut written = Vec::new();
    fs::create_dir_all(out_dir)?;
    extract_dir_recursive(&img, root_block, out_dir, &mut written)?;
    Ok(written)
}

fn extract_dir_recursive(
    img: &OperaImage,
    dir_block: u32,
    out_dir: &Path,
    written: &mut Vec<PathBuf>,
) -> Result<(), OperaError> {
    let entries = img.list_dir(dir_block)?;
    for e in entries {
        // Skip Opera-internal "label" / synthetic entries — they are not
        // files in the user-visible filesystem.
        if &e.typ == b"*lbl" {
            continue;
        }
        if e.is_directory() {
            let sub = out_dir.join(&e.name);
            fs::create_dir_all(&sub)?;
            extract_dir_recursive(img, e.first_block, &sub, written)?;
        } else {
            let bytes = img.read_file(&e)?;
            let path = out_dir.join(&e.name);
            fs::write(&path, bytes)?;
            written.push(path);
        }
    }
    Ok(())
}

fn read_be_u32(buf: &[u8]) -> u32 {
    u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]])
}

fn read_cstr(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).trim().to_owned()
}
