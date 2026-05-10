//! Build script: parse per-port TOML registry data files and
//! emit Rust source for each port's constants into `OUT_DIR`.
//!
//! Source-of-truth: `src/releases/data/<port>.toml`. The
//! corresponding `src/releases/<port>.rs` is a thin shim that
//! `include!`s the generated file from `OUT_DIR`.
//!
//! Phase 1 emits BOTH:
//!   - the new `REGISTRY: PortRegistry` constant (chain-aware
//!     lookup, see `src/registry.rs`)
//!   - the legacy per-port consts (`STAGE_TITLES`,
//!     `KNOWN_LABELS`, `STAGE_SPECIFIC_VARS`,
//!     `LABELED_CINEMATIC_ENTRIES_OVERRIDE`,
//!     `POSSIBLY_UNUSED_CODEBLOCKS`, `MD5_CHECKSUMS`, and the
//!     `resource_ids` submodule)
//! so the existing `releases::ReleaseData` API surface stays
//! intact and `disasm.rs` doesn't need to change in this phase.
//!
//! The script's parser is ad-hoc — TOML data files are
//! controlled inputs, not user-supplied content, so we don't
//! over-engineer error handling. Parse errors panic with the
//! offending file name.

use std::env;
use std::fs;
use std::path::PathBuf;

use toml::Value;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let data_dir = PathBuf::from(&manifest_dir).join("src/releases/data");
    let out_root = PathBuf::from(env::var("OUT_DIR").unwrap()).join("registries");
    fs::create_dir_all(&out_root).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/releases/data");

    let mut entries: Vec<_> = fs::read_dir(&data_dir).unwrap().collect();
    entries.sort_by_key(|e| e.as_ref().unwrap().file_name());
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let port = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let raw = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("read {}: {}", path.display(), e)
        });
        let parsed: Value = toml::from_str(&raw).unwrap_or_else(|e| {
            panic!("parse {}: {}", path.display(), e)
        });
        let rust = emit_port_rs(&port, &parsed);
        let out_path = out_root.join(format!("{port}.rs"));
        fs::write(&out_path, rust).unwrap();
    }
}

// --- emission helpers -----------------------------------------------

