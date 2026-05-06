# LUP — Design Specification

**Date:** 2026-05-06
**Status:** Approved (brainstorming phase)
**Source brief:** original `INIT.md` (removed; this spec is the source of truth).

## 1. Purpose

`lup` is a Rust CLI binary that, given a literal path argument, walks up the
directory hierarchy from `$PWD` and reports any ancestor at which that path
exists. It is the reverse of a recursive descent: a "look UP" tool.

The motivating use case is shell integration of project-scoped configuration:

```sh
source $(lup .env)        # source the nearest .env walking up from $PWD
eval $(lup -e .env)       # eval the contents of the nearest .env
```

## 2. Goals & non-goals

### Primary goals

1. **Correctness** of the upward walk against the boundary semantics defined
   in §4.
2. **Speed** — `lup` must be measurably faster than the equivalent shell loop.
   See §6 for the enforced performance floors.
3. **Predictable shell integration** — exit codes, output format, and stream
   discipline (stdout = data, stderr = diagnostics) follow conventions a
   shell author can rely on.

### Non-goals (explicit, will not be added)

| Non-goal | Rationale |
|---|---|
| Windows support | Out of scope. POSIX-only design. |
| Color / TTY-aware output | Output is for shells and pipes. |

These are tracked in [`docs/wont-do.md`](../../wont-do.md).

### Roadmap (deferred but plausibly worth doing)

Tracked in [`docs/roadmap.md`](../../roadmap.md). Notable items: glob/regex
matching, multiple query arguments, `--canonicalize`, manpage generation,
Homebrew formula, GitHub Releases automation with prebuilt binaries, fuzzing.

## 3. CLI surface

### Synopsis

```
lup [OPTIONS] <QUERY>
lup --completions <bash|zsh|fish>
lup -h | --help | -V | --version
```

`<QUERY>` is exactly one literal path. It may be single-component (`.env`) or
multi-component (`.claude/settings.json`). At each candidate ancestor `A`,
`lup` checks whether `A/<QUERY>` exists.

### Flags

| Short | Long | Purpose |
|---|---|---|
| `-a` | `--all` | Print every hit, closest-first |
| `-e` | `--echo` | Print file contents instead of paths |
| `-g` | `--git` | Stop at git root; hard-error if no git repo |
| `-r` | `--root` | Walk all the way to filesystem `/` |
| `-f` | `--files` | Only match regular files |
| `-d` | `--dirs` | Only match directories |
| `-L` | `--follow` | Follow symlinks (default) |
| `-P` | `--no-follow` | Don't follow symlinks (match the link itself) |
|      | `--relative` | Print paths relative to PWD instead of absolute |
| `-0` | `--null` | Terminate each output record with `\0` (matches `find -print0` convention) |
|      | `--completions <SHELL>` | Print baked-in completion script |
| `-h` | `--help` | Print help to stdout |
| `-V` | `--version` | Print version to stdout |

### Mutex / interaction rules

- Boundary flags `-g` and `-r` are mutually exclusive. Default boundary
  (HOME-or-root, see §4) applies if neither is given.
- Type filters `-f` and `-d` are mutually exclusive. Neither given = no type
  filter (anything-that-exists matches).
- Symlink flags `-L` and `-P` are mutually exclusive. Default is `-L`
  (follow), to match the canonical case of a symlinked `.env`.
- `-e` requires the matched entry to resolve to a regular file. Directory or
  unreadable target → exit 4 with a stderr message.
- **Default mode (path output)**: each path is followed by `\n`, including
  the last (matches `println!` / `find -print` / `which` conventions). With
  `-0`, each path is followed by `\0` instead, including the last.
- **`-e` mode (content output, single hit)**: file bytes written verbatim,
  no trailing structure added by `lup`. Whatever bytes the file contains are
  what stdout receives.
- **`-a -e` mode**: file contents concatenated in walk order with **no
  separator**. The separator-free concatenation is intentional: many files
  end with their own newline, and `lup` will not fabricate structure. With
  `-a -e -0`, each file's contents are followed by `\0`, including the last.
