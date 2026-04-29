//! The trace state machine.
//!
//! Public surface:
//! - [`Tracer`]: holds the ROM, the discovered code blocks, the
//!   per-address disassembly text, and the worklist of pending entry
//!   points.
//! - [`Disassembler`]: trait the user implements to interpret bytes
//!   into instruction strings and to declare branching behaviour. The
//!   user calls back into the [`Tracer`] (via the `&mut Tracer` arg
//!   passed to [`Disassembler::disasm_instruction`]) to drive the
//!   crawl.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::codeblock::CodeBlock;
use crate::LogLevel;

/// One contiguous slice of a (possibly relocated) ROM.
///
/// Matches Python's `(reloc_from, reloc_to, length)` 3-tuple. Multiple
/// relocation blocks are supported: the same logical address may be
/// served by different physical ROM regions on different platforms.
#[derive(Debug, Clone, Copy)]
pub struct RelocationBlock {
    /// Byte offset within the source file where the slice starts.
    pub from_offset: u64,
    /// Logical address the slice should be mounted at.
    pub to_address: u32,
    /// Length in bytes.
    pub length: u32,
}

/// What flavour of "variable" a labeled address represents in the
/// final disassembly. Matches the Python tuple's second element.
#[derive(Debug, Clone)]
pub enum VarKind {
    /// Plain labeled address.
    Var,
    /// Plain label (no associated data).
    Label,
    /// A fixed-length string of `len` ASCII bytes.
    Str(u32),
    /// A length-prefixed string (`n-1_str`): one length byte followed
    /// by `n-1` ASCII bytes.
    NMinus1Str,
    /// A jump table of `count` 16-bit little-endian pointers; each
    /// entry is also scheduled as a code entry-point.
    JumpTable(u32),
    /// A pointer table of `count` 16-bit little-endian pointers; each
    /// entry is registered as a label but **not** scheduled.
    Pointers(u32),
}

/// Metadata for one variable / labeled address.
#[derive(Debug, Clone)]
pub struct Variable {
    /// Symbol name as it should appear in the disassembly.
    pub name: String,
    /// What it is.
    pub kind: VarKind,
}

/// User-supplied disassembler.
///
/// Implementors interpret bytes into instruction strings and declare
/// branching behaviour by calling back into the [`Tracer`] (passed in
/// as `&mut Tracer`).  Inside [`Self::disasm_instruction`] the
/// implementation typically:
///
/// 1. Calls [`Tracer::fetch`] to consume operand bytes.
/// 2. Calls one of [`Tracer::subroutine`],
///    [`Tracer::return_from_subroutine`],
///    [`Tracer::conditional_branch`], [`Tracer::unconditional_jump`],
///    or [`Tracer::illegal_instruction`] to declare what kind of
///    instruction this is.
/// 3. Returns a textual representation of the instruction.
pub trait Disassembler {
    /// Interpret a single instruction starting at `tracer.pc - 1`
    /// (the opcode has already been fetched).  Use `tracer.fetch()`
    /// to read further operand bytes.
    fn disasm_instruction(&mut self, tracer: &mut Tracer, opcode: u8) -> String;

    /// Header lines emitted at the top of the listing
    /// (e.g. `org`-related preamble, copyright comments).  Default
    /// is empty.
    fn output_disasm_headers(&self) -> String {
        String::new()
    }

    /// Override to substitute the canonical name for a variable.
    /// Default returns `None`, in which case the listing falls back
    /// to the [`Variable::name`] from `tracer.variables` and finally
    /// to a hex literal.
    fn variable_name(&self, _tracer: &Tracer, _addr: u32) -> Option<String> {
        None
    }

    /// Override to substitute the canonical name for a label.
    /// Default returns `None`, in which case the listing falls back
    /// to the [`Variable::name`] (if present), then [`Tracer::labels`]
    /// (if present), then `<prefix><uppercase-hex>`.
    fn label_name(&self, _tracer: &Tracer, _addr: u32) -> Option<String> {
        None
    }
}

