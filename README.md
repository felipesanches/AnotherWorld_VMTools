# Another World VM Tools

Toolchain for software development targeting the virtual machine
originally designed for Eric Chahi's Another World (1991).

This toolchain is now a **pure-Rust workspace**. The original
Python implementation it grew out of (`awvm-disasm.py`,
`awvm-asm.py`, `releases/<release>/*.py`, `releases/common_data/*.py`)
was retired in 2026 once the Rust port reached byte-identical
parity with it on every release we had a fixture for. The git
history retains the Python source verbatim — `git log` and
`git show <sha>:awvm-disasm.py` recover it.

Licensed under GPL-3.0-or-later.

## Workspace layout

```
exectrace/      # CPU-agnostic instruction-trace framework
                # (Rust port of github.com/felipesanches/ExecTrace)
awvm/           # Another World VM library: unpacker, memlist parser,
                # bank reader, disassembler, polygon decoder,
                # assembler, OFS-format ADF reader, plus per-release
                # data tables (KNOWN_LABELS, STAGE_TITLES, etc.)
awvm-tools/     # CLI binaries:
                #   awvm-disasm     — full-pipeline disassembly
                #                     (banks2resources → resources2romset
                #                      → trace + emit per-level .asm
                #                      → polygon-SVG extraction)
                #   awvm-asm        — assemble a .asm back into bytecode;
                #                     round-trips byte-identical
                #   adf-extract     — unpack files from OFS-formatted
                #                     Amiga Disk File images
                #   banks2resources — pre-pipeline standalone resource
                #                     extractor (used by awvm-disasm)
hardcoded_data/ # Text/font ROMs the disassembler/assembler need for
                # the `text` opcode. Used by every release that
                # doesn't extract its own strings from the cartridge.
example/        # Sample assembly programs (bounce.asm, pong.asm)
validation/     # Round-trip integrity check (Rust-only)
```

## Build

```bash
cargo build --release
```

The release build uses LTO and one codegen unit (~5 s on a modern
laptop). Resulting binaries live in `target/release/`.

## Disassemble a release

For a release packaged as `memlist.bin` + `bank<NN>` files (msdos,
amiga), or extracted from a cartridge ROM (snes, genesis_europe,
gba_usa, symbian_demo):

```bash
./target/release/awvm-disasm <input_dir> all_levels <release_slug>
```

Where `<release_slug>` is one of: `msdos`, `amiga`, `snes`,
`genesis_europe`, `gba_usa`, `symbian_demo`. Run with
`--no-polygons` to skip polygon-SVG extraction (useful for fast
iteration or when comparing against a reference that doesn't
generate them).

For Amiga ADF input, unpack first:

```bash
./target/release/adf-extract DiskA.adf DiskB.adf <unpacked_dir>
./target/release/awvm-disasm <unpacked_dir> all_levels amiga
```

## Reassemble a level

```bash
./target/release/awvm-asm <input.asm>
```

Writes `<input>.bin` next to the input. Round-trip with the
disassembler is byte-identical thanks to `;@raw=...` annotations
emitted by `awvm-disasm` (each instruction's annotation captures
the exact byte sequence the disassembler consumed, so the
assembler reproduces those bytes verbatim — even when the
canonical encoding would have been lossy in pre-existing opcodes
like the 0x40-family video instructions or the `setPalette`
waste byte).

## Round-trip integrity check

```bash
validation/round_trip.sh <msdos_input_dir>
```

Asserts that every MSDOS level disasm-then-asm reproduces the
original bytecode byte-for-byte. The Python-vs-Rust parity
harnesses (`run_phase_*.sh`, `run_perf_comparison.py`) that
guarded this property during the port were retired alongside
the Python implementation.

## Running compiled "romsets"

The
[`anotherworld` MAME fork](https://github.com/felipesanches/mame/tree/anotherworld)
implements the Another World VM and can run any romset
`awvm-asm` produces. Copy the text-string and font ROMs from
`hardcoded_data/` into your MAME rompath alongside the assembled
output (see issue #15).
