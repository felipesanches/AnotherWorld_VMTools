//! Disassembler for the Another World VM bytecode.
//!
//! Port of `awvm-disasm.py:AWVM_Trace + disasm_instruction`. Output
//! text is byte-for-byte the same as the Python reference for any
//! input the Python reference accepts.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use exectrace::{Disassembler, RelocationBlock, Tracer};

use crate::releases::ReleaseData;

const VIDEO2: u32 = 0;
const CINEMATIC: u32 = 1;

const SPECIAL_PURPOSE_VARS: &[(u8, &str)] = &[
    (0x3c, "RANDOM_SEED"),
    (0x54, "HACK_VAR_54"),
    (0x67, "HACK_VAR_67"),
    (0xda, "LAST_KEYCHAR"),
    (0xdc, "HACK_VAR_DC"),
    (0xe5, "HERO_POS_UP_DOWN"),
    (0xf4, "MUS_MARK"),
    (0xf7, "HACK_VAR_F7"),
    (0xf9, "SCROLL_Y"),
    (0xfa, "HERO_ACTION"),
    (0xfb, "HERO_POS_JUMP_DOWN"),
    (0xfc, "HERO_POS_LEFT_RIGHT"),
    (0xfd, "HERO_POS_MASK"),
    (0xfe, "HERO_ACTION_POS_MASK"),
    (0xff, "PAUSE_SLICES"),
];

fn special_var_name(v: u8) -> Option<&'static str> {
    for (k, name) in SPECIAL_PURPOSE_VARS {
        if *k == v {
            return Some(name);
        }
    }
    None
}

fn variable_name(value: u8) -> String {
    match special_var_name(value) {
        Some(name) => name.to_owned(),
        None => format!("0x{:02X}", value),
    }
}

#[derive(Debug, Clone)]
pub struct VideoEntry {
    pub palette_number: u8,
    pub x: String,
    pub y: String,
    pub zoom: String,
    pub label: String,
}

/// Insertion-order-preserving map from address to VideoEntry.
///
/// Python's `dict` preserves insertion order, and the listing
/// emitter iterates it in that order — so we need the same
/// behaviour here for byte-identical headers. A separate Vec keeps
/// the insertion order; the BTreeMap services lookups.
#[derive(Debug, Default)]
pub struct VideoEntryMap {
    entries: BTreeMap<u32, VideoEntry>,
    insertion_order: Vec<u32>,
}

impl VideoEntryMap {
    pub fn get(&self, key: &u32) -> Option<&VideoEntry> {
        self.entries.get(key)
    }
    pub fn contains_key(&self, key: &u32) -> bool {
        self.entries.contains_key(key)
    }
    pub fn insert(&mut self, key: u32, value: VideoEntry) {
        if !self.entries.contains_key(&key) {
            self.insertion_order.push(key);
        }
        self.entries.insert(key, value);
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Iterate in insertion order — matches Python's dict iteration.
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &VideoEntry)> {
        self.insertion_order
            .iter()
            .map(move |k| (k, &self.entries[k]))
    }
}

/// Cross-level state for the shared (a.k.a. `VIDEO2`) polygon
/// resource. Python accumulates `video2_entries` and
/// `video2_counter` at module scope across every level, so the
/// COMMON_VIDEO_NNN labels stay consistent (and accumulating)
/// throughout the run.
#[derive(Debug, Default)]
pub struct Video2Accumulator {
    pub counter: u32,
    pub entries: VideoEntryMap,
}

/// Per-game-level disassembler state.
pub struct AwvmDisassembler<'v> {
    pub game_level: u32,
    pub current_palette_number: u8,
    cinematic_counter: u32,
    pub cinematic_entries: VideoEntryMap,
    /// Shared across levels — see [`Video2Accumulator`].
    pub video2: &'v mut Video2Accumulator,
    /// Strings table data (`str_data.rom`).
    str_data: Vec<u8>,
    /// String index table (`str_index.rom`) — pairs of LE bytes per string id.
    str_index: Vec<u8>,
    /// Per-release data tables (KNOWN_LABELS, POSSIBLY_UNUSED_CODEBLOCKS,
    /// LABELED_CINEMATIC_ENTRIES, STAGE_TITLES). Plumbed through so the
    /// disassembler is release-agnostic.
    pub release: &'static ReleaseData,
}

