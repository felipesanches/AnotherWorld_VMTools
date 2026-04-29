//! Rust port of `awvm-disasm.py`.
//!
//! Usage:
//!     awvm-disasm <input_dir> <level | "all_levels"> [release_name]
//!
//! Currently `release_name` only supports `msdos`. The CLI runs the
//! Phase-A `banks2resources` pipeline, then `resources2romset`, then
//! disassembles the requested levels into per-level `.asm` files
//! under `<cwd>/output/<release>/disasm/level_<N>/<release>_level-<N>.asm`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use awvm::{bank, disasm, disasm::Video2Accumulator, memlist, releases::msdos, romset};

fn usage() -> ExitCode {
    eprintln!(
        "usage: awvm-disasm <input_dir> <level | all_levels> [release_name]\n\
         currently only release_name=msdos is supported."
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(args.len() == 2 || args.len() == 3) {
        return usage();
    }
    let input_dir = PathBuf::from(&args[0]);
    let level_arg = &args[1];
    let release = args.get(2).cloned();

    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cannot get cwd: {e}");
            return ExitCode::from(1);
        }
    };

    let (disasm_dir, romset_dir) = match release.as_deref() {
        None => {
            // The Python reference's no-release-name branch is broken
            // (references undefined `romset_dir`); we error out for
            // parity with that observable behaviour.
            eprintln!(
                "running awvm-disasm without a release name is not supported \
                 (this matches the broken Python reference behaviour). \
                 Pass `msdos` to use the MSDOS pipeline."
            );
            return ExitCode::from(2);
        }
        Some("msdos") => {
            let output_dir = cwd.join("output");
            let release_out = output_dir.join("msdos");
            (
                release_out.join("disasm"),
                release_out.join("romset"),
            )
        }
        Some(other) => {
            eprintln!("unsupported release {:?}; only `msdos` is implemented today", other);
            return ExitCode::from(2);
        }
    };

    println!("\n=== msdos ===");

    // Step 1: banks2resources
    let resources_dir = romset_dir.parent().unwrap().join("resources");
    if let Err(e) = build_resources(&input_dir, romset_dir.parent().unwrap()) {
        eprintln!("banks2resources failed: {e}");
        return ExitCode::from(1);
    }

    // Step 2: resources2romset
    let hardcoded = locate_hardcoded_data();
    let ids = romset::ResourceIds {
        bytecode: msdos::resource_ids::BYTECODE,
        cinematic: msdos::resource_ids::CINEMATIC,
        palette: msdos::resource_ids::PALETTE,
        video2: msdos::resource_ids::VIDEO2,
    };
    if let Err(e) = romset::generate(
        &resources_dir,
        romset_dir.parent().unwrap(),
        &hardcoded,
        ids,
    ) {
        eprintln!("resources2romset failed: {e}");
        return ExitCode::from(1);
    }

    let gamerom = romset_dir.join("bytecode.rom");
    let str_data = romset_dir.join("str_data.rom");
    let str_index = romset_dir.join("str_index.rom");

    if let Err(e) = fs::create_dir_all(&disasm_dir) {
        eprintln!("cannot create {}: {e}", disasm_dir.display());
        return ExitCode::from(1);
    }

    let gamerom_size = match fs::metadata(&gamerom) {
        Ok(m) => m.len(),
        Err(e) => {
            eprintln!("cannot stat {}: {e}", gamerom.display());
            return ExitCode::from(1);
        }
    };
    let num_levels = gamerom_size / 0x10000;
    println!("Num. levels = {num_levels}");

    let levels: Vec<u32> = if level_arg == "all_levels" {
        (0..num_levels as u32).collect()
    } else {
        match level_arg.parse::<u32>() {
            Ok(n) => vec![n],
            Err(_) => return usage(),
        }
    };

    let mut video2 = Video2Accumulator::default();
    for level in levels {
        println!("disassembling level {level}...");
        let level_dir = disasm_dir.join(format!("level_{level}"));
        let asm = level_dir.join(format!("msdos_level-{level}.asm"));
        let dis = match disasm::disassemble_level(
            &gamerom,
            level,
            &str_data,
            &str_index,
            &asm,
            &mut video2,
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("level {level}: {e}");
                return ExitCode::from(1);
            }
        };
        println!("\t{} cinematic entries.", dis.cinematic_entries.len());
    }
    println!("\t{} video2 entries.", video2.entries.len());

    ExitCode::SUCCESS
}

/// Wraps the Phase-A logic from `awvm-tools/src/bin/banks2resources.rs`.
/// We re-call the library here rather than shelling to the binary.
fn build_resources(input_dir: &Path, output_dir: &Path) -> std::io::Result<()> {
    let resources_dir = output_dir.join("resources");
    fs::create_dir_all(&resources_dir)?;
    let memlist_bytes = fs::read(input_dir.join("memlist.bin"))?;
    let entries = memlist::parse(&memlist_bytes)?;
    for (i, entry) in entries.iter().enumerate() {
        let bank_path = bank::bank_path(input_dir, entry.bank_id, false);
        if !bank_path.exists() {
            continue;
        }
        match bank::read_resource(input_dir, entry, false) {
            Ok(data) => {
                let out = resources_dir.join(format!("resource-0x{:02x}.bin", i));
                fs::write(out, data)?;
            }
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("resource 0x{:02x}: {e}", i),
                ));
            }
        }
    }
    Ok(())
}

/// Find the `hardcoded_data/` directory.  We look in the workspace
/// root (relative to the binary's `cargo run` cwd) and fall back to
/// the parent of `target/`.
fn locate_hardcoded_data() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidate = cwd.join("hardcoded_data");
    if candidate.is_dir() {
        return candidate;
    }
    // Walk up from the binary location.
    if let Ok(exe) = std::env::current_exe() {
        let mut p: &Path = &exe;
        while let Some(parent) = p.parent() {
            let cand = parent.join("hardcoded_data");
            if cand.is_dir() {
                return cand;
            }
            p = parent;
        }
    }
    PathBuf::from("hardcoded_data")
}
