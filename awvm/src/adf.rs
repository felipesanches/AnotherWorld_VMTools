//! Read files out of an Amiga Disk File (ADF) image.
//!
//! Supports the **OFS** (Old File System) family — `DOS\0`,
//! `DOS\2` (international), `DOS\4` (dirCache). Sufficient for the
//! Another World Amiga release (the AnotherWorld retro-presskit
//! ADFs are OFS), and the format the original Eric Chahi releases
//! were authored on.
//!
//! FFS support is intentionally not included yet — the file-data
//! layout differs (no per-block header, file-header pointer table is
//! used for chaining instead of `next_data`) — and no FFS fixture is
//! in the current scope. The reader returns a clear `BadStructure`
//! error rather than producing garbage on FFS input.
//!
//! Reference: <https://lclevy.free.fr/adflib/adf_info.html>.

use std::io;
use std::path::{Path, PathBuf};

const SECTOR_SIZE: usize = 512;
/// Standard rootblock sector for a 880-KiB DD floppy.
const ROOT_SECTOR: u32 = 880;

/// Block-type / secType identifiers from the Amiga DOS spec.
const T_HEADER: u32 = 2;
const T_DATA: u32 = 8;
const ST_FILE: i32 = -3;
const ST_ROOT: i32 = 1;
// ST_USERDIR (2) is sub-directory; not yet used (root-only extraction).

#[derive(Debug)]
pub enum AdfError {
    Io(io::Error),
    BadMagic,
    /// Image is FFS and we don't speak FFS yet.
    UnsupportedFfs,
    BadStructure(String),
}

impl From<io::Error> for AdfError {
    fn from(e: io::Error) -> Self {
        AdfError::Io(e)
    }
}

impl std::fmt::Display for AdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdfError::Io(e) => write!(f, "io error: {e}"),
            AdfError::BadMagic => write!(f, "not an ADF (bad bootblock magic)"),
            AdfError::UnsupportedFfs => {
                write!(f, "FFS-formatted ADF not supported (only OFS is)")
            }
            AdfError::BadStructure(s) => write!(f, "bad ADF structure: {s}"),
        }
    }
}

impl std::error::Error for AdfError {}

#[derive(Debug, Clone, Copy)]
pub enum FileSystem {
    /// Old File System (DOS\0 / DOS\2 / DOS\4).
    Ofs,
    /// Fast File System (DOS\1 / DOS\3 / DOS\5).
    Ffs,
}

/// Reader over a raw ADF byte buffer.
pub struct AdfReader {
    data: Vec<u8>,
    fs: FileSystem,
    root_sector: u32,
}

/// One extracted file: name (preserved as-is from the disk) and its
/// payload bytes.
pub struct ExtractedFile {
    pub name: String,
    pub data: Vec<u8>,
}

