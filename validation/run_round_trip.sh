#!/bin/bash
# validation/run_round_trip.sh — assert that disasm → asm reproduces
# the original bytecode byte-for-byte.
#
# This is the gate for the `;@raw=` annotation work: with the
# disassembler emitting `;@raw=...` on every instruction and the
# assembler honouring it, the disasm/asm round-trip is exact even
# where the canonical encoding is lossy (the unused bits in
# 0x40-family video opcodes, the setPalette waste byte, etc.).
#
# Per level, asserts four things:
#   1. python disasm → python asm  ==  original bytecode
#   2. rust   disasm → rust   asm  ==  original bytecode
#   3. python .asm  ==  rust .asm    (cross-implementation parity)
#   4. python .bin  ==  rust .bin    (cross-implementation parity)
#
# Usage:
#     validation/run_round_trip.sh <input_dir>

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
PY_DIR="$OUT_DIR/python"
RS_DIR="$OUT_DIR/rust"

rm -rf "$OUT_DIR"
mkdir -p "$PY_DIR" "$RS_DIR"
cp -a "$REPO_ROOT/hardcoded_data" "$PY_DIR/"
cp -a "$REPO_ROOT/hardcoded_data" "$RS_DIR/"

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

echo "running Python disasm (polygon decoder stubbed)..."
export PYTHONPATH="$PYPATH"
export AWVM_REPO="$REPO_ROOT"
export AWVM_INPUT="$INPUT_DIR"
( cd "$PY_DIR" && python3 - <<'PY' > stdout.txt 2>&1
import os, sys
sys.path.insert(0, os.environ['AWVM_REPO'])
from releases.common_data.decode_polygons import PolygonDecoder
PolygonDecoder.extract_polygon_data = lambda self, *a, **k: None
sys.argv = ['awvm-disasm.py', os.environ['AWVM_INPUT'], 'all_levels', 'msdos']
exec(open(os.path.join(os.environ['AWVM_REPO'], 'awvm-disasm.py')).read())
PY
)
unset PYTHONPATH AWVM_REPO AWVM_INPUT

echo "running Rust disasm..."
( cd "$RS_DIR" && "$REPO_ROOT/target/release/awvm-disasm" "$INPUT_DIR" all_levels msdos > stdout.txt 2>&1 )

PY_DISASM="$PY_DIR/output/msdos/disasm"
RS_DISASM="$RS_DIR/output/msdos/disasm"
GAMEROM="$RS_DIR/output/msdos/romset/bytecode.rom"

echo "reassembling..."
for lvl in 0 1 2 3 4 5 6 7 8; do
    PYTHONPATH="$PYPATH" python3 "$REPO_ROOT/awvm-asm.py" \
        "$PY_DISASM/level_$lvl/msdos_level-$lvl.asm" >/dev/null 2>&1
    "$REPO_ROOT/target/release/awvm-asm" \
        "$RS_DISASM/level_$lvl/msdos_level-$lvl.asm" >/dev/null 2>&1
    dd if="$GAMEROM" of="$OUT_DIR/orig_level_$lvl.bin" bs=65536 count=1 skip=$lvl 2>/dev/null
done

fail=0
echo ""
echo "--- (1) python disasm → python asm == original ---"
for lvl in 0 1 2 3 4 5 6 7 8; do
    if cmp -s "$OUT_DIR/orig_level_$lvl.bin" "$PY_DISASM/level_$lvl/msdos_level-$lvl.bin"; then
        printf "  level %d: OK\n" "$lvl"
    else
        echo "  level $lvl: FAILED"; fail=1
    fi
done

echo ""
echo "--- (2) rust disasm → rust asm == original ---"
for lvl in 0 1 2 3 4 5 6 7 8; do
    if cmp -s "$OUT_DIR/orig_level_$lvl.bin" "$RS_DISASM/level_$lvl/msdos_level-$lvl.bin"; then
        printf "  level %d: OK\n" "$lvl"
    else
        echo "  level $lvl: FAILED"; fail=1
    fi
done

echo ""
echo "--- (3) python .asm == rust .asm ---"
for lvl in 0 1 2 3 4 5 6 7 8; do
    if cmp -s "$PY_DISASM/level_$lvl/msdos_level-$lvl.asm" "$RS_DISASM/level_$lvl/msdos_level-$lvl.asm"; then
        printf "  level %d: OK\n" "$lvl"
    else
        echo "  level $lvl: DIFFERS"; fail=1
    fi
done

echo ""
echo "--- (4) python .bin == rust .bin ---"
for lvl in 0 1 2 3 4 5 6 7 8; do
    if cmp -s "$PY_DISASM/level_$lvl/msdos_level-$lvl.bin" "$RS_DISASM/level_$lvl/msdos_level-$lvl.bin"; then
        printf "  level %d: OK\n" "$lvl"
    else
        echo "  level $lvl: DIFFERS"; fail=1
    fi
done

echo ""
if [[ $fail -ne 0 ]]; then
    echo "Round-trip parity check: FAILED"
    exit 1
fi
echo "Round-trip parity check: PASSED — disasm/asm round-trips exactly to the original bytecode in BOTH implementations, and the two implementations are byte-identical to each other."