/// The crawl state.
#[derive(Debug)]
pub struct Tracer {
    /// One byte buffer per relocation block.
    pub(crate) rom: Vec<Vec<u8>>,
    /// Mount points for each ROM slice.
    pub(crate) relocation_blocks: Vec<RelocationBlock>,
    /// Variables declared up front, keyed by address.
    pub variables: BTreeMap<u32, Variable>,
    /// Labels declared up front, keyed by address.
    pub labels: BTreeMap<u32, String>,

    /// All code blocks discovered so far.
    pub(crate) visited_ranges: Vec<CodeBlock>,
    /// Worklist: `(address, needs_label)`.
    pending_entry_points: Vec<(u32, bool)>,
    pub(crate) current_entry_point: u32,
    pub(crate) current_entry_point_needs_label: bool,
    pub(crate) pc: Option<u32>,
    /// Per-address disassembled instruction text.
    pub disasm: BTreeMap<u32, String>,
    /// Set of addresses needing label emission.
    pub(crate) labeled_addresses: BTreeSet<u32>,

    /// Logging verbosity (matches Python's `loglevel`).
    pub log_level: LogLevel,

    /// Internal flag: when an [`AddressAlreadyVisited`] flow occurs
    /// inside `fetch`, we surface it via this field rather than
    /// Python-style exceptions.
    fetch_already_visited: bool,
}

impl Tracer {
    /// Build a tracer from a single contiguous ROM (no relocation).
    pub fn from_rom(rom: Vec<u8>) -> Self {
        let len = rom.len() as u32;
        Self {
            rom: vec![rom],
            relocation_blocks: vec![RelocationBlock {
                from_offset: 0,
                to_address: 0,
                length: len,
            }],
            variables: BTreeMap::new(),
            labels: BTreeMap::new(),
            visited_ranges: Vec::new(),
            pending_entry_points: Vec::new(),
            current_entry_point: 0,
            current_entry_point_needs_label: false,
            pc: None,
            disasm: BTreeMap::new(),
            labeled_addresses: BTreeSet::new(),
            log_level: LogLevel::Error,
            fetch_already_visited: false,
        }
    }

    /// Build a tracer from a file with explicit relocation blocks.
    pub fn from_file_with_relocation(
        path: &Path,
        relocation_blocks: Vec<RelocationBlock>,
    ) -> std::io::Result<Self> {
        use std::io::{Read, Seek, SeekFrom};
        let mut f = std::fs::File::open(path)?;
        let mut rom = Vec::with_capacity(relocation_blocks.len());
        for r in &relocation_blocks {
            f.seek(SeekFrom::Start(r.from_offset))?;
            let mut buf = vec![0u8; r.length as usize];
            f.read_exact(&mut buf)?;
            rom.push(buf);
        }
        Ok(Self {
            rom,
            relocation_blocks,
            variables: BTreeMap::new(),
            labels: BTreeMap::new(),
            visited_ranges: Vec::new(),
            pending_entry_points: Vec::new(),
            current_entry_point: 0,
            current_entry_point_needs_label: false,
            pc: None,
            disasm: BTreeMap::new(),
            labeled_addresses: BTreeSet::new(),
            log_level: LogLevel::Error,
            fetch_already_visited: false,
        })
    }

    // ---------- Read primitives ----------

    /// Read one byte at the given logical address.
    pub fn read_byte(&self, addr: u32) -> u8 {
        let (i, off) = self
            .rom_address(addr)
            .expect("address outside any relocation block");
        self.rom[i][off]
    }

    /// Read one little-endian word at the given logical address.
    pub fn read_word(&self, addr: u32) -> u16 {
        u16::from(self.read_byte(addr)) | (u16::from(self.read_byte(addr + 1)) << 8)
    }

    /// Map a logical address to (relocation_index, byte_offset).
    pub fn rom_address(&self, logical_address: u32) -> Option<(usize, usize)> {
        for (i, r) in self.relocation_blocks.iter().enumerate() {
            if logical_address >= r.to_address && logical_address < r.to_address + r.length {
                return Some((i, (logical_address - r.to_address) as usize));
            }
        }
        None
    }

