# lup

- Spec (design source of truth): `docs/superpowers/specs/2026-05-06-lup-design.md`. Deviations recorded in
  `docs/decisions.md`.
- Runtime `[dependencies]`: `libc` only; any addition is justified by profile evidence in `docs/perf-decisions.md`.
  No `[build-dependencies]` in v1. Dev-deps unrestricted.
- Perf floors, enforced by `tests/perf.rs` (depth-32 fixture, hyperfine `--shell=none`):
  `lup_median × 25 ≤ forky-bash_median`, `lup_median × 2 ≤ pure-bash_median`. Aspirational 50× / 10× are warnings
  only. The in-process walk loop is measured by `benches/lookup.rs` (criterion). Rationale: `docs/perf-decisions.md`.
- Platforms: macOS (primary, Apple Silicon), Linux glibc, Linux musl static (`x86_64-unknown-linux-musl`).
- Won't-do: `docs/wont-do.md` (Windows, color output). Deferred items: `docs/roadmap.md`.
- Tooling (`mise.toml`): `cargo-nextest`, `cargo-llvm-cov` (floors 90% lib / 85% binary), `hyperfine`, `shellcheck` +
  `shellspec` (bash baseline). `samply` is dev-host only via `cargo install samply` (mise's cargo backend fails for it).
