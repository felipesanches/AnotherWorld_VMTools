//! Extract files (data + resource forks) from a classic Macintosh
//! StuffIt `.sit` archive.
//!
//! The 1993 Macintosh release of *Out of This World* (`macintosh-1993`)
//! ships as `out_of_this_world.sit` from Macintosh Garden — a StuffIt
//! 5.x archive bundling v1.0 + v1.2 + v1.3 with updaters. The
//! application binaries are 68k Mac executables whose AW VM resources
//! and engine code live in the **resource fork**, not the data fork —
//! inverse of every other AW release format.
//!
//! Pipeline:
//! 1. Parse the .sit container with the `stuffit` crate; iterate
//!    every file entry.
//! 2. For each file entry, write `<name>.data` (the data fork) and
//!    `<name>.rsrc` (the resource fork) into the output directory.
//!    The MacBinary boundary is preserved separately so downstream
//!    code can use the `macbinary` crate to walk the resource fork.
//!
//! Usage:
//!     mac-stuffit-extract <sit_path> <output_dir>

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!("usage: mac-stuffit-extract <sit_path> <output_dir>");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let sit_path: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => return usage(),
    };
    let out_dir: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => return usage(),
    };
    if args.next().is_some() {
        return usage();
    }

    if let Err(e) = fs::create_dir_all(&out_dir) {
        eprintln!("create {}: {e}", out_dir.display());
        return ExitCode::from(1);
    }

    let bytes = match fs::read(&sit_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}: {e}", sit_path.display());
            return ExitCode::from(1);
        }
    };

    // The stuffit crate exposes SitArchive::parse(&[u8]) → SitArchive,
    // whose .entries field is a Vec<SitEntry> with .name, .data_fork,
    // .resource_fork already decompressed.
    let archive = match stuffit::SitArchive::parse(&bytes) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("stuffit parse failed: {e}");
            eprintln!(
                "Note: the stuffit crate (v0.1.4) advertises StuffIt 5.0 + SIT! 1.x \
                 with compression methods 0 (store), 13 (LZ77+Huffman), 14 (Deflate, \
                 limited), 15 (Arsenic/BWT, read-only). 5.x StuffIt-X archives using \
                 unimplemented compression methods (Brimstone etc.) will fail."
            );
            return ExitCode::from(1);
        }
    };

    let mut count = 0usize;
    for entry in &archive.entries {
        let name = &entry.name;
        let safe_name: String = name
            .chars()
            .map(|c: char| if c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | ' ') { c } else { '_' })
            .collect();

        // The forks on the entry are still *compressed* — we have to
        // call decompressed_forks() to get the raw data + resource
        // fork bytes. Folder entries have empty forks; for those
        // decompressed_forks() trivially returns empty Vecs.
        let (data, rsrc) = match entry.decompressed_forks() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("WARN: decompress failed for {name:?}: {e} (skipping)");
                continue;
            }
        };

        // Data fork — many entries will have an empty data fork (Mac
        // applications often store everything in the resource fork).
        let dst = out_dir.join(format!("{safe_name}.data"));
        if let Err(e) = fs::write(&dst, &data) {
            eprintln!("write {}: {e}", dst.display());
            return ExitCode::from(1);
        }

        // Resource fork — this is where AW VM resources live.
        if !rsrc.is_empty() {
            let dst = out_dir.join(format!("{safe_name}.rsrc"));
            if let Err(e) = fs::write(&dst, &rsrc) {
                eprintln!("write {}: {e}", dst.display());
                return ExitCode::from(1);
            }
        }

        println!(
            "    {name:<48} data={:>9} bytes  rsrc={:>9} bytes",
            data.len(),
            rsrc.len()
        );
        count += 1;
    }

    println!("\ntotal entries extracted: {count}");
    ExitCode::SUCCESS
}
