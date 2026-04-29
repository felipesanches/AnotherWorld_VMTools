//! Polygon decoder for the Another World VM.
//!
//! Port of `releases/common_data/decode_polygons.py`. The decoder
//! reads a per-entry record from the cinematic / video2 polygon
//! banks, walks the (possibly hierarchical) shape, and emits an SVG
//! representation. Per the project agreement, the SVG output is
//! "semantically equivalent" to Python's pycairo output rather than
//! byte-for-byte identical: same shapes, same colours, same canvas
//! size, but the raw SVG bytes may differ in attribute ordering,
//! formatting, etc.
//!
//! The algorithm itself is faithful to the Python original — same
//! polygon-byte semantics, same coordinate math, same hierarchical
//! recursion.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::disasm::VideoEntry;

const COLOR_BLACK: u8 = 0xFF;
const DEFAULT_ZOOM: f64 = 0x40 as f64;
const MAX_POINTS: u32 = 50;

/// State for a single polygon-extraction run (one game level + one
/// kind of polygon bank).
pub struct PolygonDecoder {
    palette_data: Vec<u8>,
    polygon_data: Vec<u8>,
    game_level: u32,
    pdata_offset: u32,
}

impl PolygonDecoder {
    /// Build a decoder for cinematic-style entries (`cinematic.rom`
    /// + `palettes.rom`). `cinematic.rom` is laid out as nine
    /// 64-KiB level slabs, indexed by `game_level`.
    pub fn for_cinematic(romset_dir: &Path, game_level: u32) -> io::Result<Self> {
        Ok(Self {
            palette_data: fs::read(romset_dir.join("palettes.rom"))?,
            polygon_data: fs::read(romset_dir.join("cinematic.rom"))?,
            game_level,
            pdata_offset: 0,
        })
    }

    /// Build a decoder for video2-style entries (`video2.rom` —
    /// the single shared polygon bank that any level may reference).
    pub fn for_video2(romset_dir: &Path) -> io::Result<Self> {
        Ok(Self {
            palette_data: fs::read(romset_dir.join("palettes.rom"))?,
            polygon_data: fs::read(romset_dir.join("video2.rom"))?,
            game_level: 0,
            pdata_offset: 0,
        })
    }

    /// Extract every entry in `entries` into an SVG file at
    /// `out_dir/<label>.svg`. Returns the list of paths written.
    pub fn extract(
        &mut self,
        entries: impl IntoIterator<Item = (u32, VideoEntry)>,
        out_dir: &Path,
    ) -> io::Result<Vec<PathBuf>> {
        fs::create_dir_all(out_dir)?;
        let mut written = Vec::new();
        for (addr, entry) in entries {
            // Match Python's "HACK!" fallbacks for non-numeric x/y/zoom.
            let zoom = parse_int_else(&entry.zoom, 0x40);
            let x = parse_int_else(&entry.x, 160);
            let y = parse_int_else(&entry.y, 100);

            let mut paths = Vec::new();
            self.read_and_draw_polygon(
                addr,
                entry.palette_number,
                COLOR_BLACK,
                zoom,
                x as f64,
                y as f64,
                &mut paths,
            );

            let svg_path = out_dir.join(format!("{}.svg", entry.label));
            write_svg(&svg_path, &paths)?;
            written.push(svg_path);
        }
        Ok(written)
    }

    // --- algorithm ---------------------------------------------------

    fn fetch_polygon_data(&mut self) -> u8 {
        let idx = ((self.game_level as usize) << 16) | (self.pdata_offset as usize);
        let v = self
            .polygon_data
            .get(idx)
            .copied()
            .expect("pdata_offset out of bounds");
        self.pdata_offset = self.pdata_offset.wrapping_add(1);
        v
    }

    fn get_color_from_palette(&self, palette_number: u8, color: u8) -> (f64, f64, f64) {
        let p = ((self.game_level << 11) as usize)
            | (32 * palette_number as usize + 2 * (color as usize % 16));
        let c1 = self.palette_data[p];
        let c2 = self.palette_data[p + 1];
        let r = ((c1 & 0x0F) << 2) | ((c1 & 0x0F) >> 2);
        let g = ((c2 & 0xF0) >> 2) | ((c2 & 0xF0) >> 6);
        let b = ((c2 & 0x0F) >> 2) | ((c2 & 0x0F) << 2);
        (r as f64 / 64.0, g as f64 / 64.0, b as f64 / 64.0)
    }

