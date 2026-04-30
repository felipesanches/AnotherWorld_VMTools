//! Extract every file from an OFS-formatted ADF (Amiga Disk File)
//! image into a directory.
//!
//! Usage:
//!     adf-extract <adf_path> <output_dir> [<adf_path> ...]
//!
//! Pass multiple ADFs to merge their files into the same output
//! directory — the AnotherWorld retro-presskit ships two disks
//! (DiskA + DiskB), and the disassembler wants both unpacked side
//! by side.

use std::path::PathBuf;
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!(
        "usage: adf-extract <adf_path> <output_dir> [<adf_path> ...]\n\
         Extracts every file from each ADF into <output_dir>. If multiple\n\
         ADFs are given they all extract into the same directory; later disks\n\
         overwrite earlier ones on filename collisions."
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        return usage();
    }
    let output_dir = PathBuf::from(args.last().unwrap());
    let adf_paths = &args[..args.len() - 1];

    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        eprintln!("cannot create {}: {e}", output_dir.display());
        return ExitCode::from(1);
    }

    let mut total = 0usize;
    for adf in adf_paths {
        let path = PathBuf::from(adf);
        match awvm::adf::extract_to_dir(&path, &output_dir) {
            Ok(written) => {
                println!(
                    "{}: {} file{} extracted to {}",
                    path.display(),
                    written.len(),
                    if written.len() == 1 { "" } else { "s" },
                    output_dir.display()
                );
                for w in &written {
                    if let Some(name) = w.file_name() {
                        println!("    {}", name.to_string_lossy());
                    }
                }
                total += written.len();
            }
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                return ExitCode::from(1);
            }
        }
    }
    println!("\ntotal files extracted: {total}");
    ExitCode::SUCCESS
}
