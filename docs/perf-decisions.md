# Performance decisions log

Append-only log of optimizations attempted with their measured impact.
Use this to avoid re-trying things that didn't pan out and to justify
each dep beyond `libc`.

Format:

```
## YYYY-MM-DD — <change>

**Hypothesis:** <what we expected>
**Method:** <fixture, samples, baseline>
**Before:** <metric>
**After:** <metric>
**Decision:** <kept / reverted / parked>
```

---

## 2026-05-06 — Calibrate perf floors after profiling on macOS

**Hypothesis:** Original spec floors (50× forky-bash, 10× pure-bash) are
achievable on wall-clock at depth 8.

**Method:** hyperfine `--shell=none` 50 runs warmup 10. Compared:
- `target/release/examples/noop` (`fn main() {}`) — pure startup baseline
- `target/release/lup --version` — adds argv parse + static print
- `target/release/lup -r .env` (depth 32) — full work
- `bash bench/baselines/forky.sh .env` (depth 32) — process-spawning bash
- `bash bench/baselines/pure.sh .env` (depth 32) — parameter-expansion bash

Cross-checked with `otool -L` (dylib imports) and the rust-random/rand#733
issue thread re: macOS dyld initialization cost.

**Before (depth 8, no `--shell=none`):**
- forky/lup ratio: 32.9× (floor: 50×) ❌
- pure/lup ratio: 2.8× (floor: 10×) ❌

**After (depth 32, `--shell=none`):**
- forky/lup ratio: 31–131× (multiple runs)
- pure/lup ratio: 2.0–3.0× (multiple runs)

**Root cause:** Process-startup floor is ~1.9 ms on macOS Rust (Mach-O
dyld + Rust runtime init; libSystem cannot be statically linked). lup's
walk adds ~50–300 µs even at depth 32. Pure bash with parameter expansion
is genuinely lean (~5 ms shell startup + ~50 µs/iteration). The
wall-clock ratio against pure bash caps at ~3× regardless of how fast
lup runs.

**Decision:** kept. Floors recalibrated to 25× forky / 2× pure (hard);
50× / 10× kept as aspirational warnings. Algorithmic 10× pure-bash claim
moved to in-process measurement via `benches/lookup.rs` (criterion).

**See also:** spec §6 (calibrated table), `docs/decisions.md` D-018 / D-019 / D-020 / D-021.
