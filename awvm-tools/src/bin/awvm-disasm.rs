//! Rust port of `awvm-disasm.py`.
//!
//! Usage:
//!     awvm-disasm <input_dir> <level | "all_levels"> [release_name] [--no-polygons]
//!
//! Currently `release_name` only supports `msdos`. The CLI runs the
//! Phase-A `banks2resources` pipeline, then `resources2romset`, then
//! disassembles the requested levels into per-level `.asm` files
//! under `<cwd>/output/<release>/disasm/level_<N>/<release>_level-<N>.asm`.
//!
//! `--no-polygons` skips the polygon-SVG extraction step entirely.
//! Useful for apples-to-apples performance comparison with the
//! Python reference (whose polygon decoder crashes mid-run on this
//! fixture and is therefore stubbed in the perf harness anyway).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use awvm::{
    bank, disasm,
    disasm::Video2Accumulator,
    memlist, polygons, releases, romset,
};

fn usage() -> ExitCode {
    eprintln!(
        "usage: awvm-disasm <input_dir> <level | all_levels> [release_name]\n\
         currently only release_name=msdos is supported."
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut positional: Vec<String> = Vec::new();
    let mut no_polygons = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--no-polygons" => no_polygons = true,
            other if other.starts_with("--") => {
                eprintln!("unknown flag: {other}");
                return usage();
            }
            _ => positional.push(arg),
        }
    }
    if !(positional.len() == 2 || positional.len() == 3) {
        return usage();
    }
    let input_dir = PathBuf::from(&positional[0]);
    let level_arg = &positional[1];
    let release = positional.get(2).cloned();

    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cannot get cwd: {e}");
            return ExitCode::from(1);
        }
    };

    let release_slug = match release.as_deref() {
        None => {
            // The Python reference's no-release-name branch is broken
            // (references undefined `romset_dir`); we error out for
            // parity with that observable behaviour.
            eprintln!(
                "running awvm-disasm without a release name is not supported \
                 (this matches the broken Python reference behaviour). \
                 Pass a release slug (msdos, amiga, snes, genesis_europe, gba_usa, symbian_demo)."
            );
            return ExitCode::from(2);
        }
        Some(s) => s.to_owned(),
    };

    let release_data = match releases::by_slug(&release_slug) {
        Some(rd) => rd,
        None => {
            eprintln!("unknown release {:?}", release_slug);
            return ExitCode::from(2);
        }
    };

    let output_dir = cwd.join("output");
    let release_out = output_dir.join(&release_slug);
    let disasm_dir = release_out.join("disasm");
    let romset_dir = release_out.join("romset");

    println!("\n=== {} ===", release_slug);

    // Per-release pipeline: produces <release_out>/romset/ with at
    // minimum bytecode.rom plus the hardcoded text/font ROMs the
    // disassembler needs for the `text` opcode.
    let hardcoded = locate_hardcoded_data();
    let prep_result = match release_slug.as_str() {
        "msdos" => prepare_bank_romset(
            &input_dir,
            &release_out,
            &hardcoded,
            release_data.resource_ids,
            /* uppercase */ false,
            MemlistSource::File("memlist.bin"),
        ),
        "amiga" => prepare_bank_romset(
            &input_dir,
            &release_out,
            &hardcoded,
            release_data.resource_ids,
            /* uppercase */ true,
            MemlistSource::Embedded {
                file: "another",
                offset: 0x5ec2,
                length: 20 * 147,
            },
        ),
        "snes" => prepare_cartridge_romset(
            &input_dir,
            &release_out,
            &hardcoded,
            CartridgeSpec {
                source_filename: "Out of This World (USA).sfc",
                bytecode_chunks: &[(0x74A4C, 0x26A7), (0x81CB0, 0x51FD)],
                string_extraction: None,
            },
        ),
        "genesis_europe" => prepare_cartridge_romset(
            &input_dir,
            &release_out,
            &hardcoded,
            CartridgeSpec {
                source_filename: "Another World (Europe).md",
                bytecode_chunks: &[
                    (0x3f576, 0x51fe),
                    (0x5281a, 0x9c9a),
                    (0x693e8, 0xf564),
                    (0x88716, 0x1f88),
                    (0x919a0, 0xc714),
                    (0xbcab8, 0x0b5a),
                    (0xada78, 0x0be4),
                ],
                // genesis2romset.py:generate_text_string_roms walks the
                // cartridge from 0x382B to 0x46FE inclusive.
                string_extraction: Some((0x382B, 0x46FE)),
            },
        ),
        "gba_usa" => prepare_cartridge_romset(
            &input_dir,
            &release_out,
            &hardcoded,
            CartridgeSpec {
                source_filename: "Another World (Prototype) # GBA.GBA",
                bytecode_chunks: &[(0x6ea74, 0x10000), (0x813f8, 0x10000)],
                string_extraction: None,
            },
        ),
        other => {
            eprintln!(
                "release {:?}: prepare_romset not implemented yet \
                 (symbian_demo needs zlib + lzma decompression and is pending)",
                other
            );
            return ExitCode::from(2);
        }
    };
    if let Err(e) = prep_result {
        eprintln!("romset prep failed for {}: {e}", release_slug);
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
        let asm = level_dir.join(format!("{}_level-{level}.asm", release_slug));
        let dis = match disasm::disassemble_level(
            &gamerom,
            level,
            &str_data,
            &str_index,
            &asm,
            &mut video2,
            release_data,
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("level {level}: {e}");
                return ExitCode::from(1);
            }
        };
        println!("\t{} cinematic entries.", dis.cinematic_entries.len());

        if !no_polygons {
            // Cinematic polygons → SVG (Phase D).
            match polygons::PolygonDecoder::for_cinematic(&romset_dir, level) {
                Ok(mut pd) => {
                    let cin_dir = level_dir.join("cinematic");
                    let entries: Vec<_> = dis
                        .cinematic_entries
                        .iter()
                        .map(|(a, e)| (*a, e.clone()))
                        .collect();
                    if let Err(e) = pd.extract(entries, &cin_dir) {
                        eprintln!("level {level}: cinematic SVG extract: {e}");
                    }
                }
                Err(e) => eprintln!("level {level}: cinematic decoder init: {e}"),
            }
        }
    }
    println!("\t{} video2 entries.", video2.entries.len());

    if no_polygons {
        return ExitCode::SUCCESS;
    }

    // Common video (video2) polygons → SVG.
    match polygons::PolygonDecoder::for_video2(&romset_dir) {
        Ok(mut pd) => {
            let cv_dir = disasm_dir.join("common_video");
            let entries: Vec<_> = video2
                .entries
                .iter()
                .map(|(a, e)| (*a, e.clone()))
                .collect();
            if let Err(e) = pd.extract(entries, &cv_dir) {
                eprintln!("common_video SVG extract: {e}");
            }
        }
        Err(e) => eprintln!("video2 decoder init: {e}"),
    }

    ExitCode::SUCCESS
}

