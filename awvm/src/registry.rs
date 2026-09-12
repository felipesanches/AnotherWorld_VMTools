//! Cross-port symbol & comment registry types.
//!
//! Per the canonical-port-registry design (issue #0100 in the
//! archaeology repo), each port is a [`PortRegistry`] with an
//! optional `parent` link to its canonical port. Sibling ports
//! carry per-stage *translation cohorts* that map their byte
//! addresses to the parent's, so `resolve_label` can chain-walk
//! up to the canonical and find names defined once there.
//!
//! Phase 1 (this commit): types + chain-walking lookup are in
//! place; per-port registries are populated from
//! `src/releases/data/*.toml` by `build.rs`. Sibling ports'
//! `translation_to_parent` cohorts and the canonical port's
//! `comments`/`prologue` are empty in Phase 1 — Phase 2 +
//! Phase 3 land those.
//!
//! The companion design + integration proposal:
//!   `docs/symbol_migration/canonical_registry_design.md`
//!   `docs/symbol_migration/awvm_tools_integration_proposal.md`
//! both in the archaeology repo.

use std::cmp::Ordering;

/// Cap on chain depth (port → parent → grandparent). Beyond
/// this we treat the registry as malformed and bail out of the
/// lookup. Per #0100 Q3 (2-level by default; 3-level family
/// registries only on evidence).
const MAX_CHAIN_DEPTH: u32 = 3;

/// One port's complete registry.
///
/// `parent: None` marks the canonical port. Sibling ports point
/// up the chain to their canonical via `Some(&PARENT_REGISTRY)`.
pub struct PortRegistry {
    pub port_id: &'static str,
    pub slug: &'static str,
    pub parent: Option<&'static PortRegistry>,
    pub stage_titles: &'static [&'static str],
    pub stages: &'static [StageRegistry],
    pub md5_checksums: &'static [(&'static str, &'static str)],
    pub resource_ids_bytecode: &'static [u8],
    pub resource_ids_screen: &'static [u8],
    pub resource_ids_music: &'static [u8],
    pub resource_ids_sample: &'static [u8],
    pub resource_ids_palette: &'static [u8],
    pub resource_ids_cinematic: &'static [u8],
    pub resource_ids_video2: &'static [u8],
    /// Symbolic names for video-page operand values (plan item T7).
    ///
    /// Port-level rather than per-stage: a page id means the same thing in every
    /// stage and every port, unlike a palette slot. Only exact values are named —
    /// `copyVideoPage src` carries flag bits on top of a page number (0x80 is
    /// probably "offset by SCROLL_Y", 0x40 is not understood at all), and naming a
    /// composite would assert more than the evidence supports. See
    /// `docs/NAMING.md` §6.
    pub video_page_names: &'static [(u8, &'static str)],
    /// Symbolic names for `setup channel=` operands (plan item T7).
    ///
    /// Only channels whose role is the same in every stage and every port belong
    /// here. Most channels are stage-scoped — the bytecode reuses slots — and a
    /// channel's role is the union of the roles of the routines scheduled on it, so
    /// a stage's channels cannot be named before its routines are. See
    /// `docs/NAMING.md` §4.
    pub channel_names: &'static [(u8, &'static str)],
    /// Symbolic names for music resource ids used by `song id=` (plan item T7).
    pub music_names: &'static [(u16, &'static str)],
}

