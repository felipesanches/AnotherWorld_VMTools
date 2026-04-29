//! Another World VM toolchain — Rust port.
//!
//! This is a port of the Python `AnotherWorld_VMTools` codebase.
//! The port is being grown incrementally; modules land as the
//! corresponding Python files are ported and validated for
//! byte-identical output against the Python reference.
//!
//! Phase A landed: [`unpacker`], [`memlist`], [`bank`].

#![deny(unsafe_code)]

pub mod bank;
pub mod memlist;
pub mod unpacker;
