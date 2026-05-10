//! Thin shim: source-of-truth is `src/releases/data/symbian_demo.toml`.
//! `build.rs` regenerates the constants in this module into
//! `$OUT_DIR/registries/symbian_demo.rs`, included below.
//!
//! Phase 1 of the canonical-port-registry refactor: this file
//! used to hold ~500 lines of hand-maintained Rust constants;
//! they now live in TOML and are regenerated at build time. The
//! exported API (`KNOWN_LABELS`, `STAGE_TITLES`, `RELEASE_DATA`,
//! `resource_ids` submodule, etc.) is unchanged so existing
//! consumers (`disasm.rs`, `releases::by_slug`) compile without
//! modification.

include!(concat!(env!("OUT_DIR"), "/registries/symbian_demo.rs"));
