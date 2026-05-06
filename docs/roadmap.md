# Roadmap

Items deliberately deferred from v1. None are committed; each requires
re-evaluation if/when they're picked up.

## Distribution

- Manpage generation (`scdoc` or hand-authored).
- Homebrew formula (will live at `NavistAu/homebrew-tap`).
- Additional release targets: macOS x86_64 (Intel), Linux aarch64 (musl + glibc), Linux x86_64 glibc.
  v0.1.0 ships macOS arm64 and Linux x86_64 musl only; the rest add CI cost and demand has not been demonstrated.

## Functionality

- Glob/regex matching for the query (currently literal-only).
- Multiple positional queries (`lup .env .env.local` — find any/all).
- `--canonicalize` flag to resolve symlinks in output.
- Per-component depth limits (`--max-depth`).

## Tooling

- `cargo-fuzz` corpus for the argv parser.
- Build-time completion generation from CLI metadata (avoids hand-authored
  drift, but introduces a build-dep — pending evaluation).
- Performance regression warning vs previous CI run. The spec (§9) calls
  for warning in the CI summary when either ratio regresses >20% vs the most
  recent `bench/history/main.jsonl` row. The floor assertions still catch
  catastrophic regressions; the warning is a finer-grained signal. Implementing
  it requires CI artifact-persistence (cache-action or branch-commit) plus a
  comparison script.
- `mise`-based install of `samply` (currently `cargo install samply` direct).
