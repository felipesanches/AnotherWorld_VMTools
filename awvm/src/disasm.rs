//! Disassembler for the Another World VM bytecode.
//!
//! Port of `awvm-disasm.py:AWVM_Trace + disasm_instruction`. Output
//! text is byte-for-byte the same as the Python reference for any
//! input the Python reference accepts.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::Path;

use exectrace::{Disassembler, RelocationBlock, Tracer};

use crate::releases::ReleaseData;

const VIDEO2: u32 = 0;

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
    /// Cross-port registry with chain-walking label/comment lookup.
    /// Used for `known_label` / `labeled_cinematic` resolution so
    /// sibling ports inherit names from canonical via their
    /// translation cohorts. See `crate::registry`.
    pub registry: &'static crate::registry::PortRegistry,
    /// Which address currently owns each resolved label name, for this
    /// level only. The chain walk can map two sibling addresses onto one
    /// canonical address (two cohorts shifting onto the same target, or a
    /// shift plus implicit identity), which made the emitter declare the
    /// same label twice; the assembler then keeps one definition per name
    /// and every reference collapsed onto a single address, silently
    /// corrupting the other. See archaeology issue #0118.
    ///
    /// First claimant keeps the name; later addresses fall back to a
    /// `LABEL_`/`JUNK__` placeholder, which is honest — nobody has named
    /// that second instance yet — and keeps the round-trip byte-identical.
    name_owner: RefCell<HashMap<&'static str, u32>>,
    /// `(name, owner_addr, rejected_addr)` for every rejection above, so
    /// the caller can report the naming debt instead of hiding it.
    name_collisions: RefCell<Vec<(&'static str, u32, u32)>>,
    /// Same rule for cinematic EQU names, which collide for the same reason
    /// and with a worse effect: two polygon offsets resolving to one name
    /// emit two `NAME EQU 0x…` lines with different values, and the
    /// assembler's last-definition-wins makes every `video offset=NAME`
    /// assemble to one of them. Maps a claimed cinematic label to the
    /// polygon offset that owns it. See #0118 and #0107.
    cinematic_labels_used: HashMap<String, u32>,
    /// `(name, owner_offset, rejected_offset)` for cinematic rejections.
    cinematic_name_collisions: Vec<(String, u32, u32)>,
}

/// Outcome of claiming a label name for an address within one level.
#[derive(Debug, PartialEq, Eq)]
pub enum Claim {
    /// This address owns the name (it just claimed it, or already held it).
    Held,
    /// Another address owns it; the caller must fall back to a placeholder.
    TakenBy(u32),
}

/// One name, one address, per level.
///
/// The cross-port chain walk can map two sibling addresses onto the same
/// canonical address and therefore the same name. Emitting it twice made the
/// assembler keep a single definition, so every reference collapsed onto one
/// address and the other assembled to the wrong target — 71 corrupted bytes
/// across four MS-DOS levels before this rule existed (#0118). First claimant
/// wins; the rule is deterministic because trace order is.
pub fn claim_name(
    owners: &mut HashMap<&'static str, u32>,
    name: &'static str,
    addr: u32,
) -> Claim {
    match owners.get(name).copied() {
        Some(owner) if owner == addr => Claim::Held,
        Some(owner) => Claim::TakenBy(owner),
        None => {
            owners.insert(name, addr);
            Claim::Held
        }
    }
}