- `--completions <SHELL>` is structurally identical to `--help`: print a
  static `&'static str` to stdout and exit 0. Unknown `<SHELL>` → exit 2.

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | At least one match found and emitted |
| 1 | No match within boundary |
| 2 | Usage error (bad flags, mutex violation, missing query, unknown completion shell) |
| 3 | Boundary error (`-g` outside any git repo) |
| 4 | I/O error during `-e` read (unreadable file, vanished mid-walk) |

### Stream discipline

- **stdout**: data only (paths in default mode, contents in `-e` mode, help
  text for `-h`/`--help`, completion scripts for `--completions`).
- **stderr**: diagnostics only (no-match messages, boundary errors, usage
  errors, I/O errors).

This guarantees `source $(lup .env)` and `eval $(lup -e .env)` see only
useful data on success and nothing on stdout on failure (so the shell's own
error message about missing-file or empty-eval becomes the user-visible
diagnostic).

### Argument parser

Hand-rolled, ~80 lines. Walks argv left-to-right, recognizes `--` as
end-of-options, takes the last positional as the query. No external crate.

## 4. Algorithm

### Boundary resolution (one-time, at startup)

```
boundary_path =
  if -g:   walk up from PWD looking for `.git` dir/file; if not found → exit 3
  elif -r: "/"
  else:    if PWD is HOME or under HOME → HOME ; else → "/"
```

Boundary is **inclusive**: the boundary directory itself is checked as a
candidate before the walk stops. If `PWD == boundary` at start, exactly
one probe runs (against PWD) and then the walk terminates.

The default branch's "PWD is HOME or under HOME" test is implemented as a
byte-prefix comparison on **normalized** PWD and HOME paths. *Normalized*
here means: trailing `/` removed (except for `/` itself), no `.` or `..`
components, no double slashes — i.e., the form returned by
`getcwd(2)`/`$HOME` already provides for sane environments. We do not
canonicalize symlinks; the comparison is lexical.

### The walk

```
1. dir_fd = openat(AT_FDCWD, ".", O_DIRECTORY|O_RDONLY)   # opens current dir
2. loop:
3.   probe(dir_fd, query)                # faccessat / fstatat depending on flags
4.   if hit:
5.     if -a: record and continue
6.     else:  emit, return 0
7.   if cwd_for_dir_fd == boundary_path: stop
8.   parent_fd = openat(dir_fd, "..", O_DIRECTORY|O_RDONLY)
9.   close(dir_fd); dir_fd = parent_fd
10. end loop
11. emit recorded hits (-a) or exit 1 (no hits)
```

### Probe variants

- **Default** (no `-f`/`-d`/`-P`): `faccessat(dir_fd, query_cstr, F_OK, 0)`.
  One syscall, kernel does not fill a stat buffer.
- **Type filter** (`-f` or `-d`): `fstatat(dir_fd, query_cstr, &mut stat, 0)`,
  check `S_ISREG` / `S_ISDIR`.
- **No-follow** (`-P`): `fstatat(..., AT_SYMLINK_NOFOLLOW)`. The symlink
  itself is the candidate; type filter (if any) applies to the link itself.

Multi-component queries (e.g. `.claude/settings.json`) are passed as a single
relative path to `faccessat`/`fstatat`; the kernel resolves components.

### Tracking absolute path during walk

The walk holds a `dir_fd`. The boundary comparison needs an absolute path. We
maintain a `Vec<u8>` representing the current absolute path and pop a
component each iteration, comparing against the (normalized) boundary path
byte-for-byte. We avoid per-iteration `fcntl(F_GETPATH)` /
`readlink("/proc/self/fd/N")` calls.

### Content read for `-e`

```
file_fd = openat(dir_fd, query, O_RDONLY)
size = fstat(file_fd).st_size
buf = Vec<u8>::with_capacity(size); read(file_fd, &mut buf) loop
write(STDOUT, &buf)               # or two writes for -a -e -0 with separator
```

No UTF-8 validation. Bytes in, bytes out.

### Path representation

`OsStr` / `&[u8]` / `CStr` throughout. No `String` or `PathBuf` for the
query, the boundary, or any candidate path. argv enters as `OsString`; we
convert to `CString` once per probe (or once total, if reused).

