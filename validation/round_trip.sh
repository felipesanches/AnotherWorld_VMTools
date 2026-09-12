#!/bin/bash
# validation/round_trip.sh — Rust-only round-trip validator.
#
# Asserts that, for every level of a bank-format input, the
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
# Every level is checked even when an earlier one fails; the exit status is
# non-zero if any level failed. (It used to abort at the first failure — see the
# SIGPIPE note in the loop below.)
#
# Usage:
#     validation/round_trip.sh <input_dir> [slug]
#
# slug defaults to `msdos`. For `msdos`, <input_dir> must contain
# `memlist.bin` and `bank01..bank0d`; for `amiga` it is the bank
# directory containing `another` (which carries the memlist inline)
# plus `bank01..bank0d`.
#
# Both priority ports are gated because they fail differently: the
# registry's cross-port chain walk only runs for siblings, so a defect
# in it shows up on msdos while amiga stays clean (#0118).

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <input_dir> [slug]" >&2
    exit 2
fi
INPUT_DIR="$1"
SLUG="${2:-msdos}"
case "$SLUG" in
    msdos)
        if [[ ! -f "$INPUT_DIR/memlist.bin" ]]; then
            echo "error: $INPUT_DIR/memlist.bin not found" >&2
            exit 2
        fi
        ;;
    amiga)
        if [[ ! -f "$INPUT_DIR/another" ]]; then
            echo "error: $INPUT_DIR/another not found (amiga carries its memlist inline)" >&2
            exit 2
        fi
        ;;
    *)
        echo "error: unsupported slug $SLUG (expected msdos or amiga)" >&2
        exit 2
        ;;
esac

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$REPO_ROOT/validation/_out"
WORK="$OUT_DIR/rust"

rm -rf "$OUT_DIR"
mkdir -p "$WORK"
cp -a "$REPO_ROOT/hardcoded_data" "$WORK/"

echo "building Rust port..."
( cd "$REPO_ROOT" && cargo build --release --quiet )

# Polygons are left on: `awvm-disasm … all_levels amiga` used to panic in the
# polygon decoder after emitting every level (#0097), so disabling them here
# would have hidden that. It costs a few seconds and several thousand SVGs.
echo "running Rust disasm over all levels ($SLUG)..."
( cd "$WORK" && "$REPO_ROOT/target/release/awvm-disasm" "$INPUT_DIR" all_levels "$SLUG" > stdout.txt 2>&1 ) || {
    echo "awvm-disasm exited non-zero; tail of its output:" >&2
    tail -20 "$WORK/stdout.txt" >&2
    exit 1
}

# Label- and cinematic-name collisions are reported on stderr by the
# disassembler (#0118). They no longer corrupt bytes, but they are naming debt
# and should be visible in the gate output rather than buried in a log.
if grep -q "collision(s)" "$WORK/stdout.txt"; then
    echo ""
    echo "--- name collisions reported by awvm-disasm (naming debt, not byte errors) ---"
    grep -E "collision\(s\)|is held by" "$WORK/stdout.txt" | sed 's/^/  /'
fi

DISASM_DIR="$WORK/output/$SLUG/disasm"
GAMEROM="$WORK/output/$SLUG/romset/bytecode.rom"

fail=0
echo ""
echo "--- round-trip: rust disasm → rust asm == original bytecode ---"
for lvl in 0 1 2 3 4 5 6 7 8; do
    asm="$DISASM_DIR/level_$lvl/${SLUG}_level-$lvl.asm"
    bin="${asm%.asm}.bin"
    "$REPO_ROOT/target/release/awvm-asm" "$asm" >/dev/null 2>&1

    dd if="$GAMEROM" of="$OUT_DIR/orig_level_$lvl.bin" bs=65536 count=1 skip=$lvl 2>/dev/null
    if cmp -s "$OUT_DIR/orig_level_$lvl.bin" "$bin"; then
        printf "  level %d: OK (%d bytes)\n" "$lvl" "$(wc -c < "$bin")"
    else
        # `cmp -l | head` would hand cmp a SIGPIPE, and under `set -o pipefail`
        # that aborted the whole loop at the first failing level — so levels
        # after it were never checked and the real scope of a regression stayed
        # hidden. Capture the full diff first, then show a few lines.
        ndiff="$(cmp -l "$OUT_DIR/orig_level_$lvl.bin" "$bin" | wc -l || true)"
        printf "  level %d: FAILED (%s byte(s) differ)\n" "$lvl" "$ndiff"
        cmp -l "$OUT_DIR/orig_level_$lvl.bin" "$bin" > "$OUT_DIR/diff_level_$lvl.txt" || true
        head -5 "$OUT_DIR/diff_level_$lvl.txt" | sed 's/^/      /'
        echo "      (full diff: $OUT_DIR/diff_level_$lvl.txt)"
        fail=1
    fi
done

echo ""
if [[ $fail -ne 0 ]]; then
    echo "Round-trip check: FAILED"
    exit 1
fi
echo "Round-trip check: PASSED — all 9 $SLUG levels round-trip byte-identically."
