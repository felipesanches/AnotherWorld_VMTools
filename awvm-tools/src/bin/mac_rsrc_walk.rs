//! Walk a classic Macintosh resource fork (the `.rsrc` files
//! produced by `mac-stuffit-extract`) and emit each individual
//! resource as a separate file, named `<TYPE>_<ID>_<safe_name>.bin`.
//!
//! Mac applications stored their executable code, dialog templates,
//! menus, sound, palettes, polygon data, etc. in the resource fork
//! as TYPE+ID-keyed blobs (e.g. type='CODE' id=0..N for 68k code
//! segments; type='SIZE' id=-1 for the size flags; type='vers' for
//! version metadata). For the 1993 Mac OOTW port, the AW VM
//! resources are expected to live under custom four-char-code types
//! we haven't yet mapped.
//!
//! Usage:
//!     mac-rsrc-walk <rsrc_file> <output_dir>
//!     mac-rsrc-walk <rsrc_file> <output_dir> --summary-only
//!
//! The summary-only mode prints the (type, count, total_bytes) table
//! without writing per-resource files — useful for surveying a fork
//! to learn what resource types exist before committing to extraction.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use macbinary::ResourceFork;

fn usage() -> ExitCode {
    eprintln!("usage: mac-rsrc-walk <rsrc_file> <output_dir> [--summary-only]");
    ExitCode::from(2)
}

fn fourcc_to_string(fcc: macbinary::FourCC) -> String {
    let bytes = fcc.0.to_be_bytes();
    bytes
        .iter()
        .map(|&b| if (0x20..=0x7e).contains(&b) { b as char } else { '_' })
        .collect()
}

fn safe_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | ' ') { c } else { '_' })
        .collect()
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let rsrc_path: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => return usage(),
    };
    let out_dir: PathBuf = match args.next() {
        Some(s) => s.into(),
        None => return usage(),
    };
    let mut summary_only = false;
    if let Some(arg) = args.next() {
        if arg == "--summary-only" {
            summary_only = true;
        } else {
            eprintln!("unknown arg: {arg}");
            return usage();
        }
    }
    if args.next().is_some() {
        return usage();
    }

    let bytes = match fs::read(&rsrc_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}: {e}", rsrc_path.display());
            return ExitCode::from(1);
        }
    };

    let fork = match ResourceFork::new(&bytes) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ResourceFork::new failed: {e}");
            eprintln!(
                "Note: the input must be a raw resource fork (typically a .rsrc \
                 file produced by mac-stuffit-extract). MacBinary-wrapped files \
                 are not yet handled here."
            );
            return ExitCode::from(1);
        }
    };

    if !summary_only {
        if let Err(e) = fs::create_dir_all(&out_dir) {
            eprintln!("create {}: {e}", out_dir.display());
            return ExitCode::from(1);
        }
    }

    // Tally per-type counts and total bytes for the summary line.
    let mut tally: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut written = 0usize;

    for type_item in fork.resource_types() {
        let type_str = fourcc_to_string(type_item.resource_type());

        for rsrc in fork.resources(type_item) {
            let id = rsrc.id();
            let data = rsrc.data();
            let name = rsrc
                .name()
                .map(|n| n.to_string())
                .unwrap_or_default();

            let entry = tally.entry(type_str.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += data.len();

            if !summary_only {
                let safe_name = if name.is_empty() {
                    String::new()
                } else {
                    format!("_{}", safe_filename(&name))
                };
                // Negative IDs are common (e.g. SIZE -1, vers -1). Use
                // a textual sign so the filename sorts sanely.
                let id_part = if id < 0 {
                    format!("n{}", -(id as i32))
                } else {
                    format!("{id}")
                };
                let fname = format!(
                    "{type_str}_{id_part}{safe_name}.bin"
                );
                let path = out_dir.join(&fname);
                if let Err(e) = fs::write(&path, data) {
                    eprintln!("write {}: {e}", path.display());
                    return ExitCode::from(1);
                }
                written += 1;
            }
        }
    }

    println!("\nResource type summary ({}):", rsrc_path.display());
    println!("    {:<8}  {:>5}  {:>12}", "TYPE", "count", "total_bytes");
    for (typ, (count, bytes)) in &tally {
        println!("    {typ:<8}  {count:>5}  {bytes:>12}");
    }
    let total_count: usize = tally.values().map(|(c, _)| c).sum();
    let total_bytes: usize = tally.values().map(|(_, b)| b).sum();
    println!("    {:<8}  {:>5}  {:>12}", "(total)", total_count, total_bytes);

    if !summary_only {
        println!("\nwrote {written} per-resource file(s) to {}", out_dir.display());
    }
    ExitCode::SUCCESS
}
