#!/bin/bash
# validation/run_phase_c.sh — Phase-C parity check for the Rust port of
# the AWVM disassembler.
#
# For each MSDOS level (0..8) the Python and Rust ports both
# produce `.asm` listings; this script asserts they are byte-identical.
#
# Usage:
#     validation/run_phase_c.sh <input_dir>
#
# The Python reference's polygon decoder (Phase D, not yet ported)
# crashes mid-run on this input set with `cairo.IOError`; we
# monkey-patch it to a no-op so the disasm completes for every level.

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

# Probe for the *actual symbol* we need; a bare `import exectrace`
# spuriously succeeds because the Rust workspace has its own
# `exectrace/` directory and Python 3 treats it as a namespace package.
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
# Put exectrace_wheel FIRST on PYTHONPATH so it shadows the Rust
# namespace-package collision in REPO_ROOT/exectrace/.
PYPATH="$REPO_ROOT"
[[ -n "$EXECTRACE_PATH" ]] && PYPATH="$EXECTRACE_PATH:$PYPATH"

echo "building Rust port..."
( cd "$REPO_ROOT" && cargo build --release --quiet )

# -- Python reference --
echo "running Python reference (polygon decoder stubbed)..."
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

# -- Rust port --
echo "running Rust port..."
( cd "$RS_DIR" && "$REPO_ROOT/target/release/awvm-disasm" "$INPUT_DIR" all_levels msdos > stdout.txt 2>&1 )

# -- Compare --
echo ""
echo "--- per-level disasm parity ---"
fail=0
for lvl in 0 1 2 3 4 5 6 7 8; do
    py="$PY_DIR/output/msdos/disasm/level_$lvl/msdos_level-$lvl.asm"
    rs="$RS_DIR/output/msdos/disasm/level_$lvl/msdos_level-$lvl.asm"
    if [[ ! -f "$py" ]]; then echo "  level $lvl: MISSING (python)"; fail=1; continue; fi
    if [[ ! -f "$rs" ]]; then echo "  level $lvl: MISSING (rust)"; fail=1; continue; fi
    if diff -q "$py" "$rs" >/dev/null; then
        printf "  level %d: OK (%d bytes)\n" "$lvl" "$(wc -c < "$py")"
    else
        echo "  level $lvl: DIFFERS"
        diff "$py" "$rs" | head -10
        fail=1
    fi
done

echo ""
if [[ $fail -ne 0 ]]; then
    echo "Phase C parity check: FAILED"
    exit 1
fi
echo "Phase C parity check: PASSED"