impl<'v> AwvmDisassembler<'v> {
    pub fn new(
        game_level: u32,
        str_data: Vec<u8>,
        str_index: Vec<u8>,
        video2: &'v mut Video2Accumulator,
        release: &'static ReleaseData,
    ) -> Self {
        Self {
            game_level,
            current_palette_number: 0,
            cinematic_counter: 0,
            cinematic_entries: VideoEntryMap::default(),
            video2,
            str_data,
            str_index,
            release,
        }
    }

    fn known_label(&self, addr: u32) -> Option<&'static str> {
        for (lvl, labels) in self.release.known_labels {
            if *lvl == self.game_level {
                for (a, name) in *labels {
                    if *a == addr {
                        return Some(name);
                    }
                }
            }
        }
        None
    }

    fn is_unused_codeblock(&self, addr: u32) -> bool {
        for (lvl, addrs) in self.release.possibly_unused_codeblocks {
            if *lvl == self.game_level {
                for a in *addrs {
                    if *a == addr {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn labeled_cinematic(&self, addr: u32) -> Option<&'static str> {
        for (lvl, labels) in self.release.labeled_cinematic_entries {
            if *lvl == self.game_level {
                for (a, name) in *labels {
                    if *a == addr {
                        return Some(name);
                    }
                }
            }
        }
        None
    }

    /// Returns the human-readable label for an address, registering it
    /// with the tracer so the listing emitter prefixes it with the
    /// label declaration.
    fn label_name_register(&self, t: &mut Tracer, addr: u32) -> String {
        t.register_label(addr);
        if let Some(name) = self.known_label(addr) {
            return name.to_owned();
        }
        if self.is_unused_codeblock(addr) {
            return format!("JUNK__{:04X}", addr);
        }
        format!("LABEL_{:04X}", addr)
    }

    fn register_cinematic_entry(
        &mut self,
        x: &str,
        y: &str,
        palette_number: u8,
        zoom: &str,
        address: u32,
    ) -> String {
        if let Some(existing) = self.cinematic_entries.get(&address) {
            return existing.label.clone();
        }
        let label = match self.labeled_cinematic(address) {
            Some(name) => format!("CINEMATIC_{}", name),
            None => format!("CINEMATIC_{:03}", self.cinematic_counter),
        };
        self.cinematic_counter += 1;
        self.cinematic_entries.insert(
            address,
            VideoEntry {
                palette_number,
                x: x.to_owned(),
                y: y.to_owned(),
                zoom: zoom.to_owned(),
                label: label.clone(),
            },
        );
        label
    }

    fn register_video2_entry(
        &mut self,
        x: &str,
        y: &str,
        palette_number: u8,
        zoom: &str,
        address: u32,
    ) -> String {
        if let Some(existing) = self.video2.entries.get(&address) {
            return existing.label.clone();
        }
        let label = format!("COMMON_VIDEO_{:03}", self.video2.counter);
        self.video2.counter += 1;
        self.video2.entries.insert(
            address,
            VideoEntry {
                palette_number,
                x: x.to_owned(),
                y: y.to_owned(),
                zoom: zoom.to_owned(),
                label: label.clone(),
            },
        );
        label
    }

    fn get_text_string(&self, str_id: u32) -> String {
        let idx_off = (str_id as usize) * 2;
        if idx_off + 1 >= self.str_index.len() {
            return format!("string_{:04X}", str_id);
        }
        let lo = self.str_index[idx_off] as usize;
        let hi = self.str_index[idx_off + 1] as usize;
        let mut index = lo | (hi << 8);
        if index == 0 {
            return format!("string_{:04X}", str_id);
        }
        let mut out = String::new();
        while index < self.str_data.len() && self.str_data[index] != 0x00 {
            let c = self.str_data[index] as char;
            if c == '\n' {
                out.push_str("\\n");
            } else {
                out.push(c);
            }
            index += 1;
        }
        out
    }
}

impl<'v> Disassembler for AwvmDisassembler<'v> {
    fn output_disasm_headers(&self) -> String {
        let mut s = String::from("; Generated by AnotherWorld_VMTools\n");
        for (var, name) in SPECIAL_PURPOSE_VARS {
            s.push_str(&format!("{}\t\tEQU 0x{:02X}\n", name, var));
        }
        for (addr, v) in self.cinematic_entries.iter() {
            s.push_str(&format!("{}\t\tEQU 0x{:04X}\n", v.label, addr));
        }
        for (addr, v) in self.video2.entries.iter() {
            s.push_str(&format!("{}\t\tEQU 0x{:04X}\n", v.label, addr));
        }
        s
    }

    fn label_name(&self, _t: &Tracer, addr: u32) -> Option<String> {
        if let Some(name) = self.known_label(addr) {
            return Some(name.to_owned());
        }
        if self.is_unused_codeblock(addr) {
            return Some(format!("JUNK__{:04X}", addr));
        }
        Some(format!("LABEL_{:04X}", addr))
    }

    fn disasm_instruction(&mut self, t: &mut Tracer, opcode: u8) -> String {
        // Wrapper: the actual decoder lives in `do_disasm_instruction`.
        // After it runs, we harvest the per-instruction byte buffer
        // the tracer has been filling and append a `;@raw=...`
        // annotation so the assembler can round-trip the original
        // bytecode bit-for-bit, matching the Python reference.
        let text = self.do_disasm_instruction(t, opcode);
        let raw = match t.current_consumed_bytes() {
            Some(bs) if !bs.is_empty() => bs
                .iter()
                .map(|b| format!("0x{:02X}", b))
                .collect::<Vec<_>>()
                .join(","),
            _ => return text,
        };
        format!("{}\t;@raw={}", text, raw)
    }
}

impl<'v> AwvmDisassembler<'v> {
    fn do_disasm_instruction(&mut self, t: &mut Tracer, opcode: u8) -> String {
        if opcode & 0x80 != 0 {
            // VIDEO (opcode high bit set)
            let lo = t.fetch();
            let hi = opcode & 0x7F;
            let offset = ((u32::from(hi) << 8) | u32::from(lo)) * 2;
            let x = t.fetch();
            let y = t.fetch();
            let palette = self.current_palette_number;
            let label = self.register_cinematic_entry(
                &format!("{}", x),
                &format!("{}", y),
                palette,
                "0x40",
                offset,
            );
            return format!(
                "video type={}, offset={}, x={}, y={}",
                CINEMATIC, label, x, y
            );
        }

        if opcode & 0x40 != 0 {
            // VIDEO (alt encoding)
            let off1 = t.fetch();
            let off2 = t.fetch();
            let offset = ((u32::from(off1 & 0x7F) << 8) | u32::from(off2)) * 2;

            let x_str: String;
            let mut x: u32 = u32::from(t.fetch());
            if opcode & 0x20 == 0 {
                if opcode & 0x10 == 0 {
                    let extra = t.fetch();
                    x = (x << 8) | u32::from(extra);
                    x_str = format!("{}", x);
                } else {
                    x_str = format!("[0x{:02x}]", x);
                }
            } else {
                if opcode & 0x10 != 0 {
                    x += 0x100;
                }
                x_str = format!("{}", x);
            }

            let y_str: String;
            if opcode & 8 == 0 {
                if opcode & 4 == 0 {
                    let mut y: u32 = u32::from(t.fetch());
                    let extra = t.fetch();
                    y = (y << 8) | u32::from(extra);
                    y_str = format!("{}", y);
                } else {
                    let v = t.fetch();
                    y_str = format!("[0x{:02x}]", v);
                }
            } else {
                let v = t.fetch();
                y_str = format!("{}", v);
            }

            let zoom_str: String;
            if opcode & 2 == 0 {
                if opcode & 1 == 0 {
                    zoom_str = "0x40".to_owned();
                } else {
                    let v = t.fetch();
                    zoom_str = format!("[0x{:02x}]", v);
                }
            } else {
                if opcode & 1 != 0 {
                    zoom_str = "0x40".to_owned();
                } else {
                    let v = t.fetch();
                    zoom_str = format!("[0x{:02x}]", v);
                }
            }

            let palette = self.current_palette_number;
            if opcode & 3 == 3 {
                let label =
                    self.register_video2_entry(&x_str, &y_str, palette, &zoom_str, offset);
                return format!(
                    "video type={}, offset={}, x={}, y={}, zoom={}",
                    VIDEO2, label, x_str, y_str, zoom_str
                );
            } else {
                let label =
                    self.register_cinematic_entry(&x_str, &y_str, palette, &zoom_str, offset);
                return format!(
                    "video type={}, offset={}, x={}, y={}, zoom={}",
                    CINEMATIC, label, x_str, y_str, zoom_str
                );
            }
        }

        match opcode {
            0x00 => {
                let dst_var = variable_name(t.fetch());
                let imm_hi = t.fetch();
                let imm_lo = t.fetch();
                let imm = (u32::from(imm_hi) << 8) | u32::from(imm_lo);
                format!("mov [{}], 0x{:04X}", dst_var, imm)
            }
            0x01 => {
                let dst_var = variable_name(t.fetch());
                let src_var = variable_name(t.fetch());
                format!("mov [{}], [{}]", dst_var, src_var)
            }
            0x02 => {
                let dst_var = variable_name(t.fetch());
                let src_var = variable_name(t.fetch());
                format!("add [{}], [{}]", dst_var, src_var)
            }
            0x03 => {
                let dst_var = variable_name(t.fetch());
                let imm_hi = t.fetch();
                let imm_lo = t.fetch();
                let imm = (u32::from(imm_hi) << 8) | u32::from(imm_lo);
                if imm >= 0x8000 {
                    format!("sub [{}], 0x{:04X}", dst_var, 0x10000 - imm)
                } else {
                    format!("add [{}], 0x{:04X}", dst_var, imm)
                }
            }
            0x04 => {
                let hi = t.fetch();
                let lo = t.fetch();
                let address = (u32::from(hi) << 8) | u32::from(lo);
                t.subroutine(address);
                format!("call {}", self.label_name_register(t, address))
            }
            0x05 => {
                t.return_from_subroutine();
                "ret".to_owned()
            }
            0x06 => "break".to_owned(),
            0x07 => {
                let hi = t.fetch();
                let lo = t.fetch();
                let address = (u32::from(hi) << 8) | u32::from(lo);
                t.unconditional_jump(address);
                format!("jmp {}", self.label_name_register(t, address))
            }
            0x08 => {
                let thread_id = t.fetch();
                let hi = t.fetch();
                let lo = t.fetch();
                let pc_offset = (u32::from(hi) << 8) | u32::from(lo);
                t.schedule_entry_point(pc_offset, true);
                format!(
                    "setup channel=0x{:02X}, address={}",
                    thread_id,
                    self.label_name_register(t, pc_offset)
                )
            }
            0x09 => {
                let var = t.fetch();
                let hi = t.fetch();
                let lo = t.fetch();
                let offset = (u32::from(hi) << 8) | u32::from(lo);
                let var_name = variable_name(var);
                t.conditional_branch(offset);
                format!(
                    "djnz [{}], {}",
                    var_name,
                    self.label_name_register(t, offset)
                )
            }
            0x0a => {
                let subop = t.fetch();
                let b = t.fetch();
                let c = t.fetch();
                let var1 = variable_name(b);
                let midterm: String;
                if subop & 0x80 != 0 {
                    let var2 = variable_name(c);
                    midterm = format!("[{}]", var2);
                } else if subop & 0x40 != 0 {
                    let extra = t.fetch();
                    midterm = format!("0x{:04X}", (u32::from(c) << 8) | u32::from(extra));
                } else {
                    midterm = format!("0x{:02X}", c);
                }
                let off_hi = t.fetch();
                let off_lo = t.fetch();
                let offset = (u32::from(off_hi) << 8) | u32::from(off_lo);

                let condition = subop & 7;
                let mnemonic = match condition {
                    0 => "je",
                    1 => "jne",
                    2 => "jg",
                    3 => "jge",
                    4 => "jl",
                    5 => "jle",
                    other => {
                        return format!(
                            "; DISASM ERROR! Conditional JMP instruction with invalid condition ({})",
                            other
                        );
                    }
                };

                t.conditional_branch(offset);
                format!(
                    "{} [{}], {}, {}",
                    mnemonic,
                    var1,
                    midterm,
                    self.label_name_register(t, offset)
                )
            }
            0x0b => {
                let palette_id = t.fetch();
                let _ = t.fetch(); // waste byte
                self.current_palette_number = palette_id;
                format!("setPalette 0x{:02X}", palette_id)
            }
            0x0c => {
                let first = t.fetch();
                let last = t.fetch();
                let typ = t.fetch();
                let names = ["freezeChannels", "unfreezeChannels", "deleteChannels"];
                if typ as usize > 2 {
                    "< invalid operation type for resetThread opcode >".to_owned()
                } else {
                    format!(
                        "{} first=0x{:02X}, last=0x{:02X}",
                        names[typ as usize],
                        first,
                        last
                    )
                }
            }
            0x0d => {
                let id = t.fetch();
                format!("selectVideoPage 0x{:02X}", id)
            }
            0x0e => {
                let page = t.fetch();
                let color = t.fetch();
                format!("fill page=0x{:02X}, color=0x{:02X}", page, color)
            }
            0x0f => {
                let src = t.fetch();
                let dst = t.fetch();
                format!("copyVideoPage src=0x{:02X}, dst=0x{:02X}", src, dst)
            }
            0x10 => {
                let page = t.fetch();
                format!("blitFramebuffer 0x{:02X}", page)
            }
            0x11 => "killChannel".to_owned(),
            0x12 => {
                let hi = t.fetch();
                let lo = t.fetch();
                let str_id = (u32::from(hi) << 8) | u32::from(lo);
                let x = t.fetch();
                let y = t.fetch();
                let color = t.fetch();
                let text = self.get_text_string(str_id);
                format!(
                    "text id=0x{:04X}, x={}, y={}, color=0x{:02X} ; \"{}\"",
                    str_id, x, y, color, text
                )
            }
            0x13 => {
                let v1 = variable_name(t.fetch());
                let v2 = variable_name(t.fetch());
                format!("sub [{}], [{}]", v1, v2)
            }
            0x14 => {
                let dst = variable_name(t.fetch());
                let hi = t.fetch();
                let lo = t.fetch();
                let imm = (u32::from(hi) << 8) | u32::from(lo);
                format!("and [{}], 0x{:04X}", dst, imm)
            }
            0x15 => {
                let dst = variable_name(t.fetch());
                let hi = t.fetch();
                let lo = t.fetch();
                let imm = (u32::from(hi) << 8) | u32::from(lo);
                format!("or [{}], 0x{:04X}", dst, imm)
            }
            0x16 => {
                let var = variable_name(t.fetch());
                let hi = t.fetch();
                let lo = t.fetch();
                let imm = (u32::from(hi) << 8) | u32::from(lo);
                format!("shl [{}], 0x{:04X}", var, imm)
            }
            0x17 => {
                let var = variable_name(t.fetch());
                let hi = t.fetch();
                let lo = t.fetch();
                let imm = (u32::from(hi) << 8) | u32::from(lo);
                format!("shr [{}], 0x{:04X}", var, imm)
            }
            0x18 => {
                let hi = t.fetch();
                let lo = t.fetch();
                let res = (u32::from(hi) << 8) | u32::from(lo);
                let freq = t.fetch();
                let vol = t.fetch();
                let chan = t.fetch();
                format!(
                    "play id=0x{:04X}, freq=0x{:02X}, vol=0x{:02X}, channel=0x{:02X}",
                    res, freq, vol, chan
                )
            }
            0x19 => {
                let hi = t.fetch();
                let lo = t.fetch();
                let imm = (u32::from(hi) << 8) | u32::from(lo);
                let nibble = (imm & 0xf) as usize;
                if imm > 0x100 && nibble < self.release.stage_titles.len() {
                    if imm & 0xfff0 != 0x3E80 {
                        // Print the warning to stderr to match the Python's stdout
                        // print(); we route this via eprintln! to keep parity with
                        // the Python's print(...) on stdout.
                        // But Python prints to stdout via print()... hmm, parity.
                        // Match Python: emit on stdout via println!.
                        println!(
                            "WARN: Found an instance of the load instruction indicating a bankSwitch but with an uncommon value of {:04X} in its operands.\nExpected to see {:04X} instead.",
                            imm,
                            0x3E80 | (imm & 0xf)
                        );
                    }
                    format!(
                        "bankSwitch {};  {}",
                        imm & 0xf,
                        self.release.stage_titles[nibble]
                    )
                } else {
                    format!("load id=0x{:04X}", imm)
                }
            }
            0x1a => {
                let hi = t.fetch();
                let lo = t.fetch();
                let res = (u32::from(hi) << 8) | u32::from(lo);
                let dhi = t.fetch();
                let dlo = t.fetch();
                let delay = (u32::from(dhi) << 8) | u32::from(dlo);
                let pos = t.fetch();
                format!(
                    "song id=0x{:04X}, delay=0x{:04X}, pos=0x{:02X}",
                    res, delay, pos
                )
            }
            0x1b => "GameOver".to_owned(),
            other => {
                t.illegal_instruction(other);
                format!(
                    "; DISASM ERROR! Illegal instruction (opcode = 0x{:02X})",
                    other
                )
            }
        }
    }
}

/// Disassemble one game level from a `bytecode.rom`. Mirrors the
/// per-level loop body in `awvm-disasm.py`.
///
/// `video2` is the cross-level shared state that accumulates over the
/// lifetime of a multi-level run (matches Python's module-global
/// `video2_entries` / `video2_counter`).
pub fn disassemble_level<'v>(
    gamerom_path: &Path,
    game_level: u32,
    str_data_path: &Path,
    str_index_path: &Path,
    out_asm_path: &Path,
    video2: &'v mut Video2Accumulator,
    release: &'static ReleaseData,
) -> io::Result<AwvmDisassembler<'v>> {
    let str_data = fs::read(str_data_path)?;
    let str_index = fs::read(str_index_path)?;

    let relocation = vec![RelocationBlock {
        from_offset: u64::from(0x10000u32 * game_level),
        to_address: 0,
        length: 0x10000,
    }];

    let mut tracer = Tracer::from_file_with_relocation(gamerom_path, relocation)?;

    // POSSIBLY_UNUSED_CODEBLOCKS: schedule each as a subroutine entry
    // point — matches the Python's `subroutines=POSSIBLY_UNUSED_CODEBLOCKS.get(...)`.
    for (lvl, addrs) in release.possibly_unused_codeblocks {
        if *lvl == game_level {
            for a in *addrs {
                tracer.schedule_entry_point(*a, true);
            }
        }
    }

    let mut dis = AwvmDisassembler::new(game_level, str_data, str_index, video2, release);
    tracer.run(&mut dis, &[0x0000]);

    if let Some(parent) = out_asm_path.parent() {
        fs::create_dir_all(parent)?;
    }
    tracer.save_disassembly_listing(&dis, out_asm_path)?;

    Ok(dis)
}