/// Per-stage data within a port. Each `pairs`-shaped table is
/// expected to be sorted-by-key for `binary_search` lookup.
pub struct StageRegistry {
    pub stage_idx: u32,
    /// Stage in the parent registry that this stage translates
    /// into. `None` means same index as `stage_idx`. Set when
    /// parent uses different stage numbering — e.g. a port that
    /// ships fewer stages and reorders the indices.
    pub parent_stage_idx: Option<u32>,
    /// Optional override of the port-level `stage_titles[stage_idx]`.
    /// `None` falls through to the port-level default; siblings
    /// also fall through to canonical via `resolve_stage_title`.
    pub stage_title: Option<&'static str>,
    /// Optional stage-prologue prose lines. Populated by Phase 3
    /// (#0099). Inheritance: sibling falls through to canonical.
    pub prologue: Option<&'static [&'static str]>,
    pub labels: &'static [(u32, &'static str)],
    /// Per-routine prose comments (Phase 3, #0099).
    pub comments: &'static [(u32, &'static [&'static str])],
    pub cinematic_entries: &'static [(u32, &'static str)],
    pub possibly_unused_codeblocks: &'static [u32],
    pub variable_aliases: &'static [(u8, &'static str)],
    /// Names for `setPalette` slots. **Stage-scoped**, unlike channels, pages and
    /// music: there is one palette resource per stage, so slot 0x03 is a different
    /// set of colours in LAKE and in TANK. See `docs/NAMING.md` §8.
    pub palette_names: &'static [(u8, &'static str)],
    /// Cohort table for translating sibling byte addresses to
    /// parent byte addresses. Phase 2 populates from the
    /// archaeology-side `data/port_registries/<port>.toml`.
    /// Empty on canonical and on Phase 1 sibling registries.
    pub translation_to_parent: &'static [TranslationCohort],
    /// Cohort table for translating sibling CINEMATIC (polygon-
    /// offset) addresses to parent cinematic addresses. The
    /// polygon address space is distinct from the bytecode
    /// address space, so this cannot share `translation_to_-
    /// parent` — applying the byte-address translation to
    /// polygon offsets gives wrong results. Empty on canonical.
    /// Populated for siblings via the archaeology-side
    /// `tools/build_polygon_translations.py` and deduped UP per
    /// archaeology issue #0105.
    pub cinematic_translation_to_parent: &'static [TranslationCohort],
}

pub struct TranslationCohort {
    pub start: u32, // inclusive
    pub end: u32,   // exclusive
    pub kind: CohortKind,
}

pub enum CohortKind {
    /// `parent_addr = this_addr + delta`. delta=0 is the
    /// implicit default outside any cohort; explicit delta=0
    /// cohorts are allowed but uncommon.
    Shift(i32),
    /// Port-exclusive content; lookup terminates with None.
    NoParent,
}

impl PortRegistry {
    pub fn stage(&'static self, stage_idx: u32) -> Option<&'static StageRegistry> {
        self.stages.iter().find(|s| s.stage_idx == stage_idx)
    }

    /// Walk the chain to resolve a label name for
    /// `(stage_idx, addr)`. Returns the first hit found going
    /// from this port up to canonical.
    pub fn resolve_label(
        &'static self,
        stage_idx: u32,
        addr: u32,
    ) -> Option<&'static str> {
        self.resolve_label_with_depth(stage_idx, addr, MAX_CHAIN_DEPTH)
    }

