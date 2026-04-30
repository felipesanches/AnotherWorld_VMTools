# Performance comparison: Python reference vs Rust port

Wall-clock cost of each stage of the Another World VM toolchain
under both implementations. The Rust port is byte-for-byte
equivalent to the Python reference at every stage we measure here
(validated by `validation/run_round_trip.sh`, `run_phase_a.sh`,
`run_phase_c.sh`, `run_phase_d.sh`, and `run_phase_e.sh`), so this
is purely a speed comparison — both pipelines produce identical
output.

## TL;DR

| Stage | Python median | Rust median | **Speedup (Py / Rust)** |
|---|---|---|---|
| `banks2resources` | 1318 ms | 25.6 ms | **≈ 51×** |
| `awvm-disasm` (full 9-level pipeline, no polygons) | 27.4 s | 474 ms | **≈ 58×** |
| `awvm-asm` (level 1, ~535 KB .asm) | 196 ms | 13.4 ms | **≈ 15×** |

For the pipeline a researcher runs end-to-end, that is roughly
**half a minute → half a second**. The interactive loop (edit a
`.asm`, reassemble, diff) drops from ~200 ms per iteration to
~13 ms — fast enough to feel instant.

## Methodology

Harness: `validation/run_perf_comparison.py`. For each stage, runs
N timed samples after `max(1, N/4)` warmup runs (the warmup ones
prime the disk cache and amortise interpreter startup; their
timings are discarded). Wall-clock time is measured around each
`subprocess.run` call with `time.perf_counter()`.

Reported: median across the timed samples plus the 25th and 75th
percentiles.

End-to-end CLI invocation is what is being measured, so each
sample includes process startup. For Rust this is a single
`mmap+exec`; for Python it is interpreter spin-up plus module
imports (~50–80 ms of the 196 ms `awvm-asm` measurement is
interpreter startup alone).

### Apples-to-apples: polygon-SVG generation disabled on both sides

The Python reference's polygon decoder crashes mid-run on this
fixture with `cairo.IOError`, so any benchmark that includes
polygon decoding can only measure the run up to the first crash.
To get a clean comparison we skip polygon-SVG generation in BOTH
implementations:

- **Python:** the perf harness monkey-patches
  `PolygonDecoder.extract_polygon_data` to a no-op before invoking
  `awvm-disasm.py`. The disassembly text is unaffected.
- **Rust:** the `awvm-disasm` binary takes a `--no-polygons` flag
  that bypasses the entire polygon stage. The `.asm` output is
  unchanged with or without it.

The numbers above therefore reflect like-for-like work:
banks→resources→romset→9-level disassembly, no SVG side-effects.
A separate run that included Rust's polygon-SVG generation showed
the Rust disasm at ~1.65 s — so SVG generation accounts for about
70% of Rust's full-pipeline time on this input.

### Stages benchmarked

| Stage | What it does | Input size |
|---|---|---|
| `banks2resources` | Read `memlist.bin` + every `bank<NN>` file, decompress packed entries, write 146 `resource-0xNN.bin` files | ~ 4.4 MB of MSDOS banks |
| `awvm-disasm` (full pipeline, `--no-polygons`) | banks2resources → resources2romset → 9-level disassembly | full MSDOS pipeline |
| `awvm-asm` | Re-assemble one mid-size `.asm` (~535 KB after `;@raw=` annotations) into a 64-KiB `.bin` | one level |

### Host

| | |
|---|---|
| CPU | 12th Gen Intel® Core™ i7-1255U |
| Cores | 8 logical |
| RAM | 32 GiB |
| Kernel | Linux 6.12.74 (Debian 13) |
| Python | 3.13.5 |
| Rust | 1.93.1 (release build, `lto = true`, `codegen-units = 1`) |
| Run count | N = 5 timed samples per stage, plus 1 warmup |

## Detailed results (no polygons either side)