/// Where to find the memlist for a bank-format release.
enum MemlistSource<'a> {
    /// Plain file alongside the bank<NN> files.
    File(&'a str),
    /// Embedded inside another binary at a fixed offset (e.g. amiga's
    /// `another` executable holds the memlist at offset 0x5ec2).
    Embedded { file: &'a str, offset: usize, length: usize },
}

/// Per-cartridge-release recipe.
struct CartridgeSpec<'a> {
    /// Filename inside `input_dir` that holds the cartridge ROM.
    source_filename: &'a str,
    /// `(byte_offset, length)` pairs to extract sequentially into
    /// `bytecode.rom`. Each chunk is padded with `0xFF` to 0x10000
    /// bytes (one game level slab) before the next chunk is appended.
    bytecode_chunks: &'a [(usize, usize)],
    /// Where to source the text-string ROMs from:
    /// - `Some((start, end))`: extract genesis-style from the
    ///   cartridge over `[start, end]` (inclusive end), pre-extending
    ///   `str_data.rom` to 0x1000 and `str_index.rom` to 0x800 with
    ///   zeros to match the Python reference's output bytes.
    /// - `None`: copy the hardcoded MSDOS string ROMs from
    ///   `hardcoded_data/`.
    string_extraction: Option<(usize, usize)>,
}

/// Pipeline for releases packaged as memlist + bank<NN> files
/// (msdos, amiga). Reads the memlist, extracts every resource into
/// `<release_out>/resources/resource-0xNN.bin`, then runs the shared
/// resources2romset step to produce the romset.
fn prepare_bank_romset(
    input_dir: &Path,
    release_out: &Path,
    hardcoded: &Path,
    ids: romset::ResourceIds<'_>,
    uppercase: bool,
    source: MemlistSource<'_>,
) -> std::io::Result<()> {
    let resources_dir = release_out.join("resources");
    fs::create_dir_all(&resources_dir)?;

    let memlist_bytes: Vec<u8> = match source {
        MemlistSource::File(name) => fs::read(input_dir.join(name))?,
        MemlistSource::Embedded { file, offset, length } => {
            let raw = fs::read(input_dir.join(file))?;
            if offset + length > raw.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "{}: cannot read {} bytes at offset 0x{:x} (file is {} bytes)",
                        file,
                        length,
                        offset,
                        raw.len()
                    ),
                ));
            }
            raw[offset..offset + length].to_vec()
        }
    };

    let entries = memlist::parse(&memlist_bytes)?;
    for (i, entry) in entries.iter().enumerate() {
        let bank_path = bank::bank_path(input_dir, entry.bank_id, uppercase);
        if !bank_path.exists() {
            continue;
        }
        match bank::read_resource(input_dir, entry, uppercase) {
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

    romset::generate(&resources_dir, release_out, hardcoded, ids).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("resources2romset: {e}"))
    })?;
    Ok(())
}

