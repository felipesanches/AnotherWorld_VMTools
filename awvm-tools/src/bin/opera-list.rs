//! List the root directory of a 3DO Opera-filesystem `.bin` image.
//!
//! Usage:
//!     opera-list <bin_path>
//!     opera-list <bin_path> --extract <out_dir>
//!
//! Provides a foundation for the eventual 3DO Another World resource
//! extractor — once we know which files in the disc image hold the
//! AW VM bytecode + cinematic + palette data, an `awvm-disasm` slug
//! `3do` can be wired up.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let bin: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => {
            eprintln!("usage: opera-list <bin_path> [--extract <out_dir>]");
            return ExitCode::from(2);
        }
    };
    let mut extract_to: Option<PathBuf> = None;
    let mut next = args.next();
    if let Some(arg) = &next {
        if arg == "--extract" {
            extract_to = args.next().map(Into::into);
            next = args.next();
        }
    }
    if next.is_some() {
        eprintln!("trailing args not understood");
        return ExitCode::from(2);
    }

    let img = match awvm::opera::OperaImage::from_path(&bin) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}: {e}", bin.display());
            return ExitCode::from(1);
        }
    };

    let label = img.label().unwrap_or_else(|_| "(unknown)".into());
    let root = match img.root_dir_block() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}: {e}", bin.display());
            return ExitCode::from(1);
        }
    };
    println!("Volume label: {:?}", label);
    println!("Root dir at LBA {}", root);
    println!();

    let entries = match img.list_dir(root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}: {e}", bin.display());
            return ExitCode::from(1);
        }
    };
    println!(
        "{:5}  {:8}  {:>10}  {:<32}  {}",
        "id", "type", "bytes", "name", "first_block"
    );
    for e in &entries {
        println!(
            "{:5}  {:8}  {:>10}  {:<32}  {}",
            e.id,
            e.type_str(),
            e.byte_count,
            e.name,
            e.first_block
        );
    }
    println!("\n{} entries", entries.len());

    if let Some(out) = extract_to {
        match awvm::opera::extract_root_to_dir(&bin, &out) {
            Ok(written) => {
                println!("\nextracted {} file(s) to {}", written.len(), out.display());
                for p in &written {
                    if let Some(name) = p.file_name() {
                        println!("    {}", name.to_string_lossy());
                    }
                }
            }
            Err(e) => {
                eprintln!("extraction failed: {e}");
                return ExitCode::from(1);
            }
        }
    }
    ExitCode::SUCCESS
}
