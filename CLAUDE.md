# Project rules for `lup`

This file codifies non-negotiables for future Claude sessions.

## Dependency policy

- **`[dependencies]`**: only `libc`. Any further runtime dep requires a recorded
  profile-evidence justification in `docs/perf-decisions.md`.
- **`[build-dependencies]`**: scrutinized. None planned for v1.
- **`[dev-dependencies]`**: unrestricted (`tempfile`, `criterion`, `serde_json`,
  etc. are all fine).

## Performance floors

Hard floors enforced by `tests/perf.rs` (depth-32 fixture, hyperfine `--shell=none`):
- `lup_median × 25 ≤ forky-bash_median`  (observed: 30–130×)
- `lup_median × 2 ≤ pure-bash_median`    (observed: 2.5–3×)

Aspirational targets (logged as warnings, do not fail the build):
- 50× forky-bash, 10× pure-bash

The original 50× / 10× floor in the spec assumed pure bash would be much slower
than measured. After profiling on macOS Apple Silicon we found that both lup and
bash pay similar process-startup floors (~2 ms for Rust, ~5 ms for bash), so the
wall-clock ratio against pure bash caps at ~3× regardless of lup optimization.
The aggressive 10× pure-bash claim is true *for the algorithmic walk loop only*
and is measured by `benches/lookup.rs` (criterion, in-process).

**Do not merge a regression below the hard floors.**

## Platform support

- macOS (primary, Apple Silicon)
- Linux glibc
- Linux musl static (`x86_64-unknown-linux-musl`)
- **Never** Windows. See [`docs/wont-do.md`](docs/wont-do.md).

## Won't-do list

See [`docs/wont-do.md`](docs/wont-do.md). Two items: Windows, color output.
Anything else that's "deferred" lives in [`docs/roadmap.md`](docs/roadmap.md).

## Tooling

`mise.toml` is canonical. Tools in use:
- `cargo-nextest` (test runner)
- `cargo-llvm-cov` (coverage; floors 90% lib / 85% binary)
- `hyperfine` (perf measurement)
- `shellcheck` + `shellspec` (bash baseline lint and tests)

`samply` (profiling) is dev-host only — install via `cargo install samply`
(mise's cargo backend currently fails to resolve toolchain for it).

## Source of truth

The design specification at
[`docs/superpowers/specs/2026-05-06-lup-design.md`](docs/superpowers/specs/2026-05-06-lup-design.md)
is the design source of truth. When in doubt, read it before changing
behavior. Implementation differences from the spec require a decision-log
entry in `docs/decisions.md`.
