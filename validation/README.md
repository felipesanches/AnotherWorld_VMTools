# Validation harnesses for the Rust port

Each `run_phase_*.sh` runs both the Python reference implementation
and the corresponding Rust port over the same input, and asserts the
outputs match. These scripts are the gates each port phase must pass
before merging to `main`.

| Phase | Script | What it asserts |
|---|---|---|
| A | `run_phase_a.sh <input_dir>` | `banks2resources` Python and Rust ports produce byte-identical resource binaries (146 files for MSDOS) and identical stdout. |
| B | (TODO) | ExecTrace mechanics — covered indirectly via the disassembler in C. |
| C | (TODO) | `awvm-disasm` text output is byte-identical for every BYTECODE resource. |
| D | (TODO) | Polygon decoder produces semantically-equivalent SVG (per the user's stated bar). |
| E | (TODO) | Round-trip `awvm-disasm → awvm-asm → bytes` is byte-identical to the input bytecode for every level. |

## Phase A in 30 seconds

```bash
# input_dir must contain memlist.bin + bank01..bank0d (the MSDOS layout).
validation/run_phase_a.sh /path/to/aworld_unpacked/aworld/aworld
```

The script:
- Builds the Rust workspace in `--release`.
- Resolves the Python `exectrace` package (downloads the pypi wheel into
  `validation/_out/exectrace_wheel/` if the system Python doesn't have it).
- Runs `releases/common_data/banks2resources.py` against the input.
- Runs `target/release/banks2resources` against the same input.
- `diff`s their `stdout` and their `resources/` outputs.
- Exits 0 on match, 1 on any divergence.

Outputs land under `validation/_out/`, which is gitignored.