    /// Current PC (None if the crawl has not yet started or is paused).
    pub fn pc(&self) -> Option<u32> {
        self.pc
    }

    /// First address of the entry-point currently being walked.
    pub fn current_entry_point(&self) -> u32 {
        self.current_entry_point
    }

    /// Register an address as needing label emission.
    pub fn register_label(&mut self, address: u32) {
        self.labeled_addresses.insert(address);
    }

    // ---------- Worklist primitives ----------

    /// Schedule a future entry point. No-op if the address has already
    /// been visited or is already on the worklist.
    pub fn schedule_entry_point(&mut self, address: u32, needs_label: bool) {
        if self.already_visited(address) {
            return;
        }
        for (ep, _) in &self.pending_entry_points {
            if *ep == address {
                return;
            }
        }
        self.pending_entry_points.push((address, needs_label));
        self.log(LogLevel::Verbose, format!("SCHEDULING: {:#x}", address));
    }

    fn already_visited(&mut self, address: u32) -> bool {
        if let Some(pc) = self.pc {
            if address >= self.current_entry_point && address < pc {
                self.log(
                    LogLevel::Debug,
                    format!("RECENTLY: (PC={:#x} address={:#x})", pc, address),
                );
                return true;
            }
        }

        // Linear scan; matches Python.
        let mut split: Option<(usize, u32)> = None;
        for (i, cb) in self.visited_ranges.iter().enumerate() {
            if address >= cb.start && address <= cb.end {
                self.log(
                    LogLevel::Debug,
                    format!("ALREADY VISITED: {:#x}", address),
                );
                if address > cb.start {
                    split = Some((i, address));
                }
                if split.is_none() {
                    return true;
                }
                break;
            }
        }

        if let Some((i, address)) = split {
            // Split block at `address`.
            let (start, needs_label, mut sub_carry_old): (u32, bool, BTreeMap<u32, u32>) = {
                let cb = &self.visited_ranges[i];
                (cb.start, cb.needs_label, cb.subroutines.clone())
            };
            let mut new_subs = BTreeMap::new();
            sub_carry_old.retain(|&instr_addr, &mut call_addr| {
                if instr_addr < address {
                    new_subs.insert(instr_addr, call_addr);
                    false
                } else {
                    true
                }
            });

            let new_block = CodeBlock {
                start,
                end: address - 1,
                next_block: vec![address],
                subroutines: new_subs,
                needs_label,
            };

            {
                let cb = &mut self.visited_ranges[i];
                cb.start = address;
                cb.needs_label = true;
                cb.subroutines = sub_carry_old;
            }
            self.visited_ranges.push(new_block);
            return true;
        }

        false
    }

    fn restart_from_another_entry_point(&mut self) {
        if let Some((address, needs_label)) = self.pending_entry_points.pop() {
            self.current_entry_point = address;
            self.current_entry_point_needs_label = needs_label;
            self.pc = Some(address);
            self.log(LogLevel::Verbose, format!("Restarting from: {:#x}", address));
        } else {
            self.pc = None;
        }
    }

    fn add_range(&mut self, start: u32, end: u32, exit: Vec<u32>, needs_label: bool) {
        let (s, e) = if end < start { (end, start) } else { (start, end) };
        self.log(
            LogLevel::Debug,
            format!(
                "=== New Range: start: {:#x}  end: {:#x} needs_label: {}===",
                s, e, needs_label
            ),
        );
        self.visited_ranges
            .push(CodeBlock::new(s, e, exit, needs_label));
    }

    // ---------- Branching declarations (called from disasm_instruction) ----------

    /// Declare the current instruction as a `call` to `address`.
    pub fn subroutine(&mut self, address: u32) {
        let pc = self.pc.expect("subroutine() called outside an active trace");
        let cur = self.current_entry_point;
        let nl = self.current_entry_point_needs_label;
        self.add_range(cur, pc.saturating_sub(1), vec![pc, address], nl);
        self.schedule_entry_point(pc, false);
        self.schedule_entry_point(address, true);
        self.log(LogLevel::Verbose, format!("CALL SUBROUTINE ({:#x})", address));
        self.restart_from_another_entry_point();
    }

