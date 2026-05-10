//! Render every polygon in a port's cinematic / video2 bank as
//! a colour SVG. Built to settle the open visual question raised
//! by archaeology research finding 25 (msdos's ENDING is largely
//! redrawn graphics, but the qualitative "redrawn vs cropped vs
//! added" judgement was unresolved when only outline-only solid
//! polygons could be rendered from outside Rust code).
//!
//! This binary wraps the existing `awvm::polygons::PolygonDecoder`
//! library API:
//!
//!   1. Walk the polygon resource linearly via
//!      `awvm::polygons::walk_linear` to enumerate every offset
//!      where a polygon parses cleanly (solid + group).
//!   2. For each offset, manufacture a `VideoEntry` with default
//!      position/zoom/palette and feed `PolygonDecoder::extract`
//!      the resulting (addr, entry) pairs.
//!   3. Output one SVG per polygon at
//!      `<out>/<port>_<stage>_<offset>.svg`.
//!
//! Group polygons render with their composed children + per-child
//! palette colours, which is what makes them recognisable as
//! cinematic scene frames (the missing piece archaeology's
//! shape-only renderer couldn't produce).
//!
//! Usage:
//!     awvm-render-polygons --port amiga --stage 7 \
//!         --romset /path/to/output/amiga/romset \
//!         --out /path/to/output/dir
//!
//! `--bank cinematic` (default) reads `cinematic.rom`; `--bank
//! video2` reads `video2.rom` (the cross-stage shared bank).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use awvm::disasm::VideoEntry;
use awvm::polygons::{walk_linear, PolygonDecoder, PolygonKind};

#[derive(Default)]
struct Args {
    port: String,
    stage: u32,
    romset: PathBuf,
    out: PathBuf,
    bank: String,
    palette: u8,
    skip_groups: bool,
    skip_solids: bool,
    limit: Option<usize>,
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: awvm-render-polygons --port <slug> --stage <N> \
         --romset <dir> --out <dir>\n\
         \n\
         Optional:\n  \
           --bank cinematic|video2   (default: cinematic)\n  \
           --palette <0-31>          (default: 0)\n  \
           --skip-groups              (only render solid polygons)\n  \
           --skip-solids              (only render group polygons)\n  \
           --limit <N>                (cap output to first N polygons)\n"
    );
    ExitCode::from(2)
}

