//! Read a resource's payload out of a bank file, decompressing if
//! the entry is packed.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::memlist::MemEntry;
use crate::unpacker::{self, UnpackResult};

/// Lookup the right `bank<NN>` file for `entry.bank_id`. The Python
/// reference accepts either lowercase (`bank01`) or uppercase
/// (`bank01`) hex, controlled by an `uppercase` flag; we mirror it.
pub fn bank_path(banks_dir: &Path, bank_id: u8, uppercase: bool) -> PathBuf {
    let name = if uppercase {
        format!("bank{:02X}", bank_id)
    } else {
        format!("bank{:02x}", bank_id)
    };
    banks_dir.join(name)
}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    /// Decompression CRC check failed (resource is corrupt).
    CrcFailure,
    /// Decompressed payload was a different size than the entry
    /// promised. Matches the Python reference's "SHOULD BE … GOT …"
    /// hard error.
    SizeMismatch { expected: usize, got: usize },
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::Io(e)
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "io error: {e}"),
            ReadError::CrcFailure => write!(f, "decompression failed (CRC mismatch)"),
            ReadError::SizeMismatch { expected, got } => {
                write!(f, "size mismatch: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for ReadError {}

/// Read and (if necessary) decompress one resource from its bank.
pub fn read_resource(
    banks_dir: &Path,
    entry: &MemEntry,
    uppercase: bool,
) -> Result<Vec<u8>, ReadError> {
    let path = bank_path(banks_dir, entry.bank_id, uppercase);
    let mut file = File::open(&path)?;
    file.seek(SeekFrom::Start(entry.bank_offset as u64))?;

    let mut packed = vec![0u8; entry.packed_size as usize];
    file.read_exact(&mut packed)?;

    let raw = if entry.packed_size == entry.size {
        packed
    } else {
        match unpacker::unpack(&packed) {
            UnpackResult::Ok(v) => v,
            UnpackResult::CrcFailure => return Err(ReadError::CrcFailure),
        }
    };

    if raw.len() != entry.size as usize {
        return Err(ReadError::SizeMismatch {
            expected: entry.size as usize,
            got: raw.len(),
        });
    }

    Ok(raw)
}
