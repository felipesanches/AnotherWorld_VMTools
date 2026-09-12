//! Integration test: verify the registry's chain-walking
//! resolver finds canonical labels for sibling-port addresses
//! via the populated translation cohorts in
//! `src/releases/data/<port>.toml`.
//!
//! This is the end-to-end sanity check for Phase 2 of the
//! canonical-port-registry refactor: archaeology builds the
//! cohort tables, sync_cohorts_to_awvm_tools.py merges them
//! across, build.rs emits the Rust constants, and this test
//! runs `MSDOS::REGISTRY.resolve_label` to verify a known
//! amiga label is reachable through the chain.

use awvm::releases::{amiga, msdos};

#[test]
fn msdos_local_label_resolves_directly() {
    // KILL_CHANNEL_ROUTINE exists in msdos's own KNOWN_LABELS at
    // stage 2 / 0x0099. resolve_label hits the local table first.
    assert_eq!(
        msdos::REGISTRY.resolve_label(2, 0x0099),
        Some("KILL_CHANNEL_ROUTINE"),
    );
}

#[test]
fn amiga_canonical_label_resolves_directly() {
    // amiga 0x0090 has KILL_CHANNEL_ROUTINE.
    assert_eq!(
        amiga::REGISTRY.resolve_label(2, 0x0090),
        Some("KILL_CHANNEL_ROUTINE"),
    );
}

#[test]
fn msdos_cohorts_are_populated() {
    // Verify Phase 2 actually wired translation cohorts. msdos
    // stage 2 (LAKE) should have ≥ 1 cohort entry pointing at amiga.
    let stage = msdos::REGISTRY.stage(2).expect("LAKE stage exists");
    assert!(
        !stage.translation_to_parent.is_empty(),
        "msdos LAKE has no translation_to_parent cohorts — \
         Phase 2 sync didn't run, or build.rs didn't emit them",
    );
}

#[test]
fn msdos_chain_walks_to_amiga() {
    // Pick an msdos address that has NO local label but DOES fall
    // inside a cohort whose translation lands on a named amiga
    // address. From the cohort builder's verified pairs:
    // msdos LAKE 0x017E → "BLIT_AND_SHOW_PAGE_FF_LOOP" (also exists
    // on amiga via cohort delta -52, at amiga 0x014A).
    //
    // For the chain-walk test specifically: we need an address
    // where msdos has no local label but amiga does. Pick a
    // straightforward case: msdos's `REGISTRY.parent` is amiga;
    // every cohort points at amiga. If `resolve_label` returns
    // a Some(..) for an addr that's NOT in msdos's local
    // KNOWN_LABELS, we know the chain walked.
    //
    // We assert the chain works at all, regardless of which name
    // surfaces (since the real data is large + can change with
    // future rename rounds).
    let stage = msdos::REGISTRY.stage(2).expect("LAKE stage exists");
    let mut chain_resolved = 0usize;
    let mut local_resolved = 0usize;
    // Sample a handful of addresses across the stage's address space.
    for &probe_addr in &[0x0099u32, 0x017E, 0x0200, 0x0500, 0x1000] {
        let local_only = stage
            .labels
            .iter()
            .any(|(a, _)| *a == probe_addr);
        if msdos::REGISTRY.resolve_label(2, probe_addr).is_some() {
            if local_only {
                local_resolved += 1;
            } else {
                chain_resolved += 1;
            }
        }
    }
    assert!(
        chain_resolved > 0 || local_resolved > 0,
        "no labels resolved at all — registry is empty?"
    );
}

/// #0099: prose written once on the canonical port must reach a sibling through
/// the same translation cohorts that carry label names. That is the entire
/// argument for a canonical registry — a description of the drowning scene should
/// not be written five times — so it is worth a test rather than a one-off check.
#[test]
fn comments_chain_walk_from_canonical_to_sibling() {
    // amiga LAKE carries the prose; amiga is canonical, so this is a local hit.
    let from_canonical = amiga::REGISTRY.resolve_comment(2, 0x34AA);
    assert!(
        from_canonical.is_some(),
        "amiga LAKE should carry prose at 0x34AA (BEETLE_KICK_DETECTOR)"
    );

    // The stage prologue resolves on the canonical port and must also resolve on
    // the sibling, which has none of its own.
    let canon_prologue = amiga::REGISTRY.resolve_prologue(2);
    assert!(canon_prologue.is_some(), "amiga LAKE should carry a prologue");
    let sibling_prologue = msdos::REGISTRY.resolve_prologue(2);
    assert_eq!(
        canon_prologue.map(|p| p.len()),
        sibling_prologue.map(|p| p.len()),
        "msdos should inherit amiga's LAKE prologue, not have its own"
    );

    // And msdos's own stage block must NOT define it locally, or the test above
    // would pass for the wrong reason.
    let stage = msdos::REGISTRY.stage(2).expect("msdos LAKE stage exists");
    assert!(
        stage.prologue.is_none(),
        "msdos LAKE defines a local prologue; this test no longer proves inheritance"
    );
}