fn parse_args() -> Result<Args, ExitCode> {
    let mut a = Args {
        bank: "cinematic".to_owned(),
        ..Default::default()
    };
    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let arg = std::mem::take(&mut argv[i]);
        match arg.as_str() {
            "--port" => {
                i += 1;
                if i >= argv.len() {
                    return Err(usage());
                }
                a.port = std::mem::take(&mut argv[i]);
            }
            "--stage" => {
                i += 1;
                if i >= argv.len() {
                    return Err(usage());
                }
                a.stage = argv[i].parse().map_err(|_| usage())?;
            }
            "--romset" => {
                i += 1;
                if i >= argv.len() {
                    return Err(usage());
                }
                a.romset = PathBuf::from(&argv[i]);
            }
            "--out" => {
                i += 1;
                if i >= argv.len() {
                    return Err(usage());
                }
                a.out = PathBuf::from(&argv[i]);
            }
            "--bank" => {
                i += 1;
                if i >= argv.len() {
                    return Err(usage());
                }
                a.bank = std::mem::take(&mut argv[i]);
            }
            "--palette" => {
                i += 1;
                if i >= argv.len() {
                    return Err(usage());
                }
                a.palette = argv[i].parse().map_err(|_| usage())?;
            }
            "--skip-groups" => a.skip_groups = true,
            "--skip-solids" => a.skip_solids = true,
            "--limit" => {
                i += 1;
                if i >= argv.len() {
                    return Err(usage());
                }
                a.limit = Some(argv[i].parse().map_err(|_| usage())?);
            }
            "-h" | "--help" => return Err(usage()),
            _ => {
                eprintln!("unknown argument: {arg}");
                return Err(usage());
            }
        }
        i += 1;
    }
    if a.port.is_empty() || a.romset.as_os_str().is_empty()
        || a.out.as_os_str().is_empty()
    {
        return Err(usage());
    }
    Ok(a)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(c) => return c,
    };

    // Build the decoder. cinematic.rom is laid out as 9 64-KiB
    // slabs indexed by stage; video2.rom is a single shared bank.
    let decoder = match args.bank.as_str() {
        "cinematic" => PolygonDecoder::for_cinematic(&args.romset, args.stage),
        "video2" => PolygonDecoder::for_video2(&args.romset),
        other => {
            eprintln!("unknown --bank {other:?} (use cinematic|video2)");
            return ExitCode::from(2);
        }
    };
    let mut decoder = match decoder {
        Ok(d) => d,
        Err(e) => {
            eprintln!("loading {} bank from {}: {}",
                      args.bank, args.romset.display(), e);
            return ExitCode::from(1);
        }
    };

    // Walk the per-stage resource (within cinematic.rom) OR the
    // whole video2.rom to enumerate polygons.
    let resource_bytes = match read_resource_for_walk(&args) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("reading resource for walk: {e}");
            return ExitCode::from(1);
        }
    };
    let polys = walk_linear(&resource_bytes, 0);
    eprintln!(
        "{}: walked {} polygons ({} solid, {} group) in {} bytes",
        args.port,
        polys.len(),
        polys.iter().filter(|p| p.kind == PolygonKind::Solid).count(),
        polys.iter().filter(|p| p.kind == PolygonKind::Group).count(),
        resource_bytes.len(),
    );

    // Filter + cap.
    let entries: Vec<(u32, VideoEntry)> = polys
        .into_iter()
        .filter(|p| !(args.skip_solids && p.kind == PolygonKind::Solid))
        .filter(|p| !(args.skip_groups && p.kind == PolygonKind::Group))
        .take(args.limit.unwrap_or(usize::MAX))
        .map(|p| {
            let kind_tag = match p.kind {
                PolygonKind::Solid => "solid",
                PolygonKind::Group => "group",
            };
            let entry = VideoEntry {
                palette_number: args.palette,
                x: "160".to_owned(),
                y: "100".to_owned(),
                zoom: "64".to_owned(), // 0x40
                label: format!(
                    "{}_stage{}_{}_0x{:04X}",
                    args.port, args.stage, kind_tag, p.offset
                ),
            };
            (p.offset, entry)
        })
        .collect();

    eprintln!("rendering {} polygons to {}",
              entries.len(), args.out.display());
    match decoder.extract(entries, &args.out) {
        Ok(written) => {
            eprintln!("wrote {} SVG file(s)", written.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("extract failed: {e}");
            ExitCode::from(1)
        }
    }
}

/// For cinematic-mode renderers we want the per-stage slab from
/// `cinematic.rom` (the bank holds 9 × 64KiB slabs indexed by
/// stage). For video2-mode, the whole `video2.rom` is one slab.
fn read_resource_for_walk(args: &Args) -> std::io::Result<Vec<u8>> {
    match args.bank.as_str() {
        "cinematic" => {
            let path = args.romset.join("cinematic.rom");
            let all = fs::read(&path)?;
            let slab_start = (args.stage as usize) * 0x10000;
            let slab_end = slab_start + 0x10000;
            if slab_start >= all.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("stage {} past end of cinematic.rom ({} bytes)",
                            args.stage, all.len()),
                ));
            }
            let end = slab_end.min(all.len());
            Ok(all[slab_start..end].to_vec())
        }
        "video2" => fs::read(args.romset.join("video2.rom")),
        _ => unreachable!(),
    }
}

// Suppress unused warnings for the convenience helper.
#[allow(dead_code)]
fn _placeholder(_: &Path) {}