impl<'v> AwvmDisassembler<'v> {
    pub fn new(
        game_level: u32,
        str_data: Vec<u8>,
        str_index: Vec<u8>,
        video2: &'v mut Video2Accumulator,
        release: &'static ReleaseData,
        registry: &'static crate::registry::PortRegistry,
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
            registry,
            name_owner: RefCell::new(HashMap::new()),
            name_collisions: RefCell::new(Vec::new()),
            cinematic_labels_used: HashMap::new(),
            cinematic_name_collisions: Vec::new(),
        }
    }

    /// Stage-aware variable name. Consults the registry's
    /// per-stage `variable_aliases` first; falls back to the
    /// global `SPECIAL_PURPOSE_VARS` table; last resort is the
    /// literal `[0xNN]` form.
    ///
    /// (Variable aliases are not currently chain-walked through
    /// the canonical-port registry — they're per-port stable in
    /// practice. Could become chain-walked if a sibling port
    /// inherits from canonical's per-stage var conventions; not
    /// needed today.)
    fn variable_name(&self, value: u8) -> String {
        if let Some(stage) = self.registry.stage(self.game_level) {
            for (var, name) in stage.variable_aliases {
                if *var == value {
                    return (*name).to_owned();
                }
            }
        }
        match special_var_name(value) {
            Some(name) => name.to_owned(),
            None => format!("0x{:02X}", value),
        }
    }

    fn known_label(&self, addr: u32) -> Option<&'static str> {
        // Chain-walks via translation cohorts: sibling ports
        // inherit labels from canonical (amiga) when their
        // translation_to_parent table maps the address into a
        // canonical address that has a name. Local labels
        // always take precedence.
        let name = self.registry.resolve_label(self.game_level, addr)?;

        // A name may be claimed by exactly one address per level (#0118).
        match claim_name(&mut self.name_owner.borrow_mut(), name, addr) {
            Claim::Held => Some(name),
            Claim::TakenBy(owner) => {
                self.name_collisions
                    .borrow_mut()
                    .push((name, owner, addr));
                None
            }
        }
    }

    /// `(name, owner_offset, rejected_offset)` cinematic-name collisions for
    /// this level. Non-empty means two polygon offsets resolve to one name;
    /// the loser falls back to its numeric `CINEMATIC_NNN`. See #0118/#0107.
    pub fn cinematic_name_collisions(&self) -> &[(String, u32, u32)] {
        &self.cinematic_name_collisions
    }

    /// Deduplicated `(name, owner_addr, rejected_addr)` list of label-name
    /// collisions seen while disassembling this level. Non-empty means the
    /// registry maps two addresses onto one canonical name; each entry is a
    /// routine instance that still needs its own name. See issue #0118.
    pub fn label_name_collisions(&self) -> Vec<(&'static str, u32, u32)> {
        let mut v = self.name_collisions.borrow().clone();
        v.sort_unstable();
        v.dedup();
        v
    }

    fn is_unused_codeblock(&self, addr: u32) -> bool {
        // possibly_unused_codeblocks is currently per-port (not
        // chain-walked). Could plug into resolve_* if there's
        // ever a canonical-vs-sibling distinction; for now
        // every port lists its own set.
        if let Some(stage) = self.registry.stage(self.game_level) {
            for a in stage.possibly_unused_codeblocks {
                if *a == addr {
                    return true;
                }
            }
        }
        false
    }

    fn labeled_cinematic(&self, addr: u32) -> Option<&'static str> {
        // Chain-walked lookup via the polygon-address
        // translation cohorts in each sibling's
        // `cinematic_translation_to_parent` table. Local
        // entries (the sibling's own port-exclusive names) take
        // precedence; on a miss, the address is translated to
        // the canonical port (amiga) via the polygon cohorts
        // and looked up there.
        //
        // The polygon address space is distinct from the
        // bytecode address space, so we cannot reuse
        // `translation_to_parent` (byte-address cohorts) for
        // this — applying byte-addr translation to polygon
        // offsets returns wrong addresses. The separate
        // `cinematic_translation_to_parent` field captures the
        // polygon-renumbering pattern (research/22, #0080) as
        // shift cohorts.
        //
        // Per archaeology issue #0105, shared cinematic names
        // are deduped into amiga.toml (the canonical); siblings
        // hold only port-exclusive cinematic entries plus the
        // polygon translation table.
        self.registry.resolve_cinematic_entry(self.game_level, addr)
    }

    /// Returns the human-readable label for an address, registering it
    /// with the tracer so the listing emitter prefixes it with the
    /// label declaration.
    /// A video-page operand: the registry's symbol when the value has one,
    /// otherwise a hex literal. Only exact values are named — `copyVideoPage src`
    /// carries flag bits on top of a page number, and naming a composite would
    /// claim more than is known (see docs/NAMING.md §6).
    /// A `setup channel=` operand: the registry's symbol when the channel's role
    /// is port- and stage-invariant, otherwise a hex literal.
    fn channel(&self, v: u8) -> String {
        match self.registry.channel_name(v) {
            Some(n) => n.to_owned(),
            None => format!("0x{:02X}", v),
        }
    }

    /// A music resource id: symbolic where the track has been identified.
    fn music(&self, v: u16) -> String {
        match self.registry.music_name(v) {
            Some(n) => n.to_owned(),
            None => format!("0x{:04X}", v),
        }
    }

    fn page(&self, v: u8) -> String {
        match self.registry.video_page_name(v) {
            Some(n) => n.to_owned(),
            None => format!("0x{:02X}", v),
        }
    }

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
        // A cinematic name may be claimed by exactly one polygon offset per
        // level. Two offsets resolving to one name would emit two EQU lines
        // with different values, and every reference would assemble to
        // whichever the assembler saw last (#0118).
        let numeric = format!("CINEMATIC_{:03}", self.cinematic_counter);
        let label = match self.labeled_cinematic(address) {
            Some(name) => {
                let candidate = format!("CINEMATIC_{}", name);
                match self.cinematic_labels_used.get(&candidate).copied() {
                    Some(owner) => {
                        self.cinematic_name_collisions.push((
                            candidate,
                            owner,
                            address,
                        ));
                        numeric
                    }
                    None => candidate,
                }
            }
            None => numeric,
        };
        self.cinematic_labels_used.insert(label.clone(), address);
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
        // Stage prologue: what this stage IS, in prose, once at the top. The
        // hand-maintained sources carry such a header already; this is how a
        // freshly generated listing gets one too (#0099).
        if let Some(lines) = self.registry.resolve_prologue(self.game_level) {
            s.push_str(";\n");
            for l in lines {
                if l.is_empty() {
                    s.push_str(";\n");
                } else {
                    s.push_str("; ");
                    s.push_str(l);
                    s.push('\n');
                }
            }
            s.push_str(";\n");
        }
        for (var, name) in SPECIAL_PURPOSE_VARS {
            s.push_str(&format!("{}\t\tEQU 0x{:02X}\n", name, var));
        }
        // Page symbols must be defined before use: the assembler resolves an
        // unknown symbol to 0 without complaining, so a missing EQU here would
        // change bytes silently rather than fail.
        for (v, name) in self.registry.video_page_names_effective() {
            s.push_str(&format!("{}\t\tEQU 0x{:02X}\n", name, v));
        }
        for (v, name) in self.registry.channel_names_effective() {
            s.push_str(&format!("{}\t\tEQU 0x{:02X}\n", name, v));
        }
        for (v, name) in self.registry.music_names_effective() {
            s.push_str(&format!("{}\t\tEQU 0x{:04X}\n", name, v));
        }
        if let Some(stage) = self.registry.stage(self.game_level) {
            for (var, name) in stage.variable_aliases {
                s.push_str(&format!("{}\t\tEQU 0x{:02X}\n", name, var));
            }
        }
        for (addr, v) in self.cinematic_entries.iter() {
            s.push_str(&format!("{}\t\tEQU 0x{:04X}\n", v.label, addr));
        }
        for (addr, v) in self.video2.entries.iter() {
            s.push_str(&format!("{}\t\tEQU 0x{:04X}\n", v.label, addr));
        }
        s
    }

    fn label_prologue(&self, addr: u32) -> Option<String> {
        // Comment-only and therefore byte-neutral: the listing gains prose, the
        // assembled output cannot change. Emitted only where a label is actually
        // declared, so a comment on an address the tracer never labels is simply
        // not shown rather than silently dropped somewhere odd.
        let lines = self.registry.resolve_comment(self.game_level, addr)?;
        if lines.is_empty() {
            return None;
        }
        let mut s = String::new();
        for l in lines {
            if l.is_empty() {
                s.push_str(";\n");
            } else {
                s.push_str("; ");
                s.push_str(l);
                s.push('\n');
            }
        }
        Some(s)
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
        // After decoding, classify the instruction's BYTE FORM so we
        // can emit a precise round-trip annotation:
        //   - Canonical encoding → no annotation needed (the
        //     assembler reproduces the bytes from the source mnemonic
        //     alone).
        //   - Known non-canonical encoding → `;@enc=<NAME>` marker
        //     (or, for setPalette palette-0 trailing-0x00 case, an
        //     extra `_trailing=0x00` operand baked into the source
        //     text).
        //   - Unknown form → fall back to `;@raw=…` so byte-match
        //     never breaks. Once every known pattern is enumerated,
        //     the fallback can be removed.
        let text = self.do_disasm_instruction(t, opcode);
        let raw = match t.current_consumed_bytes() {
            Some(bs) if !bs.is_empty() => bs.to_vec(),
            _ => return text,
        };
        classify_encoding(&text, &raw)
    }
}

