#!/usr/bin/env python3
"""Performance comparison between the Python reference and the Rust port.

Runs each pipeline stage N times for each implementation and reports
the median wall-clock and 25–75% range.

Stages benchmarked:
  - banks2resources     extract every resource from the cached MSDOS banks
  - awvm-disasm         full pipeline (banks→resources→romset→disasm→.asm)
  - awvm-asm            assemble one level (level 1 — average size)

Usage:
    validation/run_perf_comparison.py <input_dir> [--runs N]
"""

import argparse
import os
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def run_timed(cmd, env=None, cwd=None):
    """Run `cmd`, capture wall time. Returns (seconds, exit_code)."""
    t0 = time.perf_counter()
    r = subprocess.run(
        cmd, env=env, cwd=cwd, capture_output=True, check=False
    )
    elapsed = time.perf_counter() - t0
    return elapsed, r.returncode


def measure(label, cmd, runs, *, env=None, cwd=None, prep=None):
    """Run `cmd` `runs` times after `runs/2` (or 1) warmup. Print stats."""
    samples = []
    # Warmup runs prime the disk cache and JIT-like warmup if any.
    warmup = max(1, runs // 4)
    for _ in range(warmup):
        if prep:
            prep()
        run_timed(cmd, env=env, cwd=cwd)
    for _ in range(runs):
        if prep:
            prep()
        elapsed, rc = run_timed(cmd, env=env, cwd=cwd)
        if rc != 0:
            print(f"  [{label}] non-zero exit ({rc}); aborting")
            return None
        samples.append(elapsed)
    samples.sort()
    median = statistics.median(samples)
    p25 = samples[len(samples) // 4]
    p75 = samples[(3 * len(samples)) // 4]
    print(
        f"  {label:<30}  median {median*1000:8.1f} ms   "
        f"p25 {p25*1000:7.1f}  p75 {p75*1000:7.1f}  "
        f"({runs} runs)"
    )
    return median


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input_dir", help="MSDOS aworld/ directory (must contain memlist.bin)")
    parser.add_argument("--runs", type=int, default=10)
    args = parser.parse_args()

    input_dir = Path(args.input_dir).resolve()
    if not (input_dir / "memlist.bin").is_file():
        sys.exit(f"error: {input_dir}/memlist.bin not found")

    # --- Resolve Python ExecTrace ---
    exectrace_path = REPO / "validation" / "_out" / "exectrace_wheel"
    if not (exectrace_path / "exectrace" / "__init__.py").is_file():
        try:
            subprocess.run(
                ["python3", "-c", "from exectrace import ExecTrace"],
                check=True, capture_output=True,
            )
            exectrace_path = None
        except subprocess.CalledProcessError:
            exectrace_path.mkdir(parents=True, exist_ok=True)
            wheel = exectrace_path / "exectrace.whl"
            print("fetching exectrace wheel...")
            subprocess.run(
                [
                    "curl", "--fail", "--silent", "--show-error", "--location",
                    "-o", str(wheel),
                    "https://files.pythonhosted.org/packages/48/79/c8006d9a81f3c71cf84ac194aeabd293ac5520c44dbce9703d2542061a43/exectrace-0.0.5-py2.py3-none-any.whl",
                ],
                check=True,
            )
            import zipfile
            with zipfile.ZipFile(wheel) as zf:
                zf.extractall(exectrace_path)

    pypath_parts = [str(REPO)]
    if exectrace_path:
        pypath_parts.insert(0, str(exectrace_path))
    py_env = {**os.environ, "PYTHONPATH": ":".join(pypath_parts)}

    # --- Build the Rust port (release) ---
    print("building Rust port (release)...")
    subprocess.run(
        ["cargo", "build", "--release", "--quiet"],
        cwd=REPO, check=True,
    )

    # --- Make a workspace dir for both runs ---
    work = Path(tempfile.mkdtemp(prefix="awvm_perf_"))
    py_dir = work / "py"
    rs_dir = work / "rs"
    py_dir.mkdir(); rs_dir.mkdir()
    shutil.copytree(REPO / "hardcoded_data", py_dir / "hardcoded_data")
    shutil.copytree(REPO / "hardcoded_data", rs_dir / "hardcoded_data")

    print(f"workspace: {work}")
    print(f"runs per measurement: {args.runs} (plus ~{max(1, args.runs // 4)} warmup)")
    print()

    # ---------- Benchmark 1: banks2resources ----------
    print("[1] banks2resources — extract 146 resources from MSDOS banks")

    # Python
    py_bnr_cmd = [
        "python3", "-c",
        f"""
import sys
sys.path.insert(0, '{REPO}')
from releases.common_data.banks2resources import Resources
import io
m = open('{input_dir}/memlist.bin', 'rb')
import os
os.makedirs('out_bnr', exist_ok=True)
Resources('{input_dir}', 'out_bnr', m).generate(uppercase=False)
""",
    ]
    py_med = measure(
        "  python (banks2resources.py)", py_bnr_cmd, args.runs,
        env=py_env, cwd=py_dir,
        prep=lambda: shutil.rmtree(py_dir / "out_bnr", ignore_errors=True),
    )

    # Rust
    rs_bnr_cmd = [
        str(REPO / "target/release/banks2resources"),
        str(input_dir), "out_bnr",
    ]
    rs_med = measure(
        "  rust   (target/release)", rs_bnr_cmd, args.runs,
        cwd=rs_dir,
        prep=lambda: shutil.rmtree(rs_dir / "out_bnr", ignore_errors=True),
    )

    speedup_bnr = (py_med / rs_med) if (py_med and rs_med) else None

    print()

    # ---------- Benchmark 2: full awvm-disasm pipeline ----------
    print("[2] awvm-disasm — full pipeline for all 9 levels")
    py_dis_cmd = [
        "python3", "-c",
        f"""
import sys, os
sys.path.insert(0, '{REPO}')
from releases.common_data.decode_polygons import PolygonDecoder
PolygonDecoder.extract_polygon_data = lambda self, *a, **k: None
sys.argv = ['', '{input_dir}', 'all_levels', 'msdos']
exec(open('{REPO}/awvm-disasm.py').read())
""",
    ]
    py_med2 = measure(
        "  python (awvm-disasm.py)", py_dis_cmd, args.runs,
        env=py_env, cwd=py_dir,
        prep=lambda: shutil.rmtree(py_dir / "output", ignore_errors=True),
    )

    # Apples-to-apples: the Python disasm above runs with the polygon
    # decoder stubbed (it crashes mid-run otherwise). The Rust build
    # gets `--no-polygons` so it skips the same step.
    rs_dis_cmd = [
        str(REPO / "target/release/awvm-disasm"),
        str(input_dir), "all_levels", "msdos", "--no-polygons",
    ]
    rs_med2 = measure(
        "  rust   (target/release)", rs_dis_cmd, args.runs,
        cwd=rs_dir,
        prep=lambda: shutil.rmtree(rs_dir / "output", ignore_errors=True),
    )
    speedup_dis = (py_med2 / rs_med2) if (py_med2 and rs_med2) else None

    print()

    # ---------- Benchmark 3: awvm-asm on level 1 ----------
    # First make sure the input .asm exists.
    print("[3] awvm-asm — assemble one level (level 1)")
    sample_asm = py_dir / "output/msdos/disasm/level_1/msdos_level-1.asm"
    if not sample_asm.is_file():
        # Run python disasm once to materialise the input.
        subprocess.run(py_dis_cmd, env=py_env, cwd=py_dir, capture_output=True, check=True)
    py_asm_input = py_dir / "level_1.asm"
    shutil.copy(sample_asm, py_asm_input)
    rs_asm_input = rs_dir / "level_1.asm"
    shutil.copy(sample_asm, rs_asm_input)

    py_asm_cmd = ["python3", f"{REPO}/awvm-asm.py", str(py_asm_input)]
    py_med3 = measure(
        "  python (awvm-asm.py)", py_asm_cmd, args.runs,
        env=py_env,
        prep=lambda: (rs_dir / "level_1.bin").exists() and os.unlink(py_dir / "level_1.bin"),
    )
    rs_asm_cmd = [str(REPO / "target/release/awvm-asm"), str(rs_asm_input)]
    rs_med3 = measure(
        "  rust   (target/release)", rs_asm_cmd, args.runs,
        prep=lambda: (rs_dir / "level_1.bin").exists() and os.unlink(rs_dir / "level_1.bin"),
    )
    speedup_asm = (py_med3 / rs_med3) if (py_med3 and rs_med3) else None

    print()
    print("=" * 70)
    print("Summary")
    print("=" * 70)
    if speedup_bnr:
        print(f"  banks2resources:  rust is {speedup_bnr:5.1f}x  faster than python")
    if speedup_dis:
        print(f"  awvm-disasm:      rust is {speedup_dis:5.1f}x  faster than python")
    if speedup_asm:
        print(f"  awvm-asm:         rust is {speedup_asm:5.1f}x  faster than python")
    print()
    print(f"workspace preserved at {work}")


if __name__ == "__main__":
    main()
