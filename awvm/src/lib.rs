//! Another World VM toolchain — Rust port.
//!
//! This is a port of the Python `AnotherWorld_VMTools` codebase.
//! The port is being grown incrementally; modules land as the
//! corresponding Python files are ported and validated for
//! byte-identical output against the Python reference.
//!
//! Phase A landed: [`unpacker`], [`memlist`], [`bank`].
//! Phase C landed: [`romset`], [`disasm`], plus the auto-generated
//! [`releases`] data tables.

#![deny(unsafe_code)]

pub mod bank;
pub mod disasm;
pub mod memlist;
pub mod releases;
pub mod romset;
pub mod unpacker;