    fn resolve_label_with_depth(
        &'static self,
        stage_idx: u32,
        addr: u32,
        depth: u32,
    ) -> Option<&'static str> {
        if depth == 0 {
            // Chain too deep; design caps at 3 levels.
            return None;
        }
        let stage = self.stage(stage_idx)?;
        if let Some(name) = lookup_str_pair(stage.labels, addr) {
            return Some(name);
        }
        let parent = self.parent?;
        let parent_addr = translate_addr(stage.translation_to_parent, addr)?;
        let parent_stage = stage.parent_stage_idx.unwrap_or(stage_idx);
        parent.resolve_label_with_depth(parent_stage, parent_addr, depth - 1)
    }

    /// The page-name table in effect for this port: its own if it has one,
    /// otherwise the parent's.
    ///
    /// The emitter needs this, not the local field. Emitting symbolic operands by
    /// chain-walking while defining the EQUs from the local table produced a
    /// listing that used `PAGE_WORK` without defining it, and the assembler
    /// resolves an unknown symbol to 0 without complaining — so every MS-DOS level
    /// silently changed bytes. The round-trip gate caught it; nothing else would
    /// have.
    pub fn video_page_names_effective(&'static self) -> &'static [(u8, &'static str)] {
        if !self.video_page_names.is_empty() {
            return self.video_page_names;
        }
        match self.parent {
            Some(p) => p.video_page_names_effective(),
            None => &[],
        }
    }

    /// Name for a `setPalette` slot in this stage, chain-walking to the parent so a
    /// sibling inherits canonical's names for a stage they share.
    pub fn palette_name(
        &'static self,
        stage_idx: u32,
        slot: u8,
    ) -> Option<&'static str> {
        let stage = self.stage(stage_idx)?;
        if let Some(&(_, n)) = stage.palette_names.iter().find(|&&(s, _)| s == slot) {
            return Some(n);
        }
        let parent = self.parent?;
        parent.palette_name(stage.parent_stage_idx.unwrap_or(stage_idx), slot)
    }

    /// Every palette name in effect for a stage, so the emitter can define exactly
    /// the symbols it will use. Local names win over inherited ones.
    pub fn palette_names_effective(
        &'static self,
        stage_idx: u32,
    ) -> Vec<(u8, &'static str)> {
        let mut out: Vec<(u8, &'static str)> = Vec::new();
        let mut reg = Some(self);
        let mut idx = stage_idx;
        while let Some(r) = reg {
            if let Some(stage) = r.stage(idx) {
                for &(s, n) in stage.palette_names {
                    if !out.iter().any(|&(e, _)| e == s) {
                        out.push((s, n));
                    }
                }
                idx = stage.parent_stage_idx.unwrap_or(idx);
            }
            reg = r.parent;
        }
        out.sort_by_key(|&(s, _)| s);
        out
    }

    /// Channel table in effect: this port's if it has one, else the parent's.
    pub fn channel_names_effective(&'static self) -> &'static [(u8, &'static str)] {
        if !self.channel_names.is_empty() {
            return self.channel_names;
        }
        match self.parent {
            Some(p) => p.channel_names_effective(),
            None => &[],
        }
    }

    /// Music-name table in effect: this port's if it has one, else the parent's.
    pub fn music_names_effective(&'static self) -> &'static [(u16, &'static str)] {
        if !self.music_names.is_empty() {
            return self.music_names;
        }
        match self.parent {
            Some(p) => p.music_names_effective(),
            None => &[],
        }
    }

    /// Name for a `setup channel=` operand, or `None` to emit a literal.
    pub fn channel_name(&'static self, v: u8) -> Option<&'static str> {
        self.channel_names_effective()
            .iter()
            .find(|&&(a, _)| a == v)
            .map(|&(_, n)| n)
    }

    /// Name for a music resource id, or `None` to emit a literal.
    pub fn music_name(&'static self, v: u16) -> Option<&'static str> {
        self.music_names_effective()
            .iter()
            .find(|&&(a, _)| a == v)
            .map(|&(_, n)| n)
    }

    /// Name for a video-page operand value, or `None` to emit a literal.
    /// Falls through to the parent so a sibling inherits canonical's table.
    pub fn video_page_name(&'static self, v: u8) -> Option<&'static str> {
        if let Some(&(_, n)) = self.video_page_names.iter().find(|&&(a, _)| a == v) {
            return Some(n);
        }
        self.parent?.video_page_name(v)
    }

    /// Walk the chain to resolve a cinematic-entry label
    /// (originally `LABELED_CINEMATIC_ENTRIES_OVERRIDE` in the
    /// per-port Rust files).
    pub fn resolve_cinematic_entry(
        &'static self,
        stage_idx: u32,
        addr: u32,
    ) -> Option<&'static str> {
        self.resolve_cinematic_with_depth(stage_idx, addr, MAX_CHAIN_DEPTH)
    }

    fn resolve_cinematic_with_depth(
        &'static self,
        stage_idx: u32,
        addr: u32,
        depth: u32,
    ) -> Option<&'static str> {
        if depth == 0 {
            return None;
        }
        let stage = self.stage(stage_idx)?;
        if let Some(name) = lookup_str_pair(stage.cinematic_entries, addr) {
            return Some(name);
        }
        let parent = self.parent?;
        // Use the polygon-address translation cohorts, NOT the
        // byte-address ones — cinematic offsets live in a
        // different address space.
        let parent_addr =
            translate_addr(stage.cinematic_translation_to_parent, addr)?;
        let parent_stage = stage.parent_stage_idx.unwrap_or(stage_idx);
        parent.resolve_cinematic_with_depth(parent_stage, parent_addr, depth - 1)
    }

    /// Walk the chain to resolve per-routine prose comments
    /// (#0099, Phase 3).
    pub fn resolve_comment(
        &'static self,
        stage_idx: u32,
        addr: u32,
    ) -> Option<&'static [&'static str]> {
        self.resolve_comment_with_depth(stage_idx, addr, MAX_CHAIN_DEPTH)
    }

    fn resolve_comment_with_depth(
        &'static self,
        stage_idx: u32,
        addr: u32,
        depth: u32,
    ) -> Option<&'static [&'static str]> {
        if depth == 0 {
            return None;
        }
        let stage = self.stage(stage_idx)?;
        for &(a, lines) in stage.comments {
            if a == addr {
                return Some(lines);
            }
        }
        let parent = self.parent?;
        let parent_addr = translate_addr(stage.translation_to_parent, addr)?;
        let parent_stage = stage.parent_stage_idx.unwrap_or(stage_idx);
        parent.resolve_comment_with_depth(parent_stage, parent_addr, depth - 1)
    }

    /// First non-None prologue walking up the chain.
    pub fn resolve_prologue(
        &'static self,
        stage_idx: u32,
    ) -> Option<&'static [&'static str]> {
        self.resolve_prologue_with_depth(stage_idx, MAX_CHAIN_DEPTH)
    }

    fn resolve_prologue_with_depth(
        &'static self,
        stage_idx: u32,
        depth: u32,
    ) -> Option<&'static [&'static str]> {
        if depth == 0 {
            return None;
        }
        if let Some(stage) = self.stage(stage_idx) {
            if let Some(p) = stage.prologue {
                return Some(p);
            }
        }
        let parent = self.parent?;
        let parent_stage = self.stage(stage_idx)
            .and_then(|s| s.parent_stage_idx)
            .unwrap_or(stage_idx);
        parent.resolve_prologue_with_depth(parent_stage, depth - 1)
    }

    /// Stage title resolution: per-stage override → port-level
    /// `stage_titles[stage_idx]` → walk to parent.
    pub fn resolve_stage_title(
        &'static self,
        stage_idx: u32,
    ) -> Option<&'static str> {
        if let Some(stage) = self.stage(stage_idx) {
            if let Some(t) = stage.stage_title {
                return Some(t);
            }
        }
        if let Some(t) = self.stage_titles.get(stage_idx as usize) {
            return Some(t);
        }
        let parent = self.parent?;
        let parent_stage = self.stage(stage_idx)
            .and_then(|s| s.parent_stage_idx)
            .unwrap_or(stage_idx);
        parent.resolve_stage_title(parent_stage)
    }
}

/// Binary-search a sorted `(addr, name)` pair list.
fn lookup_str_pair(
    pairs: &'static [(u32, &'static str)],
    addr: u32,
) -> Option<&'static str> {
    pairs
        .binary_search_by_key(&addr, |&(a, _)| a)
        .ok()
        .map(|idx| pairs[idx].1)
}