/// Pipeline for releases packaged as a single cartridge ROM (snes,
/// genesis_europe, gba_usa, symbian_demo). Extracts hardcoded byte
/// chunks into bytecode.rom (each chunk padded to 0x10000 with
/// 0xFF), then copies the hardcoded text / font ROMs the
/// disassembler needs for the `text` opcode.
fn prepare_cartridge_romset(
    input_dir: &Path,
    release_out: &Path,
    hardcoded: &Path,
    spec: CartridgeSpec<'_>,
) -> std::io::Result<()> {
    let romset_dir = release_out.join("romset");
    fs::create_dir_all(&romset_dir)?;

    let raw = fs::read(input_dir.join(spec.source_filename))?;

    // Each chunk yields one 0x10000-byte slab; copy whatever is in range and
    // pad the rest with 0xFF, matching the Python references' behaviour
    // (some releases — gba_usa in particular — declare a chunk that reads
    // past EOF and rely on the 0xFF padding to fill the rest of the slab).
    let mut bytecode = Vec::with_capacity(spec.bytecode_chunks.len() * 0x10000);
    for (start, length) in spec.bytecode_chunks {
        let end = start.saturating_add(*length).min(raw.len());
        let real_start = (*start).min(raw.len());
        let actual = &raw[real_start..end];
        bytecode.extend_from_slice(actual);
        for _ in actual.len()..0x10000 {
            bytecode.push(0xFF);
        }
    }
    fs::write(romset_dir.join("bytecode.rom"), &bytecode)?;

    if let Some((start, end)) = spec.string_extraction {
        extract_cartridge_strings(&raw, start, end, &romset_dir)?;
        // Chargen still comes from hardcoded_data — no per-release
        // chargen extraction has been ported yet.
        fs::copy(
            hardcoded.join("anotherworld_chargen.rom"),
            romset_dir.join("anotherworld_chargen.rom"),
        )?;
    } else {
        for filename in ["str_data.rom", "str_index.rom", "anotherworld_chargen.rom"] {
            fs::copy(hardcoded.join(filename), romset_dir.join(filename))?;
        }
    }
    Ok(())
}

/// Port of `genesis2romset.py:generate_text_string_roms`. Walks the
/// cartridge from `start` through `end` reading `(BE 16-bit index,
/// null-terminated ASCII string)` records; writes them into
/// `str_data.rom` (sequential null-terminated payload) and
/// `str_index.rom` (`index*2` → little-endian offset into
/// `str_data.rom`). Both ROMs are pre-extended to 0x1000 / 0x800
/// bytes with zeros to match the Python reference.
fn extract_cartridge_strings(
    raw: &[u8],
    start: usize,
    end: usize,
    romset_dir: &Path,
) -> std::io::Result<()> {
    let mut str_data: Vec<u8> = vec![0u8; 0x1000];
    let mut str_index: Vec<u8> = vec![0u8; 0x800];

    let mut strdata_addr: usize = 0;
    let mut addr: usize = start;

    while addr <= end && addr + 1 < raw.len() {
        let index = ((raw[addr] as usize) << 8) | (raw[addr + 1] as usize);
        addr += 2;

        let idx_pos = index * 2;
        if idx_pos + 2 > str_index.len() {
            str_index.resize(idx_pos + 2, 0);
        }
        str_index[idx_pos] = (strdata_addr & 0xff) as u8;
        str_index[idx_pos + 1] = ((strdata_addr >> 8) & 0xff) as u8;

        while addr < raw.len() && raw[addr] != 0 {
            if strdata_addr >= str_data.len() {
                str_data.resize(strdata_addr + 1, 0);
            }
            str_data[strdata_addr] = raw[addr];
            addr += 1;
            strdata_addr += 1;
        }
        if strdata_addr >= str_data.len() {
            str_data.resize(strdata_addr + 1, 0);
        }
        str_data[strdata_addr] = 0;
        strdata_addr += 1;
        addr += 1; // skip the null in the cartridge
    }

    fs::write(romset_dir.join("str_data.rom"), &str_data)?;
    fs::write(romset_dir.join("str_index.rom"), &str_index)?;
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
