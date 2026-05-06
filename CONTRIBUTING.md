# Contributing to `lup`

## Setup

`lup` pins all dev tooling via [mise](https://mise.jdx.dev/):

```sh
mise install
```

This installs Rust (latest stable), `cargo-nextest`, `cargo-llvm-cov`, `hyperfine`, `shellcheck`, and `shellspec`. `samply` for profiling is a separate install: `cargo install samply`.

## Development loop

```sh
cargo nextest run                          # unit + integration tests
cargo nextest run --release --test perf    # hyperfine-driven perf floors (release-only)
cargo bench --bench lookup                 # criterion in-process bench
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
shellcheck bench/baselines/*.sh bench/fixtures/*.sh tools/*.sh
```

Coverage:

```sh
cargo llvm-cov nextest --lcov --output-path coverage.lcov
tools/check-coverage.sh coverage.lcov
```

Floors enforced by `tools/check-coverage.sh`: lib ≥ 90%, binary ≥ 85%.

## Building a static Linux binary locally

```sh
rustup target add x86_64-unknown-linux-musl
sudo apt-get install -y musl-tools     # or your distro's equivalent
cargo build --release --target x86_64-unknown-linux-musl
```

macOS hosts can't produce musl binaries directly; CI does that on Linux runners.

## Project documents

- [`docs/architecture.md`](docs/architecture.md) — module layout and the walk algorithm.
- [`docs/decisions.md`](docs/decisions.md) — append-only design decisions log.
- [`docs/wont-do.md`](docs/wont-do.md) — explicit non-goals.
- [`docs/roadmap.md`](docs/roadmap.md) — deferred work.
- [`docs/perf-decisions.md`](docs/perf-decisions.md) — measured perf changes with before/after.
- [`docs/superpowers/specs/2026-05-06-lup-design.md`](docs/superpowers/specs/2026-05-06-lup-design.md) — the design specification (source of truth).
- [`CLAUDE.md`](CLAUDE.md) — non-negotiables for AI-assisted contributions (dependency policy, perf floors, won't-do list, tooling).

## Performance floors

Hard floors (test fails if missed; depth-32 fixture, hyperfine `--shell=none`):

- `lup_median × 25 ≤ forky-bash_median` (observed: 30–130×)
- `lup_median × 2 ≤ pure-bash_median` (observed: 2.5–3×)

Aspirational targets (logged as warnings, do not fail the build):

- 50× forky-bash, 10× pure-bash

The wall-clock 10× pure-bash claim is achievable only in-process and is exercised by `benches/lookup.rs` (criterion). See [`docs/perf-decisions.md`](docs/perf-decisions.md) for the full derivation.

**Do not merge a regression below the hard floors.**

## Dependency policy

- `[dependencies]`: only `libc`. Any further runtime dep requires recorded profile evidence in `docs/perf-decisions.md`.
- `[build-dependencies]`: scrutinized. None planned for v1.
- `[dev-dependencies]`: unrestricted (`tempfile`, `criterion`, `serde_json`, etc. are all fine).

## Pull request checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo nextest run --release` passes
- [ ] `cargo nextest run --release --test perf` still meets the hard floors
- [ ] Coverage floors still met
- [ ] If the change shifts public API or behavior, a `docs/decisions.md` entry has been added
- [ ] If the change is a perf optimization, a `docs/perf-decisions.md` entry with before/after has been added
