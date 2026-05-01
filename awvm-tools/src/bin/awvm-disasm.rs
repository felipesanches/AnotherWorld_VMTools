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
                cinematic_chunks: &[],
                string_extraction: None,
            },
        ),
        // SNES Europe ROM uses the same memory layout as USA — same
        // bytecode chunk offsets, same level structure. Different
        // cartridge filename and (presumably) some localised strings,
        // but those do not affect the chunk offsets the extractor
        // uses. Validated by round-trip on the locally-archived EU ROM.
        "snes-eu" | "snes_eu" => prepare_cartridge_romset(
            &input_dir,
            &release_out,
            &hardcoded,
            CartridgeSpec {
                source_filename: "Another World (Europe).sfc",
                bytecode_chunks: &[(0x74A4C, 0x26A7), (0x81CB0, 0x51FD)],
                cinematic_chunks: &[],
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
                cinematic_chunks: &[],
                // genesis2romset.py:generate_text_string_roms walks the
                // cartridge from 0x382B to 0x46FE inclusive.
                string_extraction: Some((0x382B, 0x46FE)),
            },
        ),
        // GBA-Foxy port (2004). The cartridge ROM stores each level's
        // bytecode immediately followed by that level's cinematic-
        // polygon slab + a small palette region (research/10 in the
        // archaeology repo). Chunk offsets are derived as follows:
        //
        // level_0 bytecode: 0x6EA74..0x71127 (length 0x26B3, ending
        //   with the killChannel at chunk-relative 0x26B2). Then a
        //   1-byte separator (0x0F) at 0x71127, then cinematic at
        //   0x71128.
        // level_0 cinematic.rom slab: 0x71128, length 0x10000.
        //
        // level_1 bytecode: 0x813F8..0x8661D (length 0x5225). Three
        //   bytes 0x11/0x11/0x00 at 0x8661D..0x8661F (likely two
        //   stray killChannels + 1 separator), then cinematic at
        //   0x86620.
        // level_1 cinematic.rom slab: 0x86620, length 0x10000.
        //
        // Verified by brute-force scan: at the cinematic-base offsets
        // above, ALL the CINEMATIC_xxx labels referenced in each
        // level's disassembly resolve to valid polygon-entry bytes
        // (≥ 0xC0 fill, or low-6==0x02 hierarchy). 100 % match for
        // both levels.
        //
        // The previous spec said `bytecode_chunks: &[(0x6ea74, 0x10000),
        // (0x813f8, 0x10000)]` — capturing each level's full 64 KB.
        // That over-extracted: it bundled cinematic data into
        // bytecode.rom as if it were bytecode, so the disassembler
        // emitted ~55 KB of `db <bytes>` per level after the actual
        // killChannel.
        //
        // TODO: stages 3-7 in `gba_usa.rs` STAGE_TITLES are not yet
        // mapped. The pattern (bytecode | sep | cinematic | small palette)
        // probably repeats; a brute-force scan after disassembling
        // those levels would pin them down.
        "gba_usa" => prepare_cartridge_romset(
            &input_dir,
            &release_out,
            &hardcoded,
            CartridgeSpec {
                source_filename: "Another World (Prototype) # GBA.GBA",
                bytecode_chunks: &[(0x6EA74, 0x26B3), (0x813F8, 0x5225)],
                cinematic_chunks: &[(0x71128, 0x10000), (0x86620, 0x10000)],
                string_extraction: None,
            },
        ),
        "symbian_demo" => prepare_symbian_romset(&input_dir, &release_out, &hardcoded),
        other => {
            eprintln!("release {:?}: prepare_romset not implemented", other);
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
    /// `length` is the length of the actual bytecode for that level —
    /// the rest of the slab is filled with `0xFF`. Setting `length` to
    /// the full `0x10000` was the original (incorrect) behaviour for
    /// `gba_usa`: it captured cinematic-polygon data that immediately
    /// follows each level's bytecode in the GBA ROM as if the data
    /// were bytecode. See `cinematic_chunks` and research/10.
    bytecode_chunks: &'a [(usize, usize)],
    /// `(byte_offset, length)` pairs to extract sequentially into
    /// `cinematic.rom`. Each chunk is padded with `0xFF` to 0x10000
    /// (one cinematic-bank level slab). Empty `&[]` means the
    /// cartridge stores cinematic data elsewhere (or it hasn't been
    /// mapped yet) — `cinematic.rom` is not produced.
    ///
    /// For cartridge ports the cinematic data lives directly in the
    /// cartridge ROM (no separate per-resource compression) at fixed
    /// offsets that have to be discovered per-port.
    cinematic_chunks: &'a [(usize, usize)],
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
    fn extract_padded_chunks(raw: &[u8], chunks: &[(usize, usize)]) -> Vec<u8> {
        let mut out = Vec::with_capacity(chunks.len() * 0x10000);
        for (start, length) in chunks {
            let end = start.saturating_add(*length).min(raw.len());
            let real_start = (*start).min(raw.len());
            let actual = &raw[real_start..end];
            out.extend_from_slice(actual);
            for _ in actual.len()..0x10000 {
                out.push(0xFF);
            }
        }
        out
    }

    let bytecode = extract_padded_chunks(&raw, spec.bytecode_chunks);
    fs::write(romset_dir.join("bytecode.rom"), &bytecode)?;

    // Cinematic-polygon data, if the per-port spec has identified its
    // location in the cartridge ROM. Same layout as bytecode.rom: one
    // 0x10000-byte slab per level, 0xFF-padded.
    if !spec.cinematic_chunks.is_empty() {
        let cinematic = extract_padded_chunks(&raw, spec.cinematic_chunks);
        fs::write(romset_dir.join("cinematic.rom"), &cinematic)?;
    }

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

/// Port of `symbian2romset.py:SymbianDemoROMSet`.
///
/// Reads `<input_dir>/locked_anotherworld.sis`, slices the zlib
/// payload at `[0xBBA, 0xBBA + 749540)`, decompresses it (outer
/// layer), then for each `(start, end)` chunk runs LZMA1 decompress
/// over `raw[start..end]` and pads the result to 0x10000 with `0xFF`
/// (matching the Python reference). Produces:
///   - `<release_out>/romset/bytecode.rom`     (8 levels' bytecode)
///   - `<release_out>/romset/cinematic.rom`    (1 cinematic slab)
///   - the hardcoded text/font ROMs from `hardcoded_data/`
///
/// Note: the Python reference does NOT extract symbian-specific
/// text strings; it relies on the disassembler to fall back to
/// whatever str_data.rom is present. We mirror that by copying the
/// hardcoded MSDOS string ROMs.
fn prepare_symbian_romset(
    input_dir: &Path,
    release_out: &Path,
    hardcoded: &Path,
) -> std::io::Result<()> {
    use std::io::Read;

    let romset_dir = release_out.join("romset");
    fs::create_dir_all(&romset_dir)?;

    let sis = fs::read(input_dir.join("locked_anotherworld.sis"))?;

    const ZLIB_OFFSET: usize = 0xBBA;
    const ZLIB_LENGTH: usize = 749540;
    if ZLIB_OFFSET + ZLIB_LENGTH > sis.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "locked_anotherworld.sis: cannot slice zlib payload \
                 [0x{:x}..0x{:x}) from a {}-byte file",
                ZLIB_OFFSET,
                ZLIB_OFFSET + ZLIB_LENGTH,
                sis.len()
            ),
        ));
    }
    let packed = &sis[ZLIB_OFFSET..ZLIB_OFFSET + ZLIB_LENGTH];

    let mut zlib_decoder = flate2::read::ZlibDecoder::new(packed);
    let mut raw = Vec::new();
    zlib_decoder.read_to_end(&mut raw)?;

    // Bytecode: 8 levels (note: per the Python comment, this release
    // does not have a level-0 bytecode — level numbers start at 1).
    let bytecode_chunks: &[(usize, usize)] = &[
        (0x49D8C, 0x4B38F), // level 1
        (0x55F55, 0x587A6), // level 2
        (0x60A5B, 0x6551D), // level 3
        (0x6CA75, 0x73E92), // level 4
        (0x7D4DC, 0x7E65A), // level 5
        (0x81B7E, 0x87A59), // level 6
        (0x8FDA0, 0x903E8), // level 7
        (0xC335B, 0xC39CE), // level 8
    ];
    let bytecode = decompress_lzma_chunks(&raw, bytecode_chunks)?;
    fs::write(romset_dir.join("bytecode.rom"), &bytecode)?;

    let cinematic_chunks: &[(usize, usize)] = &[(0x6551E, 0x6C675)];
    let cinematic = decompress_lzma_chunks(&raw, cinematic_chunks)?;
    fs::write(romset_dir.join("cinematic.rom"), &cinematic)?;

    for filename in ["str_data.rom", "str_index.rom", "anotherworld_chargen.rom"] {
        fs::copy(hardcoded.join(filename), romset_dir.join(filename))?;
    }
    Ok(())
}

/// Run LZMA1 ("alone" format, the default Python `lzma.LZMADecompressor()`
/// auto-detects on these byte streams) over each `[start, end)` slice
/// of `raw` and pad the result to 0x10000 with `0xFF`. Concatenates
/// all slabs into a single returned buffer.
fn decompress_lzma_chunks(raw: &[u8], chunks: &[(usize, usize)]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(chunks.len() * 0x10000);
    for (start, end) in chunks {
        if *end > raw.len() || *start >= *end {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "lzma chunk [0x{:x}, 0x{:x}) out of range for {}-byte payload",
                    start,
                    end,
                    raw.len()
                ),
            ));
        }
        let mut cursor = std::io::Cursor::new(&raw[*start..*end]);
        let mut decompressed = Vec::new();
        lzma_rs::lzma_decompress(&mut cursor, &mut decompressed).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("lzma_decompress at [0x{:x}, 0x{:x}): {e}", start, end),
            )
        })?;
        out.extend_from_slice(&decompressed);
        for _ in decompressed.len()..0x10000 {
            out.push(0xFF);
        }
    }
    Ok(out)
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
