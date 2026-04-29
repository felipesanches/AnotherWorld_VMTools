//! Disassembly listing emitter — port of
//! `ExecTrace.save_disassembly_listing`.
//!
//! The output format is intentionally bit-identical to the Python
//! reference: same indentation, same `db <hex>, ...` width (8 bytes
//! per line), same handling of strings / pointer tables.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use crate::codeblock::CodeBlock;
use crate::tracer::{Disassembler, Tracer, VarKind};
use crate::{hex16, hex8};

pub(crate) fn write(
    tracer: &Tracer,
    dis: &dyn Disassembler,
    path: &Path,
) -> io::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(dis.output_disasm_headers().as_bytes())?;

    for r in &tracer.relocation_blocks {
        let reloc_to = r.to_address;
        let reloc_length = r.length;

        let mut ranges: Vec<CodeBlock> = tracer
            .visited_ranges
            .iter()
            .filter(|cb| cb.start >= reloc_to && cb.end < reloc_to + reloc_length)
            .cloned()
            .collect();
        ranges.sort_by_key(|cb| cb.start);

        write!(f, "\n\n\torg {}\n", hex16(reloc_to))?;
        let mut next_addr = reloc_to;

        // Hack from the Python reference: append a sentinel block at
        // (reloc_to + reloc_length, -1) so the loop emits any trailing
        // data region.
        ranges.push(CodeBlock {
            start: reloc_to + reloc_length,
            end: u32::MAX, // sentinel; comparisons use start, so end is irrelevant
            next_block: Vec::new(),
            subroutines: Default::default(),
            needs_label: false,
        });

        for codeblock in &ranges {
            if codeblock.start < next_addr {
                continue;
            }

            if codeblock.start > next_addr {
                emit_data_region(
                    &mut f,
                    tracer,
                    dis,
                    next_addr,
                    codeblock.start,
                )?;
            }

            // Emit the codeblock's instructions. The sentinel block's
            // start == reloc_to + reloc_length and its end is the
            // sentinel u32::MAX; we never want to iterate that.
            if codeblock.start == reloc_to + reloc_length {
                break;
            }

            let mut indent = if tracer.labeled_addresses.contains(&codeblock.start) {
                format!(
                    "\n{}:\n\t",
                    tracer.label_name_for(dis, codeblock.start, "LABEL_")
                )
            } else {
                "\t".to_owned()
            };

            for address in codeblock.start..=codeblock.end {
                if let Some(text) = tracer.disasm.get(&address) {
                    write!(f, "{}{}\n", indent, text)?;
                    indent = "\t".to_owned();
                }
            }
            next_addr = codeblock.end + 1;
        }
    }

    Ok(())
}

#[allow(unused_assignments)] // `indent` mid-loop assignments may be re-overwritten before use, depending on the iteration path
fn emit_data_region(
    f: &mut File,
    tracer: &Tracer,
    dis: &dyn Disassembler,
    start: u32,
    end_exclusive: u32,
) -> io::Result<()> {
    // Match Python's prefix sequence: first line has the label, then
    // subsequent lines indent with just `\t`.
    let mut indent = format!(
        "{}:\n\t",
        tracer.label_name_for(dis, start, "LABEL_")
    );
    let mut data: Vec<String> = Vec::new();

    let mut addr = start;
    while addr < end_exclusive {
        if let Some(var) = tracer.variables.get(&addr).cloned() {
            // Flush any in-progress hex run before switching to a typed variable.
            if !data.is_empty() {
                writeln!(f, "{}db {}", indent, data.join(", "))?;
                indent = "\t".to_owned();
                data.clear();
            }
            indent = format!("{}:\n\t", var.name);

            match var.kind {
                VarKind::Str(n) => {
                    let mut s = String::new();
                    for _ in 0..n {
                        s.push(tracer.read_byte(addr) as char);
                        addr += 1;
                    }
                    writeln!(f, "{}db \"{}\"", indent, s)?;
                    indent = format!(
                        "{}:\n\t",
                        tracer.label_name_for(dis, addr, "LABEL_")
                    );
                    continue;
                }
                VarKind::NMinus1Str => {
                    let n = tracer.read_byte(addr);
                    addr += 1;
                    let mut s = String::new();
                    for _ in 0..n.saturating_sub(1) {
                        s.push(tracer.read_byte(addr) as char);
                        addr += 1;
                    }
                    writeln!(f, "{}db {}, \"{}\"", indent, n, s)?;
                    indent = format!(
                        "{}:\n\t",
                        tracer.label_name_for(dis, addr, "LABEL_")
                    );
                    continue;
                }
                VarKind::JumpTable(n) | VarKind::Pointers(n) => {
                    writeln!(f)?;
                    let mut local_indent = indent.clone();
                    for _ in 0..n {
                        let lo = tracer.read_byte(addr);
                        addr += 1;
                        let hi = tracer.read_byte(addr);
                        addr += 1;
                        let jump_addr = u32::from(lo) | (u32::from(hi) << 8);
                        writeln!(
                            f,
                            "{}dw {}",
                            local_indent,
                            tracer.label_name_for(dis, jump_addr, "LABEL_")
                        )?;
                        local_indent = "\t".to_owned();
                    }
                    indent = format!(
                        "{}:\n\t",
                        tracer.label_name_for(dis, addr, "LABEL_")
                    );
                    continue;
                }
                VarKind::Var | VarKind::Label => {
                    // Fall through to plain hex emission with the
                    // typed-variable-bound `indent`.
                }
            }
        }

        // Plain hex byte.
        match tracer.rom_address(addr) {
            Some((i, off)) => {
                data.push(hex8(tracer.rom[i][off]));
            }
            None => {
                if !data.is_empty() {
                    writeln!(f, "{}db {}", indent, data.join(", "))?;
                    data.clear();
                }
                addr += 1;
                continue;
            }
        }

        if data.len() == 8 {
            writeln!(f, "{}db {}", indent, data.join(", "))?;
            indent = "\t".to_owned(); // subsequent overflow lines use bare-tab indent
            data.clear();
        }
        addr += 1;
    }

    if !data.is_empty() {
        writeln!(f, "{}db {}", indent, data.join(", "))?;
    }
    Ok(())
}