### Lib API surface (`lib.rs`)

```rust
pub struct Query<'a> {
    pub path: &'a [u8],
    pub all: bool,
    pub echo: bool,
    pub kind: Option<Kind>,         // Files | Dirs
    pub follow: bool,
    pub null_separator: bool,
    pub relative: bool,
}

pub enum Boundary { Home, Git, Root }

pub enum Hit {
    Path(Vec<u8>),
    Contents(Vec<u8>),
}

pub enum LupError {
    NoMatch,
    NotInGitRepo,
    UsageError(&'static str),
    Io(std::io::Error),
}

pub fn lookup(query: &Query, boundary: Boundary) -> Result<Vec<Hit>, LupError>;
```

The exact type shape may evolve during implementation; the principle is that
tests can drive the search function directly without spawning a subprocess.

## 5. Dependency policy

- `[dependencies]` (production): only `libc`. Justified by direct kernel
  invocation (`openat`, `faccessat`, `fstatat`, `read`) for measurable
  performance gains over `std::fs::Path` wrapping. Any further runtime dep
  requires a documented profile-evidence justification.
- `[build-dependencies]`: scrutinized. Anything compiled during a production
  build needs justification. v1 plan: none.
- `[dev-dependencies]`: unrestricted. Tests, benches, fixtures, and tooling
  may freely use `criterion`, `tempfile`, etc.

## 6. Performance plan

### Cargo release profile

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
overflow-checks = false
debug = false
```

Rationale: at typical depths (3–10 ancestors), process startup and
dynamic-linker work dominate the inner walk. LTO + single codegen unit
maximize linker freedom for inlining and dead-code stripping.
`panic = "abort"` removes unwinding tables and the panic runtime, reducing
binary size and cold-start overhead.

### Linkage

- **macOS**: link against system `libSystem` (no static option). The `libc`
  crate is bindings-only — it adds zero shared libraries to the load list.
- **Linux glibc**: dynamic, default.
- **Linux musl**: static target `x86_64-unknown-linux-musl`, in scope for
  v1. Produces a single self-contained binary.

### Startup-cost minimization

- No `format!` / `write!` on hot paths; `write_all(&[u8])` to stdout.
- No `Path::canonicalize`; no redundant `getcwd`.
- One `io::stdout().lock()` for the lifetime of the process; flush on Drop.
- No proc-macro crates; no large generic instantiation.

### Performance floors (enforced)

| Baseline | Floor | Aspirational |
|---|---|---|
| Forky bash (spawns `dirname` per iteration) | `lup_median × 50 ≤ baseline_median` | 100× |
| Pure bash (parameter expansion + `[[ -e ]]`) | `lup_median × 10 ≤ baseline_median` | 100× |

Both floors enforced by `tests/perf.rs`. Aspirational target produces a CI
warning when missed but does not fail the build.

### Profile-driven decision gate

Before adding any optimization beyond the plan above (threading,
`io_uring`, custom allocator, per-OS API paths beyond POSIX, etc.):

1. Capture a flame graph or `samply` profile of a realistic-startup run.
2. Confirm the targeted layer is >5% of total measured time.
3. Record before/after measurements in `docs/perf-decisions.md`.

This codifies the principle: if 99% of time is in kernel syscalls, threads
don't help.

### Things explicitly NOT done unless profile demands it

- Threading (walk is sequential, ~3–10 syscalls).
- `io_uring` (Linux-only, overhead exceeds work at this scale).
- mimalloc / jemalloc (we barely allocate).
- SIMD anything (no parsing on hot path).

## 7. Testing strategy

### Test layers

1. **Unit tests** (in-module `#[cfg(test)]`): pure logic — argv parsing,
   mutex rules, exit-code mapping, byte-prefix HOME-under-PWD detection.
2. **Integration tests** (`tests/walk.rs`, `tests/cli.rs`): tempdir
   fixtures via `tempfile` (dev-dep). Tests drive both `lib::lookup()`
   directly and the spawned binary via `std::process::Command`.
3. **Performance tests** (`tests/perf.rs`, release-only): hyperfine-driven
   floor assertions described in §6.

### Integration scenarios (tempdir fixtures)

