# Design decisions

Append-only log of key decisions. Each entry: ID, summary, 1-line rationale,
spec section reference.

| ID | Decision | Rationale | Spec ref |
|---|---|---|---|
| D-001 | Match = anything-that-exists | Examples mix files and directories; one rule fits both | §3 (Q1) |
| D-002 | Boundary inclusive | `~/.env` must be reachable for `source $(lup .env)` | §4 (Q2) |
| D-003 | Default boundary falls back to / when PWD outside HOME | Search tool should still search even from /etc | §4 (Q3) |
| D-004 | `-g` hard-errors when not in git repo | Explicit user intent should not silently fall back | §4 (Q3) |
| D-005 | No-match: exit 1, terse stderr | Standard tool convention (which/find/grep) | §3 (Q4) |
| D-006 | Default output is absolute path | Unambiguous; no extra syscall over what we already have | §3 (Q5) |
| D-007 | `--relative` flag is opt-in | Most use cases want absolute; relative needs computation | §3 (Q5) |
| D-008 | `-0` is NUL-terminator (not separator), per `find -print0` | Consistency with established convention | §3 |
| D-009 | Hand-rolled argv parser; only `libc` in `[dependencies]` | Speed is primary goal; clap costs build/binary size and adds nothing for our flag set | §5 |
| D-010 | Direct `libc` syscalls on hot path | `Path` wrapping costs allocations; no portability gain we use | §4 |
| D-011 | Won't-do: Windows | POSIX-only design; `openat` semantics not equivalent | §2 |
| D-012 | Won't-do: color output | Output is for shell pipes; color creates parsing hazards | §2 |
| D-013 | Lib + bin split with public modules | Tests can drive `lookup()` directly without subprocess overhead | §4 lib API |
| D-014 | Completions as hand-authored static strings, no `build.rs` | No build-deps, no codegen complexity, full control | §3 / §8 |
| D-015 | musl static Linux build is v1 deliverable | Single-file shippable binary; minimum cold start | §6 linkage |
| D-016 | Coverage floors: 90% lib, 85% binary | Catches dead branches without forcing 100% on glue code | §7 |
| D-017 | Bash baselines verified by shellcheck + shellspec | A broken baseline silently inflates speed claims | §7 |
| D-018 | Perf floors recalibrated to 25× forky / 2× pure (hard); 50× / 10× kept as aspirational | Profiling on macOS showed ~1.9 ms startup floor + ~50 µs walk; pure-bash with parameter expansion at any reasonable depth caps at ~3× wall-clock; the original 10× pure-bash floor is achievable only in-process (criterion bench), not on wall-clock | §6 |
| D-019 | Fixture depth: 32 (was 8) for `tests/perf.rs` | At depth 8 process-startup dominates; at depth 32 the bash loops dominate, exposing the algorithmic ratio | §6 |
| D-020 | Hyperfine `--shell=none` | Sub-5 ms measurements need it (hyperfine warns); avoids `/bin/sh -c` wrapping noise | §7 |
| D-021 | Criterion in-process bench (`benches/lookup.rs`) for walk-cost-only measurement | Wall-clock can't isolate walk from startup; criterion gives microsecond-precision per iteration | §6 |
| D-022 | `LupError::UsageError(String)` instead of spec's `UsageError(&'static str)` | Lets the parser report which flag was wrong by formatting the offending bytes (e.g. "unknown short flag: -x"); the spec's `&'static str` constraint precluded this. Public API change vs spec §4. | §4 |
| D-023 | GitHub Releases automation moved out of roadmap into v0.1.0 (`.github/workflows/release.yml`) | Triggered on `v*` tag push or manual dispatch. Builds macOS arm64 + Linux x86_64 musl binaries with sha256 sums, publishes via `gh release create --generate-notes`. Other targets (Intel macOS, aarch64 Linux, glibc Linux) deferred to roadmap until demand is shown. | §9 |