/// Decide which round-trip annotation (if any) is needed for an
/// instruction whose source-form is `text` and whose original bytes
/// are `raw`. Returns either `text` unchanged (canonical), `text`
/// with a `;@enc=…` suffix appended, or `text` with a `_trailing`
/// operand inserted (for setPalette).
///
/// The four catalogued non-canonical patterns are listed below.
/// Anything else is assumed to round-trip canonically — the
/// assembler reproduces the bytes from the source mnemonic +
/// operands alone. If a fresh extraction surfaces a fifth
/// non-canonical pattern, byte-match verification will fail and
/// the new pattern needs a new arm here. We deliberately do NOT
/// emit `;@raw=…` as a fallback: that mechanism was retired in
/// 2026-05-04 and is now a hard parse error in `awvm-asm`.
fn classify_encoding(text: &str, raw: &[u8]) -> String {
    // Pattern 1: video zoom-as-var with bit-1 instead of bit-0.
    // Opcodes in the 0x40-0x7F range with `(opcode & 0x03) == 2`
    // decode "zoom = var" via the alt path. Source text always
    // starts with `video` and contains `zoom=[`.
    if !raw.is_empty() {
        let op = raw[0];
        if (0x40..=0x7F).contains(&op)
            && (op & 0x03) == 0x02
            && text.trim_start().starts_with("video")
            && text.contains("zoom=[")
        {
            return format!("{}\t;@enc=alt", text);
        }
    }

    // Pattern 2: bankSwitch with non-canonical operand word.
    // Opcode 0x19, operand word in 0x07Dx or 0x07Ex (canonical is
    // 0x3E8x). Source text starts with `bankSwitch`.
    if raw.len() >= 3
        && raw[0] == 0x19
        && text.trim_start().starts_with("bankSwitch")
    {
        let word = ((raw[1] as u16) << 8) | (raw[2] as u16);
        if (word & 0xFFF0) == 0x07D0 {
            return format!("{}\t;@enc=legacy_d", text);
        }
        if (word & 0xFFF0) == 0x07E0 {
            return format!("{}\t;@enc=legacy_e", text);
        }
    }

    // Pattern 3: setPalette palette-0 with trailing 0x00 (canonical
    // is 0xFF). Source text starts with `setPalette`. We rewrite
    // the source line to include an explicit `_trailing=0x00`
    // operand instead of emitting an annotation.
    if raw.len() >= 3
        && raw[0] == 0x0B
        && raw[2] == 0x00
        && text.trim_start().starts_with("setPalette")
    {
        return format!("{}, _trailing=0x00", text);
    }

    // Default: canonical encoding. The assembler reproduces the
    // bytes from `text` alone; no annotation needed.
    text.to_owned()
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
                "video offset={}, x={}, y={}",
                label, x, y
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
                // For full-form CINEMATIC default zoom, drop the
                // `zoom=0x40` keyword unless compact form (opcode
                // 0x80+) could have encoded the same x, y, offset.
                // In the could-be-compact case we keep `zoom=0x40`
                // explicit so the encoder round-trips to full form
                // rather than picking the cheaper compact form.
                let default_zoom = (opcode & 3) == 0;
                let x_byte_lit_le_ff = (opcode & 0x20 != 0) && (opcode & 0x10 == 0);
                let y_byte_lit_le_ff = (opcode & 0x08) != 0;
                let offset_fits = offset <= 0xFFFE;
                let could_be_compact = x_byte_lit_le_ff && y_byte_lit_le_ff && offset_fits;
                if default_zoom && !could_be_compact {
                    return format!(
                        "video offset={}, x={}, y={}",
                        label, x_str, y_str
                    );
                }
                return format!(
                    "video offset={}, x={}, y={}, zoom={}",
                    label, x_str, y_str, zoom_str
                );
            }
        }

        match opcode {
            0x00 => {
                let dst_var = self.variable_name(t.fetch());
                let imm_hi = t.fetch();
                let imm_lo = t.fetch();
                let imm = (u32::from(imm_hi) << 8) | u32::from(imm_lo);
                format!("mov [{}], 0x{:04X}", dst_var, imm)
            }
            0x01 => {
                let dst_var = self.variable_name(t.fetch());
                let src_var = self.variable_name(t.fetch());
                format!("mov [{}], [{}]", dst_var, src_var)
            }
            0x02 => {
                let dst_var = self.variable_name(t.fetch());
                let src_var = self.variable_name(t.fetch());
                format!("add [{}], [{}]", dst_var, src_var)
            }
            0x03 => {
                let dst_var = self.variable_name(t.fetch());
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
                    "setup channel={}, address={}",
                    self.channel(thread_id as u8),
                    self.label_name_register(t, pc_offset)
                )
            }
            0x09 => {
                let var = t.fetch();
                let hi = t.fetch();
                let lo = t.fetch();
                let offset = (u32::from(hi) << 8) | u32::from(lo);
                let var_name = self.variable_name(var);
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
                let var1 = self.variable_name(b);
                let midterm: String;
                if subop & 0x80 != 0 {
                    let var2 = self.variable_name(c);
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
                format!("selectVideoPage {}", self.page(id))
            }
            0x0e => {
                let page = t.fetch();
                let color = t.fetch();
                format!("fill page={}, color=0x{:02X}", self.page(page), color)
            }
            0x0f => {
                let src = t.fetch();
                let dst = t.fetch();
                format!("copyVideoPage src={}, dst={}", self.page(src), self.page(dst))
            }
            0x10 => {
                let page = t.fetch();
                format!("blitFramebuffer {}", self.page(page))
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
                let v1 = self.variable_name(t.fetch());
                let v2 = self.variable_name(t.fetch());
                format!("sub [{}], [{}]", v1, v2)
            }
            0x14 => {
                let dst = self.variable_name(t.fetch());
                let hi = t.fetch();
                let lo = t.fetch();
                let imm = (u32::from(hi) << 8) | u32::from(lo);
                format!("and [{}], 0x{:04X}", dst, imm)
            }
            0x15 => {
                let dst = self.variable_name(t.fetch());
                let hi = t.fetch();
                let lo = t.fetch();
                let imm = (u32::from(hi) << 8) | u32::from(lo);
                format!("or [{}], 0x{:04X}", dst, imm)
            }
            0x16 => {
                let var = self.variable_name(t.fetch());
                let hi = t.fetch();
                let lo = t.fetch();
                let imm = (u32::from(hi) << 8) | u32::from(lo);
                format!("shl [{}], 0x{:04X}", var, imm)
            }
            0x17 => {
                let var = self.variable_name(t.fetch());
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
                    "song id={}, delay=0x{:04X}, pos=0x{:02X}",
                    self.music(res as u16), delay, pos
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
    registry: &'static crate::registry::PortRegistry,
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
    if let Some(stage) = registry.stage(game_level) {
        for a in stage.possibly_unused_codeblocks {
            tracer.schedule_entry_point(*a, true);
        }
    }

    let mut dis =
        AwvmDisassembler::new(game_level, str_data, str_index, video2, release, registry);
    tracer.run(&mut dis, &[0x0000]);

    if let Some(parent) = out_asm_path.parent() {
        fs::create_dir_all(parent)?;
    }
    tracer.save_disassembly_listing(&dis, out_asm_path)?;

    // Report label-name collisions rather than emitting a duplicate label
    // and letting the assembler silently pick one (#0118). These are real
    // naming debt: a second instance of a canonical routine that needs its
    // own name in the port's local registry.
    let collisions = dis.label_name_collisions();
    if !collisions.is_empty() {
        eprintln!(
            "warning: level {}: {} label-name collision(s); the later address \
             falls back to a placeholder so the round-trip stays byte-identical. \
             Name these in the port's local registry (archaeology issue #0118):",
            game_level,
            collisions.len()
        );
        for (name, owner, rejected) in &collisions {
            eprintln!(
                "  {} is held by 0x{:04X}; 0x{:04X} emitted as a placeholder",
                name, owner, rejected
            );
        }
    }

    let cin = dis.cinematic_name_collisions();
    if !cin.is_empty() {
        eprintln!(
            "warning: level {}: {} cinematic-name collision(s); the later polygon \
             offset falls back to its numeric CINEMATIC_NNN. Two offsets sharing a \
             name is the #0107 misapplication showing up as a byte-level defect:",
            game_level,
            cin.len()
        );
        for (name, owner, rejected) in cin {
            eprintln!(
                "  {} is held by offset 0x{:04X}; 0x{:04X} fell back to numeric",
                name, owner, rejected
            );
        }
    }

    Ok(dis)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #0118: the first address to claim a name keeps it; a second address
    /// resolving to the same name is told to use a placeholder instead, which
    /// is what keeps the round-trip byte-identical.
    #[test]
    fn one_name_one_address_per_level() {
        let mut owners: HashMap<&'static str, u32> = HashMap::new();

        // msdos level 6: 0x76E9 and 0x84CC both chain-walk to amiga 0x5E84.
        assert_eq!(claim_name(&mut owners, "DRAW_ALIEN_POSE_0_0", 0x76E9), Claim::Held);
        assert_eq!(
            claim_name(&mut owners, "DRAW_ALIEN_POSE_0_0", 0x84CC),
            Claim::TakenBy(0x76E9)
        );
    }

    /// The owner may re-ask as often as it likes: the label declaration and
    /// every operand referencing it go through the same lookup, and they must
    /// agree or the listing would declare one name and reference another.
    #[test]
    fn claim_is_idempotent_for_the_owner() {
        let mut owners: HashMap<&'static str, u32> = HashMap::new();
        assert_eq!(claim_name(&mut owners, "KILL_CHANNEL_ROUTINE", 0x007A), Claim::Held);
        assert_eq!(claim_name(&mut owners, "KILL_CHANNEL_ROUTINE", 0x007A), Claim::Held);
        assert_eq!(
            claim_name(&mut owners, "KILL_CHANNEL_ROUTINE", 0x007E),
            Claim::TakenBy(0x007A)
        );
        // ...and the rejection does not transfer ownership.
        assert_eq!(claim_name(&mut owners, "KILL_CHANNEL_ROUTINE", 0x007A), Claim::Held);
    }

    /// Distinct names are independent.
    #[test]
    fn distinct_names_do_not_interfere() {
        let mut owners: HashMap<&'static str, u32> = HashMap::new();
        assert_eq!(claim_name(&mut owners, "A", 1), Claim::Held);
        assert_eq!(claim_name(&mut owners, "B", 2), Claim::Held);
        assert_eq!(claim_name(&mut owners, "A", 3), Claim::TakenBy(1));
        assert_eq!(claim_name(&mut owners, "B", 4), Claim::TakenBy(2));
    }
}
