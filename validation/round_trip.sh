#!/bin/bash
# validation/round_trip.sh — Rust-only round-trip validator.
#
# Asserts that, for every level of an MSDOS-format input, the
# pipeline `awvm-disasm → awvm-asm` reproduces the original
# bytecode bit-for-bit. This is the standing correctness check
# now that the Python reference has been removed: byte-identical
# round-trip is a strong correctness signal independent of any
# external comparator.
#
# Note: the Python-vs-Rust parity harnesses that lived alongside
# this file (`run_phase_a.sh`, `run_phase_c.sh`, `run_phase_d.sh`,
# `run_phase_e.sh`, `run_round_trip.sh`, `run_perf_comparison.py`)
# were retired together with the Python implementation — see the
# repo's README.md for context. They proved Rust↔Python parity for
# msdos / amiga / genesis_europe at multiple commits during the
# port; the git history retains them.
#
# Usage:
#     validation/round_trip.sh <input_dir>
#
# <input_dir> must contain `memlist.bin` and `bank01..bank0d`.

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <input_dir>" >&2
    exit 2
fi
INPUT_DIR="$1"
if [[ ! -f "$INPUT_DIR/memlist.bin" ]]; then
    echo "error: $INPUT_DIR/memlist.bin not found" >&2
    exit 2
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$REPO_ROOT/validation/_out"
WORK="$OUT_DIR/rust"

rm -rf "$OUT_DIR"
mkdir -p "$WORK"
cp -a "$REPO_ROOT/hardcoded_data" "$WORK/"

echo "building Rust port..."
( cd "$REPO_ROOT" && cargo build --release --quiet )

echo "running Rust disasm over all levels..."
( cd "$WORK" && "$REPO_ROOT/target/release/awvm-disasm" "$INPUT_DIR" all_levels msdos --no-polygons > stdout.txt 2>&1 )

DISASM_DIR="$WORK/output/msdos/disasm"
GAMEROM="$WORK/output/msdos/romset/bytecode.rom"

fail=0
echo ""
echo "--- round-trip: rust disasm → rust asm == original bytecode ---"
for lvl in 0 1 2 3 4 5 6 7 8; do
    asm="$DISASM_DIR/level_$lvl/msdos_level-$lvl.asm"
    bin="${asm%.asm}.bin"
    "$REPO_ROOT/target/release/awvm-asm" "$asm" >/dev/null 2>&1

    dd if="$GAMEROM" of="$OUT_DIR/orig_level_$lvl.bin" bs=65536 count=1 skip=$lvl 2>/dev/null
    if cmp -s "$OUT_DIR/orig_level_$lvl.bin" "$bin"; then
        printf "  level %d: OK (%d bytes)\n" "$lvl" "$(wc -c < "$bin")"
    else
        echo "  level $lvl: FAILED"
        cmp -l "$OUT_DIR/orig_level_$lvl.bin" "$bin" | head -3
        fail=1
    fi
done

echo ""
if [[ $fail -ne 0 ]]; then
    echo "Round-trip check: FAILED"
    exit 1
fi
echo "Round-trip check: PASSED — all 9 MSDOS levels round-trip byte-identically."
