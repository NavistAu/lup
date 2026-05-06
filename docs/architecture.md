# Architecture

`lup` walks **up** from `$PWD`, asking the kernel at each ancestor whether the
literal query path exists. The walk uses an open file descriptor (`openat`)
plus relative-path probes (`faccessat` / `fstatat`), avoiding repeated
path-string concatenation kernel-side.

The hot path:

```
dir_fd = openat(AT_FDCWD, ".", O_DIRECTORY)
loop:
    matched? = faccessat(dir_fd, query, F_OK)   # or fstatat for type filtering
    if matched: emit / record
    if at_boundary: break
    parent_fd = openat(dir_fd, "..", O_DIRECTORY)
    close(dir_fd); dir_fd = parent_fd            # via OwnedFd guard
```

Boundary resolution and result formatting are off the hot path.

The full design (CLI surface, exit codes, lib API, performance plan, testing
strategy) lives in
[`superpowers/specs/2026-05-06-lup-design.md`](superpowers/specs/2026-05-06-lup-design.md).

## Module layout

| Module | Responsibility |
|---|---|
| `walk` | The search loop, type definitions (`Query`, `Boundary`, `Hit`, `KindFilter`, `LupError`), `OwnedFd` RAII guard, content read for `-e` |
| `boundary` | Default HOME-or-/ resolution; git-root probe |
| `output` | Stdout writers: paths, contents, NUL-termination, relative paths |
| `cli` | Hand-rolled argv parser, `ParsedArgs`, mutex validation, `HELP` and `VERSION` constants |
| `completions` | Hand-authored bash/zsh/fish completion scripts (static `&'static str` via `include_str!`) |
| `main` | argv → cli::parse → walk::lookup → output::write_hits → ExitCode |

## Performance characteristics

Measured on macOS Apple Silicon at fixture depth 32:

| Phase | Cost |
|---|---|
| Process spawn (fork + exec) | ~0.2 ms |
| macOS dyld phase (libSystem load, relocations) | ~1 ms |
| Rust runtime init (TLS, panic handlers) | ~0.5 ms |
| Argv parse + boundary resolution + walk + output | ~50–300 µs |

The walk loop itself is ~8 µs per ancestor, near the kernel's syscall floor for
`faccessat`. Wall-clock subprocess timings are dominated by the ~1.5–1.9 ms
process-startup floor that any dynamically-linked Rust binary pays on macOS.

## Linkage

- **macOS**: dynamic link against `libSystem.B.dylib`. No static-link option;
  Apple does not support static linking of libSystem.
- **Linux glibc**: dynamic link against `libc.so.6`.
- **Linux musl**: full static binary via `x86_64-unknown-linux-musl` target.
  Single-file deployable; no runtime dependencies.