- Single-component query: hit at PWD; hit one level up; hit at boundary;
  no hit anywhere.
- Multi-component query (`a/b/c`).
- `-a` ordering verified closest-first.
- `-g` outside a git repo → exit 3.
- `-g` inside a fixture-git-repo → finds repo root.
- PWD outside HOME-equivalent → walks to fixture-`/`.
- `-f` on a directory match → no hit. `-d` on a file match → no hit.
- `-L` vs `-P` on a symlink loop / dangling symlink.
- `-e` reads file contents to stdout, exit 0.
- `-e` on a directory → exit 4 with stderr message.
- `-a -e` concatenates contents in order.
- `-a -e -0` separates contents with NUL bytes.
- `--relative` produces `../../.env`-style output.
- Bad CLI (mutex violations, unknown flag, missing query) → exit 2.

### Coverage targets

Enforced via `cargo-llvm-cov` in CI:

- **Library code** (`src/lib.rs` and internal modules): ≥90% line coverage.
- **Whole-binary**: ≥85% line coverage.

Below-floor blocks merge.

### Bash baseline correctness

Bash baselines (`bench/baselines/forky.sh`, `bench/baselines/pure.sh`) are
first-class artifacts under test:

- `shellcheck` runs as a CI lint gate (zero warnings tolerated).
- `shellspec` specs in `bench/baselines/spec/` verify each baseline
  correctly finds a planted target in a fixture tree, fails on a missing
  target, and exits with the matching codes.

Without these, a broken baseline could silently inflate the speed ratio.

### Performance history

- `bench/history/<release-tag>.json` — one file per tagged release,
  containing hyperfine output for both baselines and `lup`.
- `bench/history/main.jsonl` — one line per CI run on `main`.
- A small comparison script reports per-release regression vs. the
  previous tag. >20% regression in either ratio is a release blocker;
  warn-only on `main` runs unless a floor breaks.

### CI matrix

- `macos-latest` (primary target).
- `ubuntu-latest` with both glibc and musl builds.

### Not tested

- Concurrent invocation (single-shot process, no shared state).
- Symlink races (TOCTOU is intrinsic to filesystem walking; no atomicity
  claim is made).
- Filesystem-specific oddities (resource forks, xattrs).

## 8. Project structure

```
lup/
├── Cargo.toml
├── Cargo.lock                   # committed (binary crate)
├── mise.toml                    # toolchain + dev tools
├── README.md
├── CLAUDE.md                    # rules summary for future Claude sessions
├── docs/
│   ├── architecture.md          # short overview pointing to spec
│   ├── decisions.md             # ADR log: each key decision + 1-line rationale + spec ref
│   ├── wont-do.md               # explicit non-goals (Windows, color)
│   ├── roadmap.md               # deferred items
│   ├── perf-decisions.md        # ongoing perf measurement log
│   └── superpowers/specs/2026-05-06-lup-design.md   # this spec
├── src/
│   ├── lib.rs                   # public API: Query, Boundary, Hit, lookup()
│   ├── main.rs                  # arg parsing → lib::lookup() → stdout/stderr/exit
│   ├── cli.rs                   # hand-rolled argv parser + help text consts
│   ├── walk.rs                  # the openat/faccessat/fstatat hot loop
│   ├── boundary.rs              # boundary resolution (HOME prefix, git-root probe)
│   ├── output.rs                # stdout writers: path mode, echo mode, NUL separator
│   └── completions/
│       ├── mod.rs               # `match shell { ... }` returning &'static str
│       ├── bash.txt             # hand-authored completion script
│       ├── zsh.txt
│       └── fish.txt
├── tests/
│   ├── walk.rs                  # integration tests against tempdir fixtures
│   ├── cli.rs                   # subprocess-based CLI/exit-code tests
│   └── perf.rs                  # hyperfine-driven floor assertions (release-only)
├── benches/
│   └── lookup.rs                # criterion-based in-process benchmark
├── bench/
│   ├── baselines/
│   │   ├── forky.sh             # process-spawning bash baseline
│   │   ├── pure.sh              # parameter-expansion bash baseline
│   │   └── spec/                # shellspec specs verifying baseline correctness
│   ├── fixtures/
│   │   └── build_tree.sh        # constructs the depth-8 test tree under $TMPDIR
│   └── history/
│       ├── main.jsonl           # rolling per-CI-run entries
│       └── <release-tag>.json   # one file per tagged release
└── .github/
    └── workflows/
        └── ci.yml
```