fn emit_port_rs(port: &str, root: &Value) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "// AUTO-GENERATED from src/releases/data/{port}.toml by build.rs.\n\
         // Edit the TOML, not this file.\n\n"
    ));

    // --- legacy constants (preserve the existing API surface) ---

    let md5 = root.get("md5_checksums").and_then(Value::as_array);
    s.push_str("pub const MD5_CHECKSUMS: &[(&str, &str)] = &[\n");
    if let Some(arr) = md5 {
        for entry in arr {
            let pair = entry.as_array().unwrap();
            let k = pair[0].as_str().unwrap();
            let v = pair[1].as_str().unwrap();
            s.push_str(&format!("    ({}, {}),\n", esc_str(k), esc_str(v)));
        }
    }
    s.push_str("];\n\n");

    s.push_str("pub const STAGE_TITLES: &[&str] = &[\n");
    if let Some(arr) = root.get("stage_titles").and_then(Value::as_array) {
        for v in arr {
            s.push_str(&format!("    {},\n", esc_str(v.as_str().unwrap())));
        }
    }
    s.push_str("];\n\n");

    // resource_ids submodule
    s.push_str("pub mod resource_ids {\n");
    let rids = root.get("resource_ids").and_then(Value::as_table);
    for (kind_lower, kind_upper) in [
        ("bytecode", "BYTECODE"),
        ("screen", "SCREEN"),
        ("music", "MUSIC"),
        ("sample", "SAMPLE"),
        ("palette", "PALETTE"),
        ("cinematic", "CINEMATIC"),
        ("video2", "VIDEO2"),
    ] {
        let ids: Vec<u8> = rids
            .and_then(|t| t.get(kind_lower))
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_integer().map(|i| i as u8)).collect())
            .unwrap_or_default();
        s.push_str(&format!(
            "    pub const {kind_upper}: &[u8] = &[{}];\n",
            ids.iter()
                .map(|b| format!("0x{:02X}", b))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    s.push_str("}\n\n");

    let stages = collect_stages(root);

    // KNOWN_LABELS
    s.push_str("pub const KNOWN_LABELS: &[(u32, &[(u32, &str)])] = &[\n");
    for stage in &stages {
        if let Some(labels) = &stage.labels {
            s.push_str(&format!("    ({}, &[\n", stage.idx));
            for (addr, name) in labels {
                s.push_str(&format!("        (0x{:04X}, {}),\n", addr, esc_str(name)));
            }
            s.push_str("    ]),\n");
        }
    }
    s.push_str("];\n\n");

    // POSSIBLY_UNUSED_CODEBLOCKS
    s.push_str("pub const POSSIBLY_UNUSED_CODEBLOCKS: &[(u32, &[u32])] = &[\n");
    for stage in &stages {
        if let Some(ids) = &stage.possibly_unused_codeblocks {
            let ids_str = ids
                .iter()
                .map(|i| format!("0x{:04X}", i))
                .collect::<Vec<_>>()
                .join(", ");
            s.push_str(&format!("    ({}, &[{ids_str}]),\n", stage.idx));
        }
    }
    s.push_str("];\n\n");

    // LABELED_CINEMATIC_ENTRIES_OVERRIDE
    s.push_str("pub const LABELED_CINEMATIC_ENTRIES_OVERRIDE: &[(u32, &[(u32, &str)])] = &[\n");
    for stage in &stages {
        if let Some(entries) = &stage.cinematic_entries {
            s.push_str(&format!("    ({}, &[\n", stage.idx));
            for (addr, name) in entries {
                s.push_str(&format!("        (0x{:04X}, {}),\n", addr, esc_str(name)));
            }
            s.push_str("    ]),\n");
        }
    }
    s.push_str("];\n\n");

    // STAGE_SPECIFIC_VARS
    s.push_str("pub const STAGE_SPECIFIC_VARS: &[(u32, &[(u8, &str)])] = &[\n");
    for stage in &stages {
        if let Some(aliases) = &stage.variable_aliases {
            s.push_str(&format!("    ({}, &[\n", stage.idx));
            for (var, name) in aliases {
                s.push_str(&format!("        (0x{:02X}, {}),\n", var, esc_str(name)));
            }
            s.push_str("    ]),\n");
        }
    }
    s.push_str("];\n\n");

    // RELEASE_DATA — same shape as before.
    let slug = root.get("slug").and_then(Value::as_str).unwrap_or(port);
    s.push_str(&format!(
        "pub const RELEASE_DATA: crate::releases::ReleaseData = crate::releases::ReleaseData {{\n\
         \x20   slug: {},\n\
         \x20   stage_titles: STAGE_TITLES,\n\
         \x20   known_labels: KNOWN_LABELS,\n\
         \x20   possibly_unused_codeblocks: POSSIBLY_UNUSED_CODEBLOCKS,\n\
         \x20   labeled_cinematic_entries: LABELED_CINEMATIC_ENTRIES_OVERRIDE,\n\
         \x20   stage_specific_vars: STAGE_SPECIFIC_VARS,\n\
         \x20   resource_ids: crate::romset::ResourceIds {{\n\
         \x20       bytecode: resource_ids::BYTECODE,\n\
         \x20       cinematic: resource_ids::CINEMATIC,\n\
         \x20       palette: resource_ids::PALETTE,\n\
         \x20       video2: resource_ids::VIDEO2,\n\
         \x20   }},\n\
         }};\n\n",
        esc_str(slug)
    ));

    // --- new REGISTRY (PortRegistry from src/registry.rs) -------------

    // Per-stage static arrays for the registry. We need stable
    // identifiers for each per-stage slice the registry borrows.
    // Emit them as `static <PORT>_STAGE_<idx>_LABELS: &[...]` etc.
    let port_upper = port.to_uppercase();
    for stage in &stages {
        if let Some(labels) = &stage.labels {
            s.push_str(&format!(
                "static {}_STAGE_{}_LABELS: &[(u32, &str)] = &[\n",
                port_upper, stage.idx
            ));
            for (addr, name) in labels {
                s.push_str(&format!("    (0x{:04X}, {}),\n", addr, esc_str(name)));
            }
            s.push_str("];\n");
        }
        if let Some(entries) = &stage.cinematic_entries {
            s.push_str(&format!(
                "static {}_STAGE_{}_CINEMATIC_ENTRIES: &[(u32, &str)] = &[\n",
                port_upper, stage.idx
            ));
            for (addr, name) in entries {
                s.push_str(&format!("    (0x{:04X}, {}),\n", addr, esc_str(name)));
            }
            s.push_str("];\n");
        }
        if let Some(aliases) = &stage.variable_aliases {
            s.push_str(&format!(
                "static {}_STAGE_{}_VAR_ALIASES: &[(u8, &str)] = &[\n",
                port_upper, stage.idx
            ));
            for (var, name) in aliases {
                s.push_str(&format!("    (0x{:02X}, {}),\n", var, esc_str(name)));
            }
            s.push_str("];\n");
        }
        if let Some(ids) = &stage.possibly_unused_codeblocks {
            let ids_str = ids
                .iter()
                .map(|i| format!("0x{:04X}", i))
                .collect::<Vec<_>>()
                .join(", ");
            s.push_str(&format!(
                "static {}_STAGE_{}_UNUSED_CODEBLOCKS: &[u32] = &[{ids_str}];\n",
                port_upper, stage.idx
            ));
        }
        if let Some(cohorts) = &stage.translation_to_parent {
            s.push_str(&format!(
                "static {}_STAGE_{}_COHORTS: &[crate::registry::TranslationCohort] = &[\n",
                port_upper, stage.idx
            ));
            for c in cohorts {
                let kind_expr = match c.kind.as_str() {
                    "shift" => format!("crate::registry::CohortKind::Shift({})", c.delta),
                    "no_parent" => "crate::registry::CohortKind::NoParent".to_owned(),
                    other => panic!("unknown cohort kind {other:?}"),
                };
                s.push_str(&format!(
                    "    crate::registry::TranslationCohort {{ start: 0x{:04X}, end: 0x{:04X}, kind: {} }},\n",
                    c.start, c.end, kind_expr,
                ));
            }
            s.push_str("];\n");
        }
    }
    s.push_str("\n");

    s.push_str(&format!(
        "static {port_upper}_STAGES: &[crate::registry::StageRegistry] = &[\n"
    ));
    for stage in &stages {
        let labels_ref = if stage.labels.is_some() {
            format!("{}_STAGE_{}_LABELS", port_upper, stage.idx)
        } else {
            "&[]".to_owned()
        };
        let cin_ref = if stage.cinematic_entries.is_some() {
            format!("{}_STAGE_{}_CINEMATIC_ENTRIES", port_upper, stage.idx)
        } else {
            "&[]".to_owned()
        };
        let aliases_ref = if stage.variable_aliases.is_some() {
            format!("{}_STAGE_{}_VAR_ALIASES", port_upper, stage.idx)
        } else {
            "&[]".to_owned()
        };
        let unused_ref = if stage.possibly_unused_codeblocks.is_some() {
            format!("{}_STAGE_{}_UNUSED_CODEBLOCKS", port_upper, stage.idx)
        } else {
            "&[]".to_owned()
        };
        let cohorts_ref = if stage.translation_to_parent.is_some() {
            format!("{}_STAGE_{}_COHORTS", port_upper, stage.idx)
        } else {
            "&[]".to_owned()
        };
        s.push_str(&format!(
            "    crate::registry::StageRegistry {{\n\
             \x20       stage_idx: {idx},\n\
             \x20       parent_stage_idx: None,\n\
             \x20       stage_title: None,\n\
             \x20       prologue: None,\n\
             \x20       labels: {labels_ref},\n\
             \x20       comments: &[],\n\
             \x20       cinematic_entries: {cin_ref},\n\
             \x20       possibly_unused_codeblocks: {unused_ref},\n\
             \x20       variable_aliases: {aliases_ref},\n\
             \x20       translation_to_parent: {cohorts_ref},\n\
             \x20   }},\n",
            idx = stage.idx,
        ));
    }
    s.push_str("];\n\n");

    let parent = root.get("parent").and_then(Value::as_str);
    let parent_expr = match parent {
        Some(p) => format!("Some(&crate::releases::{}::REGISTRY)", p),
        None => "None".to_owned(),
    };

    s.push_str(&format!(
        "pub static REGISTRY: crate::registry::PortRegistry = crate::registry::PortRegistry {{\n\
         \x20   port_id: {},\n\
         \x20   slug: {},\n\
         \x20   parent: {},\n\
         \x20   stage_titles: STAGE_TITLES,\n\
         \x20   stages: {port_upper}_STAGES,\n\
         \x20   md5_checksums: MD5_CHECKSUMS,\n\
         \x20   resource_ids_bytecode: resource_ids::BYTECODE,\n\
         \x20   resource_ids_screen: resource_ids::SCREEN,\n\
         \x20   resource_ids_music: resource_ids::MUSIC,\n\
         \x20   resource_ids_sample: resource_ids::SAMPLE,\n\
         \x20   resource_ids_palette: resource_ids::PALETTE,\n\
         \x20   resource_ids_cinematic: resource_ids::CINEMATIC,\n\
         \x20   resource_ids_video2: resource_ids::VIDEO2,\n\
         }};\n",
        esc_str(port),
        esc_str(slug),
        parent_expr,
    ));

    s
}

#[derive(Default)]
struct StageData {
    idx: u32,
    labels: Option<Vec<(u32, String)>>,
    cinematic_entries: Option<Vec<(u32, String)>>,
    possibly_unused_codeblocks: Option<Vec<u32>>,
    variable_aliases: Option<Vec<(u8, String)>>,
    /// Sibling-port translation cohorts (Phase 2). For canonical
    /// ports this is None / empty.
    translation_to_parent: Option<Vec<CohortData>>,
}

struct CohortData {
    start: u32,
    end: u32,
    kind: String, // "shift" or "no_parent"
    delta: i32, // 0 if no_parent
}

fn collect_stages(root: &Value) -> Vec<StageData> {
    let mut stages: Vec<StageData> = Vec::new();
    let arr = match root.get("stages").and_then(Value::as_array) {
        Some(a) => a,
        None => return stages,
    };
    for v in arr {
        let t = v.as_table().unwrap();
        let idx = t.get("idx").and_then(Value::as_integer).unwrap() as u32;
        let mut s = StageData {
            idx,
            ..Default::default()
        };
        if let Some(labels) = t.get("labels").and_then(Value::as_array) {
            s.labels = Some(labels.iter().map(parse_addr_name_pair).collect());
        }
        if let Some(entries) = t.get("cinematic_entries").and_then(Value::as_array) {
            s.cinematic_entries = Some(entries.iter().map(parse_addr_name_pair).collect());
        }
        if let Some(arr) = t.get("possibly_unused_codeblocks").and_then(Value::as_array) {
            s.possibly_unused_codeblocks = Some(
                arr.iter()
                    .map(|v| v.as_integer().unwrap() as u32)
                    .collect(),
            );
        }
        if let Some(aliases) = t.get("variable_aliases").and_then(Value::as_array) {
            s.variable_aliases = Some(
                aliases
                    .iter()
                    .map(|v| {
                        let a = v.as_array().unwrap();
                        let var = a[0].as_integer().unwrap() as u8;
                        let name = a[1].as_str().unwrap().to_owned();
                        (var, name)
                    })
                    .collect(),
            );
        }
        if let Some(cohorts) = t.get("translation_to_parent").and_then(Value::as_array) {
            s.translation_to_parent = Some(
                cohorts
                    .iter()
                    .map(|v| {
                        let table = v.as_table().unwrap();
                        let start = table.get("start").and_then(Value::as_integer).unwrap() as u32;
                        let end = table.get("end").and_then(Value::as_integer).unwrap() as u32;
                        let kind = table.get("kind").and_then(Value::as_str).unwrap().to_owned();
                        let delta = table
                            .get("delta")
                            .and_then(Value::as_integer)
                            .map(|i| i as i32)
                            .unwrap_or(0);
                        CohortData { start, end, kind, delta }
                    })
                    .collect(),
            );
        }
        stages.push(s);
    }
    // Sort by idx so emitted constants have deterministic order.
    stages.sort_by_key(|s| s.idx);
    stages
}

fn parse_addr_name_pair(v: &Value) -> (u32, String) {
    let a = v.as_array().unwrap();
    let addr = a[0].as_integer().unwrap() as u32;
    let name = a[1].as_str().unwrap().to_owned();
    (addr, name)
}

fn esc_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