/// Translate a child-port byte address to its parent-port
/// equivalent via the cohort table.
///
/// Returns `Some(addr)` for implicit identity (no cohort
/// matched), `Some(addr + delta)` for a Shift cohort, or `None`
/// for a NoParent cohort.
pub fn translate_addr(
    cohorts: &[TranslationCohort],
    addr: u32,
) -> Option<u32> {
    let result = cohorts.binary_search_by(|c| {
        if addr < c.start {
            Ordering::Greater
        } else if addr >= c.end {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    });
    match result {
        Ok(idx) => match cohorts[idx].kind {
            CohortKind::Shift(d) => Some((addr as i64 + d as i64) as u32),
            CohortKind::NoParent => None,
        },
        Err(_) => Some(addr), // implicit identity outside any cohort
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static EMPTY_LABELS: &[(u32, &str)] = &[];
    static EMPTY_CINEMATIC: &[(u32, &str)] = &[];
    static EMPTY_CODEBLOCKS: &[u32] = &[];
    static EMPTY_VAR_ALIASES: &[(u8, &str)] = &[];
    static EMPTY_COMMENTS: &[(u32, &[&str])] = &[];
    static EMPTY_COHORTS: &[TranslationCohort] = &[];

    #[test]
    fn translate_addr_identity_outside_cohort() {
        let cohorts: &[TranslationCohort] = &[];
        assert_eq!(translate_addr(cohorts, 0x1234), Some(0x1234));
    }

    #[test]
    fn translate_addr_shift() {
        let cohorts: &[TranslationCohort] = &[
            TranslationCohort {
                start: 0x100,
                end: 0x200,
                kind: CohortKind::Shift(-7),
            },
        ];
        assert_eq!(translate_addr(cohorts, 0x150), Some(0x149));
        assert_eq!(translate_addr(cohorts, 0x99), Some(0x99));
        assert_eq!(translate_addr(cohorts, 0x250), Some(0x250));
    }

    #[test]
    fn translate_addr_no_parent() {
        let cohorts: &[TranslationCohort] = &[
            TranslationCohort {
                start: 0x9C00,
                end: 0xA000,
                kind: CohortKind::NoParent,
            },
        ];
        assert_eq!(translate_addr(cohorts, 0x9D00), None);
        assert_eq!(translate_addr(cohorts, 0x8FFF), Some(0x8FFF));
        assert_eq!(translate_addr(cohorts, 0xA000), Some(0xA000));
    }

    static AMIGA_STAGE_2_LABELS: &[(u32, &str)] = &[
        (0x0090, "KILL_CHANNEL_ROUTINE"),
        (0x0200, "SOME_VIDEO_BUFFER_MAGIC"),
    ];

    static AMIGA_STAGES: &[StageRegistry] = &[
        StageRegistry {
            stage_idx: 2,
            parent_stage_idx: None,
            stage_title: None,
            prologue: None,
            labels: AMIGA_STAGE_2_LABELS,
            comments: EMPTY_COMMENTS,
            cinematic_entries: EMPTY_CINEMATIC,
            possibly_unused_codeblocks: EMPTY_CODEBLOCKS,
            variable_aliases: EMPTY_VAR_ALIASES,
            palette_names: &[],
            translation_to_parent: EMPTY_COHORTS,
            cinematic_translation_to_parent: EMPTY_COHORTS,
        },
    ];

    static AMIGA: PortRegistry = PortRegistry {
        port_id: "amiga",
        slug: "amiga",
        parent: None,
        stage_titles: &["", "", "Lake"],
        stages: AMIGA_STAGES,
        md5_checksums: &[],
        resource_ids_bytecode: &[],
        resource_ids_screen: &[],
        video_page_names: &[],
        channel_names: &[],
        music_names: &[],
        resource_ids_music: &[],
        resource_ids_sample: &[],
        resource_ids_palette: &[],
        resource_ids_cinematic: &[],
        resource_ids_video2: &[],
    };

    static MSDOS_STAGE_2_COHORTS: &[TranslationCohort] = &[
        TranslationCohort {
            start: 0x0099,
            end: 0x0100,
            kind: CohortKind::Shift(-9),
        },
    ];

    static MSDOS_STAGES: &[StageRegistry] = &[
        StageRegistry {
            stage_idx: 2,
            parent_stage_idx: None,
            stage_title: None,
            prologue: None,
            labels: EMPTY_LABELS,
            comments: EMPTY_COMMENTS,
            cinematic_entries: EMPTY_CINEMATIC,
            possibly_unused_codeblocks: EMPTY_CODEBLOCKS,
            variable_aliases: EMPTY_VAR_ALIASES,
            palette_names: &[],
            translation_to_parent: MSDOS_STAGE_2_COHORTS,
            cinematic_translation_to_parent: EMPTY_COHORTS,
        },
    ];

    static MSDOS: PortRegistry = PortRegistry {
        port_id: "msdos",
        slug: "msdos",
        parent: Some(&AMIGA),
        stage_titles: &[],
        stages: MSDOS_STAGES,
        md5_checksums: &[],
        resource_ids_bytecode: &[],
        resource_ids_screen: &[],
        video_page_names: &[],
        channel_names: &[],
        music_names: &[],
        resource_ids_music: &[],
        resource_ids_sample: &[],
        resource_ids_palette: &[],
        resource_ids_cinematic: &[],
        resource_ids_video2: &[],
    };

    #[test]
    fn resolve_label_via_chain() {
        // msdos 0x0099 → translates to amiga 0x0090 → "KILL_CHANNEL_ROUTINE"
        assert_eq!(
            MSDOS.resolve_label(2, 0x0099),
            Some("KILL_CHANNEL_ROUTINE"),
        );
    }

    #[test]
    fn resolve_label_implicit_identity() {
        // msdos 0x0200 is outside the cohort range (0x0099..0x0100)
        // → implicit identity → amiga 0x0200 → "SOME_VIDEO_BUFFER_MAGIC"
        assert_eq!(
            MSDOS.resolve_label(2, 0x0200),
            Some("SOME_VIDEO_BUFFER_MAGIC"),
        );
    }

    #[test]
    fn resolve_label_canonical_direct() {
        assert_eq!(
            AMIGA.resolve_label(2, 0x0090),
            Some("KILL_CHANNEL_ROUTINE"),
        );
    }

    #[test]
    fn resolve_label_unknown() {
        assert_eq!(MSDOS.resolve_label(2, 0xFFFF), None);
    }
}