Notes:

- Single crate; lib + bin in one package via `[lib]` + `[[bin]]`. No
  workspace.
- Completion scripts are hand-authored static text under
  `src/completions/`, included via `include_str!`. No `build.rs` in v1.
- `benches/` uses `criterion` (dev-dep) for ad-hoc local tuning;
  `tests/perf.rs` is the *enforcing* perf test.

## 9. Build, CI, release

### Build

- `cargo build --release` for host target.
- `cargo build --release --target x86_64-unknown-linux-musl` on Linux for
  the static artifact.
- All build/dev tooling resolved via `mise install` from `mise.toml`.

### CI workflow (`.github/workflows/ci.yml`)

Single workflow, matrix over `macos-latest` and `ubuntu-latest`. Steps in
order, sequential within a job:

1. `mise install`
2. `cargo fmt --check`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `shellcheck bench/baselines/*.sh bench/fixtures/*.sh`
5. `shellspec bench/baselines/spec/`
6. `cargo nextest run --release`
7. `cargo llvm-cov --lcov --output-path coverage.lcov`, then assert
   `lib ≥ 90%` and `binary ≥ 85%` via a small parsing script.
8. `cargo test --release --test perf` (hyperfine-driven floor
   assertions; produces `bench/history/main.jsonl` row).
9. (Linux only) musl static build and a smoke test that runs the binary
   in a static-busybox container.

Regression vs. previous `main.jsonl` row of >20% in either ratio: warn in
CI summary, do not block. Floor breach: hard fail.

### Release process (manual, v1)

1. Bump version in `Cargo.toml`.
2. `cargo build --release` (and musl variant on Linux).
3. Tag `v0.1.0`. CI on the tag captures `bench/history/v0.1.0.json`.

No GitHub Releases automation, no Homebrew formula, no announcements — all
roadmapped.

### Versioning

SemVer 0.x while flag/exit-code surface settles. Promote to 1.0 when the
surface is frozen.

### Branching

`main` is the only long-lived branch. Topic branches for features. PRs
require CI pass.

## 10. Decisions captured (Q&A summary)

These are the answers that drove this design, recorded for future-session
context. The full discussion lives in the brainstorming transcript.

| # | Question | Decision |
|---|---|---|
| Q1 | What counts as a match? | Anything that exists; type and symlink filter flags refine. |
| Q2 | Is the boundary inclusive? | Yes. `~/.env` reachable when default boundary is `$HOME`. |
| Q3 | Boundary fallback when not applicable? | `-g` hard-errors if no git repo. Default boundary falls back to `/` if PWD is outside HOME. Explicit `-r` always walks to `/`. |
| Q4 | What happens on no match? | Exit 1, terse line on stderr, nothing on stdout. |
| Q5 | Output path format? | Absolute by default; `--relative` flag for cwd-relative. No `--canonicalize` in v1. |
| Q6 | Performance floors? | 50× vs forky-bash, 10× vs pure-bash, both enforced. 100× aspirational. Test against both baselines. |
| Q7 | v1 scope inclusions? | `-h`/`-V`, lib+bin split, baked-in completions, `-0`/`--null` IN. Manpage / Homebrew / GH releases / others to roadmap. |
| — | Runtime dependency policy? | `libc` only in `[dependencies]`. Dev-deps unrestricted. Build-deps scrutinized. |
| — | Platform support? | macOS + Linux (glibc + musl static). Windows is won't-do. |
| — | Color output? | Won't-do. |
| — | Test runner & coverage? | `cargo-nextest`; `cargo-llvm-cov` with 90%/85% floors. |
| — | Bash baseline verification? | `shellcheck` + `shellspec` first-class CI gates. |
| — | Performance harness? | `hyperfine` with logged history per release. |
| — | Toolchain enumeration? | `mise.toml`. |