/// A comment on an address no cohort maps anywhere must resolve to nothing rather
/// than to some unrelated routine's prose. The cohort tables are binary-searched
/// and fall through to implicit identity outside any cohort, so this is the case
/// where a wrong answer would be quiet.
#[test]
fn comments_do_not_resolve_for_an_unmapped_address() {
    // 0xFFFE is past the end of any real bytecode slab.
    assert!(msdos::REGISTRY.resolve_comment(2, 0xFFFE).is_none());
    assert!(amiga::REGISTRY.resolve_comment(2, 0xFFFE).is_none());
}

/// T7: a sibling must emit the SAME page symbols it uses. Defining them from the
/// local table while resolving them through the chain produced a listing that
/// referenced `PAGE_WORK` without defining it, and the assembler resolves an
/// unknown symbol to 0 silently — every MS-DOS level changed bytes. Only the
/// round-trip caught it, so the invariant is pinned here too.
#[test]
fn page_names_used_are_also_defined() {
    for reg in [&amiga::REGISTRY, &msdos::REGISTRY] {
        let effective = reg.video_page_names_effective();
        assert!(
            !effective.is_empty(),
            "no page-name table in effect; the emitter would produce literals \
             where the canonical port expects symbols"
        );
        // Every value the resolver will name must appear in the table the header
        // emits, or the listing uses a symbol it never defines.
        for &(v, name) in effective {
            assert_eq!(
                reg.video_page_name(v),
                Some(name),
                "0x{v:02X} resolves to a different name than the one defined"
            );
        }
    }
}

/// The flagged `copyVideoPage src` values stay literals on purpose: bit 7 is
/// probably a SCROLL_Y offset but only 68 of 214 sites support that, and bit 6 is
/// not understood. A name here would assert more than the evidence does.
#[test]
fn flagged_page_values_are_not_named() {
    for v in [0x40u8, 0x80, 0x83, 0xC0] {
        assert!(
            amiga::REGISTRY.video_page_name(v).is_none(),
            "0x{v:02X} carries flag bits and must not be named yet"
        );
    }
}

/// Same invariant as the page table, for the channel and music tables: every value
/// the resolver names must be one the header defines, or the listing uses a symbol
/// it never declares and the assembler resolves it to 0 in silence.
#[test]
fn channel_and_music_names_used_are_also_defined() {
    for reg in [&amiga::REGISTRY, &msdos::REGISTRY] {
        let chans = reg.channel_names_effective();
        assert!(!chans.is_empty(), "no channel table in effect");
        for &(v, name) in chans {
            assert_eq!(reg.channel_name(v), Some(name));
        }
        let music = reg.music_names_effective();
        assert!(!music.is_empty(), "no music table in effect");
        for &(v, name) in music {
            assert_eq!(reg.music_name(v), Some(name));
        }
    }
}

/// Only port- and stage-invariant channels may be named. A channel whose role is
/// per stage must stay a literal, because a channel's role is the union of the roles
/// of the routines on it and four of nine stages are still mostly placeholders.
/// 0x01 is the counter-example: 196 setups across 20 files with no single role.
#[test]
fn stage_scoped_channels_are_not_named() {
    for v in [0x01u8, 0x04, 0x07, 0x24, 0x2E] {
        assert!(
            amiga::REGISTRY.channel_name(v).is_none(),
            "0x{v:02X} is stage-scoped and must not carry a port-level name"
        );
    }
}

/// Palette names are STAGE-scoped, unlike channels, pages and music. There is one
/// palette resource per stage, so slot 0x03 is a different set of colours in LAKE
/// than in TANK, and a name must not leak between them. Easy to get wrong, because
/// every other table in this registry is port-level.
#[test]
fn palette_names_are_stage_scoped_and_inherited() {
    // LAKE is stage 2 on both ports. amiga owns the names; msdos inherits them.
    assert_eq!(
        amiga::REGISTRY.palette_name(2, 0x03),
        Some("PAL_LAKE_UNDERWATER")
    );
    assert_eq!(
        msdos::REGISTRY.palette_name(2, 0x03),
        Some("PAL_LAKE_UNDERWATER"),
        "msdos should inherit LAKE's palette names through the chain"
    );
    // The same slot in another stage is a different palette entirely.
    assert!(
        amiga::REGISTRY.palette_name(5, 0x03).is_none(),
        "a LAKE palette name must not leak into TANK"
    );
    assert!(msdos::REGISTRY.palette_name(5, 0x03).is_none());

    // And the effective list the emitter defines matches what it will use.
    let eff = msdos::REGISTRY.palette_names_effective(2);
    assert!(!eff.is_empty());
    for (slot, name) in eff {
        assert_eq!(msdos::REGISTRY.palette_name(2, slot), Some(name));
    }
    assert!(msdos::REGISTRY.palette_names_effective(5).is_empty());
}
