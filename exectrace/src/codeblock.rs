//! `CodeBlock` — a contiguous range of program memory discovered by
//! the trace.

use std::collections::BTreeMap;

/// A `CodeBlock` represents an inclusive `[start, end]` byte range in
/// program memory that the trace has determined to be code.
///
/// Mirrors `exectrace.CodeBlock` from the Python reference. The
/// `next_block` field holds the addresses that this block's terminating
/// instruction may transfer control to (empty for `ret`, one entry for
/// an unconditional jump, two for a conditional branch).
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// First byte address belonging to this block.
    pub start: u32,
    /// Last byte address belonging to this block (inclusive).
    pub end: u32,
    /// Successor addresses.  Empty if the block terminates in a return
    /// or an illegal opcode.
    pub next_block: Vec<u32>,
    /// Map of `instruction_address -> routine_address` for every
    /// subroutine call within the block.
    pub subroutines: BTreeMap<u32, u32>,
    /// Whether this block's first address must be emitted with a label
    /// prefix in the disassembly listing.
    pub needs_label: bool,
}

impl CodeBlock {
    /// Build a fresh code block.
    pub fn new(start: u32, end: u32, next_block: Vec<u32>, needs_label: bool) -> Self {
        Self {
            start,
            end,
            next_block,
            subroutines: BTreeMap::new(),
            needs_label,
        }
    }

    /// Record a subroutine call site within this block.
    pub fn add_subroutine_call(&mut self, instr_address: u32, routine_address: u32) {
        self.subroutines.insert(instr_address, routine_address);
    }
}
