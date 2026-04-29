//! Rust port of ExecTrace — a CPU-agnostic instruction-trace framework.
//!
//! ExecTrace crawls every reachable code-path in a binary by stepping
//! through one instruction at a time, asking a user-supplied
//! [`Disassembler`] to interpret each opcode and to declare branching
//! behaviour via callbacks ([`Tracer::subroutine`],
//! [`Tracer::conditional_branch`], etc.). The result is:
//!
//! 1. A list of contiguous [`CodeBlock`]s covering every reached byte.
//! 2. A per-address disassembly text map.
//! 3. A `.asm`-style listing emitted by [`Tracer::save_disassembly_listing`],
//!    interleaving label declarations, data regions, and disassembled
//!    instructions in address order.
//!
//! The Rust API mirrors the Python semantics — same crawl algorithm,
//! same listing-output format — while leaning on Rust types where
//! possible. The crate is `no_std`-safe in the future but currently
//! depends on `std` for its file-listing emitter.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod codeblock;
mod listing;
mod tracer;

pub use codeblock::CodeBlock;
pub use tracer::{Disassembler, RelocationBlock, Tracer, VarKind, Variable};

/// Format a 16-bit value the way the Python reference does
/// (`"0x%04X" % v`).
pub fn hex16(v: u32) -> String {
    format!("0x{:04X}", v)
}

/// Format an 8-bit value the way the Python reference does
/// (`"0x%02X" % v`).
pub fn hex8(v: u8) -> String {
    format!("0x{:02X}", v)
}

/// Logging verbosity.  Mirrors the Python reference's three integer
/// levels (`ERROR=0`, `VERBOSE=1`, `DEBUG=2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Critical messages only.
    Error = 0,
    /// Informative non-error messages.
    Verbose = 1,
    /// Developer-facing debug messages.
    Debug = 2,
}