    fn fill_polygon(
        &mut self,
        palette_number: u8,
        color: u8,
        zoom: i64,
        cx: f64,
        cy: f64,
        out: &mut Vec<SvgPath>,
    ) {
        let (r, g, b) = self.get_color_from_palette(palette_number, color);
        let bbox_w = self.fetch_polygon_data() as f64 * zoom as f64 / DEFAULT_ZOOM;
        let bbox_h = self.fetch_polygon_data() as f64 * zoom as f64 / DEFAULT_ZOOM;
        let num_points = self.fetch_polygon_data() as u32;

        if num_points & 1 != 0 || num_points >= MAX_POINTS {
            // Match Python: this is a hard error in the reference.
            // We fail more gracefully (skip the polygon) since we are
            // a library; the caller can decide what to do.
            return;
        }

        let mut pt_x = Vec::with_capacity(num_points as usize);
        let mut pt_y = Vec::with_capacity(num_points as usize);
        for _ in 0..num_points {
            let x = self.fetch_polygon_data() as f64 * zoom as f64 / DEFAULT_ZOOM;
            let y = self.fetch_polygon_data() as f64 * zoom as f64 / DEFAULT_ZOOM;
            pt_x.push(cx - bbox_w / 2.0 + x);
            pt_y.push(cy - bbox_h / 2.0 + y);
        }

        // Python heuristic: a 4-point degenerate polygon that's
        // really a line gets bumped to a 1-px-wide lozenge so the
        // SVG viewer renders it.
        if num_points == 4
            && pt_x[0] == pt_x[3]
            && pt_x[1] == pt_x[2]
            && pt_y[0] == pt_y[3]
            && pt_y[1] == pt_y[2]
        {
            pt_x[2] += 2.0;
            pt_x[3] += 2.0;
        }

        out.push(SvgPath {
            color: (r, g, b),
            points: pt_x.into_iter().zip(pt_y).collect(),
        });
    }

    fn read_and_draw_polygon(
        &mut self,
        address: u32,
        palette_number: u8,
        color: u8,
        zoom: i64,
        x: f64,
        y: f64,
        out: &mut Vec<SvgPath>,
    ) {
        self.pdata_offset = address;
        let value = self.fetch_polygon_data();

        if value >= 0xC0 {
            let effective_color = if color & 0x80 != 0 {
                value & 0x3F
            } else {
                color
            };
            let backup = self.pdata_offset;
            self.fill_polygon(palette_number, effective_color, zoom, x, y, out);
            self.pdata_offset = backup;
        } else {
            let value = value & 0x3F;
            if value == 2 {
                self.read_and_draw_polygon_hierarchy(palette_number, zoom, x, y, out);
            } else {
                // Match Python's "ERROR" behaviour minus the sys.exit;
                // again, we are a library — bail on this polygon.
            }
        }
    }

    fn read_and_draw_polygon_hierarchy(
        &mut self,
        palette_number: u8,
        zoom: i64,
        pgc_x: f64,
        pgc_y: f64,
        out: &mut Vec<SvgPath>,
    ) {
        let pt_x = pgc_x - (self.fetch_polygon_data() as f64 * zoom as f64 / DEFAULT_ZOOM);
        let pt_y = pgc_y - (self.fetch_polygon_data() as f64 * zoom as f64 / DEFAULT_ZOOM);
        let num_children = self.fetch_polygon_data() as u32 + 1;

        for _ in 0..num_children {
            let off_hi = self.fetch_polygon_data() as u32;
            let off_lo = self.fetch_polygon_data() as u32;
            let offset = (off_hi << 8) | off_lo;

            let po_x = pt_x + (self.fetch_polygon_data() as f64 * zoom as f64 / DEFAULT_ZOOM);
            let po_y = pt_y + (self.fetch_polygon_data() as f64 * zoom as f64 / DEFAULT_ZOOM);

            let mut color = 0xFFu8;
            if offset & 0x8000 != 0 {
                color = self.fetch_polygon_data() & 0x7F;
                let _ = self.fetch_polygon_data(); // waste a byte (Python parity)
            }

            let backup = self.pdata_offset;
            self.read_and_draw_polygon(
                (offset & 0x7FFF) * 2,
                palette_number,
                color,
                zoom,
                po_x,
                po_y,
                out,
            );
            self.pdata_offset = backup;
        }
    }
}

/// One filled polygon as it will appear in the SVG.
struct SvgPath {
    color: (f64, f64, f64),
    points: Vec<(f64, f64)>,
}

fn write_svg(path: &Path, paths: &[SvgPath]) -> io::Result<()> {
    let mut f = fs::File::create(path)?;
    writeln!(f, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        f,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="320" height="200" viewBox="0 0 320 200">"#
    )?;
    for p in paths {
        if p.points.is_empty() {
            continue;
        }
        let (r, g, b) = p.color;
        let hex = format!(
            "#{:02x}{:02x}{:02x}",
            (r * 255.0).round().clamp(0.0, 255.0) as u8,
            (g * 255.0).round().clamp(0.0, 255.0) as u8,
            (b * 255.0).round().clamp(0.0, 255.0) as u8
        );
        write!(f, r#"  <path fill="{}" d=""#, hex)?;
        for (i, (x, y)) in p.points.iter().enumerate() {
            if i == 0 {
                write!(f, "M{:.3} {:.3}", x, y)?;
            } else {
                write!(f, " L{:.3} {:.3}", x, y)?;
            }
        }
        writeln!(f, " Z\"/>")?;
    }
    writeln!(f, "</svg>")?;
    Ok(())
}

fn parse_int_else(s: &str, fallback: i64) -> i64 {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).unwrap_or(fallback)
    } else {
        s.parse::<i64>().unwrap_or(fallback)
    }
}
