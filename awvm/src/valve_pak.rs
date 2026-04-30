//! Reader for the Valve PAK format (the "PACK" magic header used
//! by Quake / Half-Life / and, in our context, Eric Chahi's Windows
//! XP "1.1c" Another World hi-res demo distributed via
//! anotherworld.fr).
//!
//! Format (well-known, see e.g.
//! <https://quakewiki.org/wiki/.pak>):
//!
//! ```text
//! offset  type      meaning
//! 0..3    u8[4]     "PACK" magic
//! 4..7    u32 LE    file table offset
//! 8..11   u32 LE    file table length in bytes (entries × 64)
//!
//! Each file table entry is 64 bytes:
//!   0..55   u8[56]    null-terminated filename (slashes preserved)
//!   56..59  u32 LE    payload offset
//!   60..63  u32 LE    payload length
//! ```
//!
//! Note: the Windows XP Another World 1.1c PAK does NOT contain the
//! Eric Chahi VM bytecode that the rest of awvm-tools targets. The
//! 1.1c build is a DirectX-era graphical remake whose assets are
//! shaders, BMPs, WAVs, and OGG music — a distinct engine. The
//! reader is provided for completeness so the WinXP build's assets
//! can still be inspected, but its output does NOT feed
//! `awvm-disasm`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"PACK";
const ENTRY_LEN: usize = 64;
const NAME_LEN: usize = 56;

#[derive(Debug)]
pub enum PakError {
    Io(io::Error),
    BadMagic,
    BadStructure(String),
}

impl From<io::Error> for PakError {
    fn from(e: io::Error) -> Self {
        PakError::Io(e)
    }
}

impl std::fmt::Display for PakError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PakError::Io(e) => write!(f, "io error: {e}"),
            PakError::BadMagic => write!(f, "not a Valve PAK (bad 'PACK' magic)"),
            PakError::BadStructure(s) => write!(f, "bad PAK structure: {s}"),
        }
    }
}

impl std::error::Error for PakError {}

#[derive(Debug, Clone)]
pub struct PakEntry {
    pub name: String,
    pub offset: u32,
    pub length: u32,
}

pub struct PakReader {
    data: Vec<u8>,
    entries: Vec<PakEntry>,
}

impl PakReader {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, PakError> {
        if data.len() < 12 {
            return Err(PakError::BadStructure("file shorter than 12-byte header".into()));
        }
        if &data[0..4] != MAGIC {
            return Err(PakError::BadMagic);
        }
        let dir_off = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let dir_len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        if dir_off + dir_len > data.len() {
            return Err(PakError::BadStructure(format!(
                "directory at 0x{dir_off:x} + 0x{dir_len:x} exceeds file size {}",
                data.len()
            )));
        }
        if dir_len % ENTRY_LEN != 0 {
            return Err(PakError::BadStructure(format!(
                "directory length 0x{dir_len:x} is not a multiple of {}",
                ENTRY_LEN
            )));
        }
        let mut entries = Vec::with_capacity(dir_len / ENTRY_LEN);
        for i in 0..(dir_len / ENTRY_LEN) {
            let base = dir_off + i * ENTRY_LEN;
            let raw_name = &data[base..base + NAME_LEN];
            let end = raw_name.iter().position(|&b| b == 0).unwrap_or(NAME_LEN);
            let name = String::from_utf8_lossy(&raw_name[..end]).into_owned();
            let off = u32::from_le_bytes(data[base + NAME_LEN..base + NAME_LEN + 4].try_into().unwrap());
            let len = u32::from_le_bytes(data[base + NAME_LEN + 4..base + NAME_LEN + 8].try_into().unwrap());
            entries.push(PakEntry {
                name,
                offset: off,
                length: len,
            });
        }
        Ok(Self { data, entries })
    }

    pub fn from_path(path: &Path) -> Result<Self, PakError> {
        let bytes = fs::read(path)?;
        Self::from_bytes(bytes)
    }

    pub fn entries(&self) -> &[PakEntry] {
        &self.entries
    }

    pub fn read_entry(&self, entry: &PakEntry) -> Result<&[u8], PakError> {
        let s = entry.offset as usize;
        let e = s + entry.length as usize;
        if e > self.data.len() {
            return Err(PakError::BadStructure(format!(
                "entry {:?} payload [0x{s:x}..0x{e:x}) exceeds file size {}",
                entry.name,
                self.data.len()
            )));
        }
        Ok(&self.data[s..e])
    }
}

/// Extract every entry into `out_dir`, preserving the directory
/// structure encoded in entry filenames (`/` or `\` separators).
pub fn extract_to_dir(pak_path: &Path, out_dir: &Path) -> Result<Vec<PathBuf>, PakError> {
    let reader = PakReader::from_path(pak_path)?;
    fs::create_dir_all(out_dir)?;
    let mut written = Vec::with_capacity(reader.entries().len());
    for entry in reader.entries() {
        let normalized = entry.name.replace('\\', "/");
        let path = out_dir.join(&normalized);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let payload = reader.read_entry(entry)?;
        fs::write(&path, payload)?;
        written.push(path);
    }
    Ok(written)
}