impl AdfReader {
    /// Build a reader from the raw bytes of an ADF image.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, AdfError> {
        if data.len() < SECTOR_SIZE * 2 {
            return Err(AdfError::BadStructure(
                "image is shorter than two sectors".into(),
            ));
        }
        if &data[0..3] != b"DOS" {
            return Err(AdfError::BadMagic);
        }
        // Bit 0 of the flag byte distinguishes OFS (0) from FFS (1).
        // Bits 1..2 toggle international / dirCache modes (we treat them as OFS).
        let fs = if data[3] & 1 == 0 {
            FileSystem::Ofs
        } else {
            FileSystem::Ffs
        };
        if matches!(fs, FileSystem::Ffs) {
            return Err(AdfError::UnsupportedFfs);
        }
        // The bootblock can record an alternate rootblock pointer at
        // offset 8 (BE u32), but it's commonly 0 on non-bootable disks
        // — fall back to the standard for DD floppies (sector 880).
        let bb_root = read_be_u32(&data, 8);
        let max_sector = (data.len() / SECTOR_SIZE) as u32;
        let root_sector = if bb_root != 0 && bb_root < max_sector {
            bb_root
        } else {
            ROOT_SECTOR
        };
        Ok(Self {
            data,
            fs,
            root_sector,
        })
    }

    /// Read the entire image from `path` and build a reader.
    pub fn from_path(path: &Path) -> Result<Self, AdfError> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(bytes)
    }

    pub fn filesystem(&self) -> FileSystem {
        self.fs
    }

    /// Return every file in the root directory (no recursion into
    /// subdirectories — they are skipped, but you can extend this if
    /// future fixtures need it).
    pub fn list_root_files(&self) -> Result<Vec<ExtractedFile>, AdfError> {
        let root = self.sector(self.root_sector)?;
        if read_be_i32(root, SECTOR_SIZE - 4) != ST_ROOT {
            return Err(AdfError::BadStructure(format!(
                "rootblock at sector {} is not ST_ROOT",
                self.root_sector
            )));
        }

        let mut out = Vec::new();
        for i in 0..72usize {
            let mut sec = read_be_u32(root, 24 + i * 4);
            // Walk the hash chain — collisions append via SECTOR_SIZE-16.
            while sec != 0 {
                let block = self.sector(sec)?;
                let sec_type = read_be_i32(block, SECTOR_SIZE - 4);
                let block_type = read_be_u32(block, 0);
                if block_type != T_HEADER {
                    return Err(AdfError::BadStructure(format!(
                        "expected T_HEADER at sector {sec}, got type {block_type}"
                    )));
                }
                let name = read_bcpl_name(block);
                if sec_type == ST_FILE {
                    let data = self.read_file_data(block)?;
                    out.push(ExtractedFile { name, data });
                }
                // Sub-directories (ST_USERDIR) are silently skipped for now.
                sec = read_be_u32(block, SECTOR_SIZE - 16);
            }
        }
        Ok(out)
    }

    /// Read `count`-byte file payload following an OFS file header's
    /// `first_data` pointer. The `next_data` link in each data block
    /// chains us to the next.
    fn read_file_data(&self, header: &[u8]) -> Result<Vec<u8>, AdfError> {
        let file_size = read_be_u32(header, SECTOR_SIZE - 188) as usize;
        let mut out = Vec::with_capacity(file_size);
        let mut sec = read_be_u32(header, 16); // first_data
        while sec != 0 {
            let block = self.sector(sec)?;
            let block_type = read_be_u32(block, 0);
            if block_type != T_DATA {
                return Err(AdfError::BadStructure(format!(
                    "expected T_DATA at sector {sec}, got type {block_type}"
                )));
            }
            let data_size = read_be_u32(block, 12) as usize;
            let next_data = read_be_u32(block, 16);
            if data_size > SECTOR_SIZE - 24 {
                return Err(AdfError::BadStructure(format!(
                    "OFS data block at sector {sec}: data_size {data_size} > 488"
                )));
            }
            out.extend_from_slice(&block[24..24 + data_size]);
            sec = next_data;
        }
        if out.len() != file_size {
            return Err(AdfError::BadStructure(format!(
                "file size mismatch: header says {file_size}, walked {} bytes",
                out.len()
            )));
        }
        Ok(out)
    }

    fn sector(&self, sector: u32) -> Result<&[u8], AdfError> {
        let offset = (sector as usize) * SECTOR_SIZE;
        if offset + SECTOR_SIZE > self.data.len() {
            return Err(AdfError::BadStructure(format!(
                "sector {sector} is past end of image"
            )));
        }
        Ok(&self.data[offset..offset + SECTOR_SIZE])
    }
}

/// Convenience: extract every file from an ADF into `out_dir`,
/// preserving on-disk filenames. Returns the list of paths written.
pub fn extract_to_dir(adf_path: &Path, out_dir: &Path) -> Result<Vec<PathBuf>, AdfError> {
    let reader = AdfReader::from_path(adf_path)?;
    let files = reader.list_root_files()?;
    std::fs::create_dir_all(out_dir)?;
    let mut written = Vec::with_capacity(files.len());
    for f in files {
        let path = out_dir.join(&f.name);
        std::fs::write(&path, &f.data)?;
        written.push(path);
    }
    Ok(written)
}

fn read_be_u32(buf: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ])
}

fn read_be_i32(buf: &[u8], offset: usize) -> i32 {
    i32::from_be_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ])
}

/// Decode the BCPL (length-prefixed) string at `block[BSIZE-80..]`.
/// Falls back to lossy UTF-8 decoding so off-spec bytes don't fail
/// the whole extract.
fn read_bcpl_name(block: &[u8]) -> String {
    let off = SECTOR_SIZE - 80;
    let len = block[off] as usize;
    let end = (off + 1 + len).min(block.len());
    String::from_utf8_lossy(&block[off + 1..end]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_too_small_image() {
        let r = AdfReader::from_bytes(vec![0u8; 100]);
        assert!(matches!(r, Err(AdfError::BadStructure(_))));
    }

    #[test]
    fn rejects_bad_magic() {
        let r = AdfReader::from_bytes(vec![0u8; SECTOR_SIZE * 2]);
        assert!(matches!(r, Err(AdfError::BadMagic)));
    }
}
