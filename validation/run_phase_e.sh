#!/bin/bash
# validation/run_phase_e.sh — Phase-E parity check for the assembler.
#
# Asserts that Rust `awvm-asm` produces byte-identical bytes to
# Python `awvm-asm.py` for every level's `.asm` (which we generate
# fresh from the same MSDOS input via Phase C's disasm step).
#
# NOTE on the round-trip aspiration: the Python README states that
# `awvm-disasm → awvm-asm` should round-trip to the original
# bytecode bytes. In practice it does not for levels 1..7 of the
# MSDOS release: the disasm discards a few unused/uninterpreted
# opcode bits, so the asm cannot reconstruct them. This is an
# inherent property of the Python disasm/asm pair and is observed
# identically by the Rust port. The parity check this harness
# performs is therefore Python↔Rust on the `.asm → .bin`
# transformation, not original↔reassembled.
#
# Usage:
#     validation/run_phase_e.sh <input_dir>

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
WORK="$OUT_DIR/work"

rm -rf "$OUT_DIR"
mkdir -p "$WORK"
cp -a "$REPO_ROOT/hardcoded_data" "$WORK/"

EXECTRACE_PATH=""
if ! python3 -c "from exectrace import ExecTrace" 2>/dev/null; then
    EXECTRACE_PATH="$OUT_DIR/exectrace_wheel"
    mkdir -p "$EXECTRACE_PATH"
    echo "fetching exectrace==0.0.5 wheel..."
    curl --fail --silent --show-error --location \
        -o "$EXECTRACE_PATH/exectrace.whl" \
        "https://files.pythonhosted.org/packages/48/79/c8006d9a81f3c71cf84ac194aeabd293ac5520c44dbce9703d2542061a43/exectrace-0.0.5-py2.py3-none-any.whl"
    python3 -c "import zipfile; zipfile.ZipFile('$EXECTRACE_PATH/exectrace.whl').extractall('$EXECTRACE_PATH')"
fi
PYPATH="$REPO_ROOT"
[[ -n "$EXECTRACE_PATH" ]] && PYPATH="$EXECTRACE_PATH:$PYPATH"

echo "building Rust port..."
( cd "$REPO_ROOT" && cargo build --release --quiet )

# Use the Rust disassembler to produce the .asm files (Phase C is the
# trusted source of truth for disasm output, since we already validated
# parity there). We assemble each level twice — once with the Rust
# awvm-asm, once with the Python awvm-asm.py — and diff.
echo "running Rust disassembler (Phase C) over all levels..."
( cd "$WORK" && "$REPO_ROOT/target/release/awvm-disasm" "$INPUT_DIR" all_levels msdos > stdout.txt 2>&1 )

DISASM_DIR="$WORK/output/msdos/disasm"
fail=0
echo ""
echo "--- per-level Rust↔Python awvm-asm parity ---"
for lvl in 0 1 2 3 4 5 6 7 8; do
    asm="$DISASM_DIR/level_$lvl/msdos_level-$lvl.asm"
    rs_bin="$DISASM_DIR/level_$lvl/msdos_level-$lvl.bin"
    py_asm="$WORK/level_$lvl.asm"
    py_bin="$WORK/level_$lvl.bin"

    # Rust assembles in place (replaces .asm extension with .bin).
    "$REPO_ROOT/target/release/awvm-asm" "$asm" > /dev/null

    # Python assembles a copy (also replaces .asm with .bin).
    cp "$asm" "$py_asm"
    PYTHONPATH="$PYPATH" python3 "$REPO_ROOT/awvm-asm.py" "$py_asm" > /dev/null 2>&1

    if cmp -s "$py_bin" "$rs_bin"; then
        printf "  level %d: OK (%d bytes) sha=%s\n" "$lvl" \
            "$(wc -c < "$rs_bin")" "$(sha256sum "$rs_bin" | cut -c1-12)"
    else
        echo "  level $lvl: DIFFERS"
        cmp -l "$py_bin" "$rs_bin" | head -3
        fail=1
    fi
done

echo ""
if [[ $fail -ne 0 ]]; then
    echo "Phase E parity check: FAILED"
    exit 1
fi
echo "Phase E parity check: PASSED"
