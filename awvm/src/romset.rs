//! Build a per-release romset from extracted resources.
//!
//! Port of `releases/common_data/resources2romset.py:ROMSet.generate`.
//! Concatenates per-level binaries (bytecode, cinematic, palettes,
//! video2) with `0xFF` padding to the engine's expected fixed sizes,
//! and copies `str_data.rom` / `str_index.rom` / `anotherworld_chargen.rom`
//! from the project's `hardcoded_data/` directory.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Per-release manifest of which resource ids belong in which ROM.
#[derive(Debug, Clone, Copy)]
pub struct ResourceIds<'a> {
    pub bytecode: &'a [u8],
    pub cinematic: &'a [u8],
    pub palette: &'a [u8],
    pub video2: &'a [u8],
}

/// Generate a complete romset under `output_dir/romset/`.
///
/// `input_dir` must contain `resource-0xNN.bin` files (output of
/// `banks2resources`). `hardcoded_data_dir` must contain the
/// committed `str_data.rom`, `str_index.rom`, and
/// `anotherworld_chargen.rom`.
pub fn generate(
    input_dir: &Path,
    output_dir: &Path,
    hardcoded_data_dir: &Path,
    ids: ResourceIds<'_>,
) -> io::Result<PathBuf> {
    let romset_dir = output_dir.join("romset");
    fs::create_dir_all(&romset_dir)?;

    write_padded(
        &romset_dir.join("bytecode.rom"),
        input_dir,
        ids.bytecode,
        0x10000,
    )?;
    write_padded(
        &romset_dir.join("cinematic.rom"),
        input_dir,
        ids.cinematic,
        0x10000,
    )?;
    write_concat(&romset_dir.join("palettes.rom"), input_dir, ids.palette)?;

    if ids.video2.len() > 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected at most one video2 resource id",
        ));
    }
    if let Some(&res) = ids.video2.first() {
        write_padded(
            &romset_dir.join("video2.rom"),
            input_dir,
            &[res],
            0x8000,
        )?;
    }

    for filename in ["str_data.rom", "str_index.rom", "anotherworld_chargen.rom"] {
        fs::copy(
            hardcoded_data_dir.join(filename),
            romset_dir.join(filename),
        )?;
    }

    Ok(romset_dir)
}

/// Concatenate each resource padded with `0xFF` to a fixed length.
fn write_padded(
    out: &Path,
    input_dir: &Path,
    resource_ids: &[u8],
    length: usize,
) -> io::Result<()> {
    let mut buf = Vec::with_capacity(length * resource_ids.len());
    for &id in resource_ids {
        let path = input_dir.join(format!("resource-0x{:02x}.bin", id));
        let data = fs::read(&path)?;
        buf.extend_from_slice(&data);
        for _ in data.len()..length {
            buf.push(0xFF);
        }
    }
    fs::write(out, &buf)
}

/// Plain concatenation (no padding).
fn write_concat(out: &Path, input_dir: &Path, resource_ids: &[u8]) -> io::Result<()> {
    let mut buf = Vec::new();
    for &id in resource_ids {
        let path = input_dir.join(format!("resource-0x{:02x}.bin", id));
        buf.extend_from_slice(&fs::read(&path)?);
    }
    fs::write(out, &buf)
}
