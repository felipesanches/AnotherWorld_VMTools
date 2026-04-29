#!/bin/bash
# validation/run_phase_a.sh — Phase-A parity check for the Rust port of
# banks2resources.
#
# Runs the Python reference (releases/common_data/banks2resources.py)
# and the Rust port (target/release/banks2resources) over the same
# MSDOS-format `memlist.bin` + `bank<NN>` directory and asserts that
# their outputs are byte-identical.
#
# Usage:
#     validation/run_phase_a.sh <input_dir>
#
# <input_dir> must contain `memlist.bin` and `bank01`..`bank0d`.
# If $AWVM_PY_EXECTRACE is unset, the script downloads the wheel of
# `exectrace==0.0.5` from PyPI into a tmp dir and uses that, so the
# validation works on a clean checkout without `pip install`.

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
PY_OUT="$OUT_DIR/python"
RS_OUT="$OUT_DIR/rust"

rm -rf "$OUT_DIR"
mkdir -p "$PY_OUT" "$RS_OUT"

# --- Resolve Python ExecTrace ---------------------------------------------
EXECTRACE_PATH="${AWVM_PY_EXECTRACE:-}"
if [[ -z "$EXECTRACE_PATH" ]]; then
    if python3 -c "import exectrace" 2>/dev/null; then
        EXECTRACE_PATH=""  # already importable from system
    else
        EXECTRACE_PATH="$OUT_DIR/exectrace_wheel"
        mkdir -p "$EXECTRACE_PATH"
        echo "fetching exectrace==0.0.5 wheel from PyPI..."
        curl --fail --silent --show-error --location \
            -o "$EXECTRACE_PATH/exectrace.whl" \
            "https://files.pythonhosted.org/packages/48/79/c8006d9a81f3c71cf84ac194aeabd293ac5520c44dbce9703d2542061a43/exectrace-0.0.5-py2.py3-none-any.whl"
        python3 -c "import zipfile; zipfile.ZipFile('$EXECTRACE_PATH/exectrace.whl').extractall('$EXECTRACE_PATH')"
    fi
fi

PYTHONPATH_SET="$REPO_ROOT"
if [[ -n "$EXECTRACE_PATH" ]]; then
    PYTHONPATH_SET="$EXECTRACE_PATH:$PYTHONPATH_SET"
fi

# --- Build the Rust port (release) ----------------------------------------
echo "building Rust port..."
( cd "$REPO_ROOT" && cargo build --release --quiet )

# --- Run Python reference -------------------------------------------------
echo "running Python reference..."
export PYTHONPATH="$PYTHONPATH_SET"
export AWVM_VAL_INPUT="$INPUT_DIR"
export AWVM_VAL_OUT="$PY_OUT"
python3 - <<'PY' > "$PY_OUT/stdout.txt"
import os
from releases.common_data.banks2resources import Resources
INPUT = os.environ['AWVM_VAL_INPUT']
OUT = os.environ['AWVM_VAL_OUT']
memlist = open(f"{INPUT}/memlist.bin", "rb")
Resources(INPUT, OUT, memlist).generate(uppercase=False)
PY
unset PYTHONPATH AWVM_VAL_INPUT AWVM_VAL_OUT

# --- Run Rust port --------------------------------------------------------
echo "running Rust port..."
"$REPO_ROOT/target/release/banks2resources" "$INPUT_DIR" "$RS_OUT" > "$RS_OUT/stdout.txt"

# --- Compare --------------------------------------------------------------
fail=0

echo ""
echo "--- comparing stdout (per-entry log lines) ---"
if diff "$PY_OUT/stdout.txt" "$RS_OUT/stdout.txt" >/dev/null; then
    echo "OK: stdout identical"
else
    echo "FAIL: stdout differs:"
    diff "$PY_OUT/stdout.txt" "$RS_OUT/stdout.txt" | head -30
    fail=1
fi

echo ""
echo "--- comparing extracted resource binaries ---"
if diff -r "$PY_OUT/resources/" "$RS_OUT/resources/" >/dev/null; then
    py_count=$(ls "$PY_OUT/resources/" | wc -l)
    rs_count=$(ls "$RS_OUT/resources/" | wc -l)
    echo "OK: $py_count Python / $rs_count Rust resource files all byte-identical"
else
    echo "FAIL: resource files differ:"
    diff -rq "$PY_OUT/resources/" "$RS_OUT/resources/" | head -20
    fail=1
fi

echo ""
if [[ $fail -ne 0 ]]; then
    echo "Phase A parity check: FAILED"
    exit 1
fi
echo "Phase A parity check: PASSED"