    /// Declare the current instruction as a `ret`.
    pub fn return_from_subroutine(&mut self) {
        let pc = self
            .pc
            .expect("return_from_subroutine() called outside an active trace");
        let cur = self.current_entry_point;
        let nl = self.current_entry_point_needs_label;
        self.add_range(cur, pc.saturating_sub(1), Vec::new(), nl);
        self.log(LogLevel::Verbose, "RETURN FROM SUBROUTINE".to_owned());
        self.restart_from_another_entry_point();
    }

    /// Declare the current instruction as a conditional branch.
    pub fn conditional_branch(&mut self, address: u32) {
        self.log(
            LogLevel::Verbose,
            format!("CONDITIONAL BRANCH to {:#x}", address),
        );
        self.branch(address, true);
    }

    /// Declare the current instruction as an unconditional jump.
    pub fn unconditional_jump(&mut self, address: u32) {
        self.log(
            LogLevel::Verbose,
            format!("UNCONDITIONAL JUMP to {:#x}", address),
        );
        self.branch(address, false);
    }

    fn branch(&mut self, address: u32, conditional: bool) {
        let pc = self.pc.expect("branch() called outside an active trace");
        let cur = self.current_entry_point;
        let nl = self.current_entry_point_needs_label;
        if address > cur && address < pc {
            self.add_range(cur, address.saturating_sub(1), vec![address], nl);
            self.add_range(address, pc.saturating_sub(1), vec![pc, address], true);
            if conditional {
                self.schedule_entry_point(pc, false);
            }
        } else {
            self.add_range(cur, pc.saturating_sub(1), vec![pc, address], nl);
            if conditional {
                self.schedule_entry_point(pc, false);
            }
            self.schedule_entry_point(address, true);
        }
        self.restart_from_another_entry_point();
    }

    /// Declare an unparseable opcode.  Crawl ends.
    pub fn illegal_instruction(&mut self, opcode: u8) {
        let pc = self
            .pc
            .expect("illegal_instruction() called outside an active trace");
        let cur = self.current_entry_point;
        let nl = self.current_entry_point_needs_label;
        self.add_range(
            cur,
            pc.saturating_sub(1),
            Vec::new(),
            nl,
        );
        self.log(
            LogLevel::Error,
            format!(
                "[{:#x}] ILLEGAL: {:#x}",
                pc.saturating_sub(1),
                opcode
            ),
        );
        let _ = pc; // already consumed
        self.pc = None;
    }

    // ---------- Fetch ----------

    /// Read the byte at the current PC and advance.
    ///
    /// If the address has already been visited, sets an internal flag
    /// that the run-loop will detect to drive the
    /// already-visited-restart codepath, and returns 0.
    pub fn fetch(&mut self) -> u8 {
        let pc = self.pc.expect("fetch() called outside an active trace");
        if self.already_visited(pc) {
            self.fetch_already_visited = true;
            return 0;
        }
        let value = match self.rom_address(pc) {
            Some((i, off)) => self.rom[i][off],
            None => {
                // Match Python: try restart_from_another_entry_point and re-fetch.
                self.restart_from_another_entry_point();
                if self.pc.is_none() {
                    self.fetch_already_visited = true;
                    return 0;
                }
                return self.fetch();
            }
        };
        self.log(
            LogLevel::Debug,
            format!("Fetch at {:#x}: {:#x}", pc, value),
        );
        self.pc = Some(pc + 1);
        value
    }

    // ---------- Public crawl entry point ----------

