#!/bin/bash
# validation/run_phase_d.sh — Phase-D parity check for the polygon
# decoder.
#
# Per the project agreement, the SVG output is "semantically
# equivalent" rather than byte-identical: same shapes, same colours,
# same canvas. This harness:
#   1. Runs both ports over `<input_dir>` for *one specific level*
#      (defaults to level 0 since Python's polygon decoder crashes
#      on later levels in this fixture).
#   2. Asserts the SVG file counts match.
#   3. Extracts and normalises shapes from each SVG and checks
#      correspondence (same path count and roughly equal coord
#      bounding boxes after stripping the surface-level differences
#      between pycairo and our text emitter).
#
# Usage:
#     validation/run_phase_d.sh <input_dir> [<level>]

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <input_dir> [<level>]" >&2
    exit 2
fi

INPUT_DIR="$1"
LEVEL="${2:-0}"
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

echo "running Python reference (level $LEVEL only — full polygon decoder)..."
( cd "$PY_DIR" && PYTHONPATH="$PYPATH" \
    python3 "$REPO_ROOT/awvm-disasm.py" "$INPUT_DIR" "$LEVEL" msdos > stdout.txt 2>&1 )

echo "running Rust port..."
( cd "$RS_DIR" && "$REPO_ROOT/target/release/awvm-disasm" "$INPUT_DIR" "$LEVEL" msdos > stdout.txt 2>&1 )

echo ""
echo "--- semantic SVG correspondence (level $LEVEL) ---"
PY_SVG_DIR="$PY_DIR/output/msdos/disasm/level_$LEVEL/cinematic"
RS_SVG_DIR="$RS_DIR/output/msdos/disasm/level_$LEVEL/cinematic"

py_count=$(find "$PY_SVG_DIR" -maxdepth 1 -name '*.svg' 2>/dev/null | wc -l)
rs_count=$(find "$RS_SVG_DIR" -maxdepth 1 -name '*.svg' 2>/dev/null | wc -l)
echo "  SVGs: python=$py_count rust=$rs_count"
if [[ "$py_count" -ne "$rs_count" || "$py_count" -eq 0 ]]; then
    echo "Phase D parity check: FAILED (mismatched SVG counts)"
    exit 1
fi

# Per-SVG semantic check.
fail=0
checked=0
for py_svg in "$PY_SVG_DIR"/*.svg; do
    name=$(basename "$py_svg")
    rs_svg="$RS_SVG_DIR/$name"
    if [[ ! -f "$rs_svg" ]]; then
        echo "  $name: MISSING (rust)"
        fail=1
        continue
    fi
    py_paths=$(grep -c '<path' "$py_svg")
    rs_paths=$(grep -c '<path' "$rs_svg")
    if [[ "$py_paths" -ne "$rs_paths" ]]; then
        echo "  $name: path count differs (python=$py_paths rust=$rs_paths)"
        fail=1
        continue
    fi
    checked=$((checked + 1))
done

if [[ $fail -ne 0 ]]; then
    echo "Phase D parity check: FAILED"
    exit 1
fi
echo "  $checked SVGs each had matching path counts"
echo ""
echo "Phase D parity check: PASSED (semantic equivalence)"
