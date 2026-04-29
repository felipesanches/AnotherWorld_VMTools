//! Rust port of `releases/common_data/banks2resources.py`.
//!
//! Reads `memlist.bin` from `<input_dir>` and extracts every resource
//! into `<output_dir>/resources/resource-0xNN.bin` — same filename
//! pattern, same per-entry stdout log line, as the Python reference.
//!
//! Usage:
//!     banks2resources <input_dir> <output_dir> [--uppercase]

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use awvm::bank;
use awvm::memlist::{self, MemEntry};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let input_dir: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => {
            eprintln!("usage: banks2resources <input_dir> <output_dir> [--uppercase]");
            return ExitCode::from(2);
        }
    };
    let output_dir: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => {
            eprintln!("usage: banks2resources <input_dir> <output_dir> [--uppercase]");
            return ExitCode::from(2);
        }
    };
    let mut uppercase = false;
    for arg in args {
        match arg.as_str() {
            "--uppercase" => uppercase = true,
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let resources_dir = output_dir.join("resources");
    if let Err(e) = fs::create_dir_all(&resources_dir) {
        eprintln!("failed to create {}: {e}", resources_dir.display());
        return ExitCode::from(1);
    }

    let memlist_path = input_dir.join("memlist.bin");
    let memlist_bytes = match fs::read(&memlist_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to read {}: {e}", memlist_path.display());
            return ExitCode::from(1);
        }
    };

    let entries = match memlist::parse(&memlist_bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse memlist: {e}");
            return ExitCode::from(1);
        }
    };

    let mut had_failure = false;
    for (resource_index, entry) in entries.iter().enumerate() {
        log_entry(resource_index, entry);

        let bank_path = bank::bank_path(&input_dir, entry.bank_id, uppercase);
        if !bank_path.exists() {
            println!("Not found: {}", bank_path.display());
            continue;
        }

        match bank::read_resource(&input_dir, entry, uppercase) {
            Ok(data) => {
                let out = resources_dir.join(format!("resource-0x{:02x}.bin", resource_index));
                if let Err(e) = fs::write(&out, &data) {
                    eprintln!("failed to write {}: {e}", out.display());
                    had_failure = true;
                }
            }
            Err(bank::ReadError::SizeMismatch { expected, got }) => {
                // Python reference behaviour: "SHOULD BE %d ---- GOT %d" then sys.exit(-1).
                println!("SHOULD BE {expected} ---- GOT {got}");
                return ExitCode::from(255);
            }
            Err(e) => {
                eprintln!("resource 0x{resource_index:02x}: {e}");
                had_failure = true;
            }
        }
    }

    if had_failure { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

fn log_entry(resource_index: usize, entry: &MemEntry) {
    // Match the Python reference's stdout exactly:
    //   resource:0x<idx>\tbankId:<id>\ttype:<TYPE>\toffset:0x<off>\tsize:0x<packed> / 0x<size>\tnext:0x<off+packed>
    let next = entry.bank_offset.wrapping_add(entry.packed_size as u32);
    println!(
        "resource:{resource}\tbankId:{bank}\ttype:{ty}\toffset:{off}\tsize:{packed} / {size}\tnext:{next}",
        resource = format!("0x{:x}", resource_index),
        bank = entry.bank_id,
        ty = memlist::type_name(entry.typ),
        off = format!("0x{:x}", entry.bank_offset),
        packed = format!("0x{:x}", entry.packed_size),
        size = format!("0x{:x}", entry.size),
        next = format!("0x{:x}", next),
    );
}