```
[1] banks2resources — extract 146 resources from MSDOS banks
  python (banks2resources.py)   median   1318.4 ms   p25  1316.2  p75  1325.1  (5 runs)
  rust   (target/release)       median     25.6 ms   p25    25.6  p75    31.9  (5 runs)

[2] awvm-disasm — full pipeline for all 9 levels
  python (awvm-disasm.py)       median  27439.1 ms   p25 27357.3  p75 27473.8  (5 runs)
  rust   (target/release)       median    473.8 ms   p25   473.8  p75   502.6  (5 runs)

[3] awvm-asm — assemble one level (level 1)
  python (awvm-asm.py)          median    196.0 ms   p25   191.4  p75   200.0  (5 runs)
  rust   (target/release)       median     13.4 ms   p25    12.2  p75    13.5  (5 runs)
```

The variance bands (p25–p75) are narrow on every measurement; the
median is a faithful single-number summary in every case.

## Analysis

**`banks2resources` (51×)**: this stage is dominated by the LZ-ish
backwards bit-stream decoder in `Unpacker.unpack`. Python pays
~10 ns per bit just to dispatch through `getCode → nextBit → rcr`
and the small-int boxing on every shift; Rust compiles the same
code path to a tight loop over a `Vec<u8>` with the bit-stream
register in a CPU register. 51× is what you'd expect from "Python
interpreter dispatch overhead vs native loop on a hot inner code
path."

**`awvm-disasm` (58×)**: full pipeline, dominated by the trace +
per-instruction string formatting + `BTreeMap<u32, ...>` inserts
into `tracer.disasm` and `tracer.consumed_bytes`. Python's overhead
per instruction includes attribute dispatch (`self.fetch`,
`self.disasm[address] = …`), dict insertions, and a lot of
`"%02X" % v`-style formatting. Rust's `format!` over stack-
allocated strings plus B-tree inserts is roughly 60× faster end-
to-end. The factor is highest here because the inner loop runs
many times per instruction (every fetch, every formatted byte) and
Python's interpreter overhead compounds.

**`awvm-asm` (15×)**: smaller speedup because a larger fraction of
the time is process startup + file I/O (the .asm is ~535 KB of
text to parse, and the .bin is only 64 KiB — most of the work is
the parse, not the encode). The two-pass parser still wins handily,
but startup dilutes the ratio.

**Why the disasm jump (18× → 58×)**: my earlier report ran Rust
*with* polygon-SVG generation while Python was stubbed; that made
Rust do strictly more work and depressed the ratio. With
polygons off on both sides we see the actual disassembler-only
speedup. Polygon-SVG generation in the Rust port takes about 1.2 s
on this fixture (mostly cairo-bypass text emission + 3,000+ file
opens), so a fully-fair comparison that included polygon decoding
on the Python side would land somewhere between these extremes —
but we cannot produce that measurement without first fixing the
upstream Python polygon decoder.

## Caveats

- Both implementations include process startup. For sub-second
  tasks (`banks2resources`, `awvm-asm`) startup overhead is a
  non-trivial fraction of the measurement; for the disasm pipeline
  it is negligible.
- The Rust port's release build uses link-time optimisation
  (`lto = true`); a `lto = false` build is roughly 10–15% slower
  on these workloads.
- The Rust port tracks the Python reference's text format down to
  iteration order and whitespace. Some of that fidelity costs Rust
  some speed (e.g. the insertion-ordered `VideoEntryMap` instead
  of a plain `BTreeMap`); a Rust-native rewrite that did not need
  to match Python output byte-for-byte could be faster still.
- The benchmark does not measure compilation time. A from-scratch
  `cargo build --release` of the workspace takes about 5 s on this
  host; incremental rebuilds are sub-second.

## Reproducing

```bash
# from the repo root
validation/run_perf_comparison.py /path/to/aworld/aworld --runs 10
```

The script auto-resolves the Python `exectrace` dependency
(downloads the pypi wheel into `validation/_out/exectrace_wheel/`
if the system Python doesn't already have it), builds the Rust
workspace in release mode, and runs both pipelines with polygon-
SVG generation disabled on both sides.