    /// Walk every reachable code path starting from `entry_points`.
    pub fn run(&mut self, dis: &mut dyn Disassembler, entry_points: &[u32]) {
        for ep in entry_points {
            self.schedule_entry_point(*ep, true);
        }
        self.restart_from_another_entry_point();
        if let Some(addr) = self.pc {
            self.register_label(addr);
        }

        while let Some(pc) = self.pc {
            self.fetch_already_visited = false;
            let opcode = self.fetch();
            if self.fetch_already_visited {
                self.log(LogLevel::Verbose, format!("ALREADY BEEN AT {:#x}!", pc));
                if pc > self.current_entry_point {
                    let cur = self.current_entry_point;
                    let nl = self.current_entry_point_needs_label;
                    self.add_range(cur, pc.saturating_sub(1), vec![pc], nl);
                }
                self.restart_from_another_entry_point();
                continue;
            }
            let text = dis.disasm_instruction(self, opcode);
            self.log(LogLevel::Debug, format!("{:#x}: {}", pc, text));
            self.disasm.insert(pc, text);
        }
    }

    // ---------- Listing ----------

    /// Render the assembly listing into `path`.
    pub fn save_disassembly_listing(
        &self,
        dis: &dyn Disassembler,
        path: &Path,
    ) -> std::io::Result<()> {
        crate::listing::write(self, dis, path)
    }

    /// Compute the variable's name for use by the listing emitter.
    pub fn variable_name_for(&self, dis: &dyn Disassembler, addr: u32) -> String {
        if let Some(s) = dis.variable_name(self, addr) {
            return s;
        }
        if let Some(v) = self.variables.get(&addr) {
            return v.name.clone();
        }
        crate::hex16(addr)
    }

    /// Compute the label's name for use by the listing emitter.
    pub fn label_name_for(&self, dis: &dyn Disassembler, addr: u32, prefix: &str) -> String {
        if let Some(s) = dis.label_name(self, addr) {
            return s;
        }
        if let Some(v) = self.variables.get(&addr) {
            return v.name.clone();
        }
        if let Some(s) = self.labels.get(&addr) {
            return s.clone();
        }
        format!("{}{:04X}", prefix, addr)
    }

    /// Sort the visited ranges by start address (used by the listing
    /// and by tests).
    pub fn sorted_ranges(&self) -> Vec<CodeBlock> {
        let mut v = self.visited_ranges.clone();
        v.sort_by_key(|b| b.start);
        v
    }

    fn log(&self, level: LogLevel, msg: String) {
        if (self.log_level as u8) >= (level as u8) {
            eprintln!("{}", msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trivial disassembler for unit tests: every byte is a 1-byte
    /// instruction; opcode 0x00 is `ret`, 0xff is illegal, all others
    /// are no-ops.
    struct TrivialDis;
    impl Disassembler for TrivialDis {
        fn disasm_instruction(&mut self, t: &mut Tracer, opcode: u8) -> String {
            match opcode {
                0x00 => {
                    t.return_from_subroutine();
                    "ret".to_owned()
                }
                0xff => {
                    t.illegal_instruction(opcode);
                    "ILLEGAL".to_owned()
                }
                _ => format!("nop {:#x}", opcode),
            }
        }
    }

    #[test]
    fn straight_line_until_ret() {
        // 5 nops then ret.
        let mut t = Tracer::from_rom(vec![0x10, 0x11, 0x12, 0x13, 0x14, 0x00]);
        let mut d = TrivialDis;
        t.run(&mut d, &[0]);
        assert_eq!(t.disasm.len(), 6);
        assert_eq!(t.disasm[&0], "nop 0x10");
        assert_eq!(t.disasm[&5], "ret");
        let ranges = t.sorted_ranges();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start, 0);
        assert_eq!(ranges[0].end, 5);
    }

    #[test]
    fn illegal_opcode_terminates_crawl() {
        let mut t = Tracer::from_rom(vec![0x10, 0xff, 0x12]);
        let mut d = TrivialDis;
        t.run(&mut d, &[0]);
        // 0x10 disassembled, 0xff disassembled (illegal), 0x12 not visited.
        assert_eq!(t.disasm.len(), 2);
        assert!(!t.disasm.contains_key(&2));
    }
}
