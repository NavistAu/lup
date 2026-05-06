# Won't-do

These are explicit non-goals. They are not roadmap items. Adding them
requires changing the design.

## Windows support

`lup` is built directly on POSIX `openat` / `faccessat` / `fstatat`
semantics. Windows path resolution and file-handle semantics are
sufficiently different that supporting both would require a parallel
implementation. The user has no Windows target and no plans for one.

## Color output

`lup` writes to stdout for consumption by shells (`source $(lup .env)`).
Color escape codes injected into stdout would break those use cases. Color
is also a parsing hazard for any tool that captures `lup`'s output. There is
no version of `lup` that emits color, even with `--color`.
