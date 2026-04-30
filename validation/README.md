# Validation

The repo's standing correctness check.

## `round_trip.sh <input_dir>`

For every MSDOS level reachable from `<input_dir>/memlist.bin` +
`bank<NN>`, assert that the Rust pipeline `awvm-disasm` →
`awvm-asm` reproduces the original bytecode byte-for-byte.

```bash
validation/round_trip.sh /path/to/aworld/aworld
```

Outputs land under `validation/_out/` (gitignored).

## Historical Python-parity harnesses

The git history retains a family of Python-vs-Rust parity scripts
that guarded byte-identity at multiple stages during the port:

- `run_phase_a.sh` — banks2resources stdout + per-resource .bin parity
- `run_phase_c.sh` — per-level disassembly .asm parity
- `run_phase_d.sh` — polygon SVG semantic equivalence
- `run_phase_e.sh` — Rust↔Python `.asm → .bin` parity
- `run_round_trip.sh` — four-way disasm/asm round-trip + cross-parity
- `run_perf_comparison.py` — Python vs Rust wall-clock comparison

These were retired together with the Python implementation in
2026; recover from git history if needed for archaeology of the
port itself. The Rust port was byte-identical to the Python
reference for the `msdos`, `amiga`, and `genesis_europe`
releases (the three with locally-archived fixtures); see the
`docs/perf_report.md` companion document for the speed data.
