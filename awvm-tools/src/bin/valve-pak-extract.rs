//! Extract every file from a Valve PAK (`PACK` magic) into a
//! directory.
//!
//! Provided primarily so the Windows XP Another World 1.1c
//! `Data/Pak01.pak` can be unpacked. NOTE: the 1.1c build is a
//! distinct engine — DirectX-era graphical remake — so the
//! contents are shaders, BMPs, WAVs and OGG music, NOT the Eric
//! Chahi VM bytecode that the rest of awvm-tools targets.
//!
//! Usage:
//!     valve-pak-extract <pak_path> <output_dir>

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let pak: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => {
            eprintln!("usage: valve-pak-extract <pak_path> <output_dir>");
            return ExitCode::from(2);
        }
    };
    let out: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => {
            eprintln!("usage: valve-pak-extract <pak_path> <output_dir>");
            return ExitCode::from(2);
        }
    };
    match awvm::valve_pak::extract_to_dir(&pak, &out) {
        Ok(written) => {
            println!(
                "{}: extracted {} file{} to {}",
                pak.display(),
                written.len(),
                if written.len() == 1 { "" } else { "s" },
                out.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}: {e}", pak.display());
            ExitCode::from(1)
        }
    }
}
