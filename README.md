# lup

**Look UP the directory hierarchy for a file.**

`lup` walks up from your current directory and finds the nearest place where a given path exists. It's the answer to "where's the closest `.env` / `.git` / `package.json` / `tsconfig.json` above me?".

```sh
lup .env                          # → /Users/you/projects/foo/.env
lup .git                          # → /Users/you/projects/foo/.git
lup .claude/settings.json         # multi-component paths work too
```

Designed for use in shell scripts and dotfiles:

```sh
source $(lup .env)                # source the nearest .env
eval $(lup -e .env)               # eval its contents directly
cd $(dirname $(lup .git))         # cd to the repo root
```

It is fast — at depth 32 it runs ~120× faster than the equivalent process-spawning bash loop (`dirname` + `[ -e ]`), and ~3× faster than the leanest pure-bash equivalent. Most of `lup`'s ~2 ms wall-clock time is unavoidable Rust-on-macOS process startup; the actual walk loop is ~50–300 µs.

## Install

### Homebrew

```sh
brew install navistau/tap/lup
```

### Pre-built binaries

Download from the [latest release](https://github.com/NavistAu/lup/releases/latest) for your platform:

- macOS (Apple Silicon): `lup-macos-arm64`
- Linux (x86_64, static musl): `lup-linux-x86_64-musl`

Make it executable and put it on your `$PATH`:

```sh
chmod +x lup-*
sudo mv lup-* /usr/local/bin/lup
```

### From source

Requires Rust 1.74 or newer:

```sh
cargo install --git https://github.com/NavistAu/lup
```

Or clone and `cargo install --path .`.

## Usage

```
lup [OPTIONS] <QUERY>
```

`<QUERY>` is a literal path (not a glob). It can be a single name (`.env`) or a multi-component relative path (`.claude/settings.json`).

By default `lup`:

- starts at `$PWD` and checks each ancestor in turn,
- stops at `$HOME` if you're underneath it, or at `/` otherwise,
- prints the absolute path of the **first hit**, or
- exits 1 with a terse stderr message if there's no match within the boundary.

### Options

| Flag | Effect |
|---|---|
| `-a`, `--all` | Print every hit, closest-first |
| `-e`, `--echo` | Print the **contents** of the matched file (useful for `eval $(lup -e ...)`) |
| `-g`, `--git` | Stop at the git repo root; error 3 if you're not in a repo |
| `-r`, `--root` | Walk all the way to `/` instead of stopping at `$HOME` |
| `-f`, `--files` | Match only regular files |
| `-d`, `--dirs` | Match only directories |
| `-L`, `--follow` | Follow symlinks (default) |
| `-P`, `--no-follow` | Don't follow symlinks |
|     | `--relative` | Print paths relative to PWD instead of absolute |
| `-0`, `--null` | Terminate each output record with `\0` (matches `find -print0`) |
|     | `--completions <SHELL>` | Print a completion script for `bash`, `zsh`, or `fish` |
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

### Examples

```sh
# Source the nearest .env file
source $(lup .env)

# Evaluate its contents directly without sourcing a file
eval $(lup -e .env)

# Find every config file walking up — useful for layering
lup -a .npmrc

# Restrict to inside the current git repo
lup -g .env

# Find a repo root (its .git can be a directory or a worktree file)
lup .git

# Use the matched path relative to where you are
lup --relative .git
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Match(es) found |
| 1 | No match within boundary |
| 2 | Usage error (bad flags, mutex violation, missing query) |
| 3 | `-g` requested but you're not inside a git repository |
| 4 | I/O error during `-e` read |

### Stream discipline

- **stdout**: data only — paths in default mode, file bytes in `-e` mode, help text for `-h`, completion script for `--completions`.
- **stderr**: diagnostics only.

This means `source $(lup .env)` works correctly: on a hit it sources the right file, on a miss the shell sees an empty argument and reports its own error rather than choking on a stderr leak.

## Shell completions

`lup` ships with completion scripts embedded in the binary. To enable them:

```sh
# bash
lup --completions bash > ~/.local/share/bash-completion/completions/lup

# zsh — somewhere in $fpath
lup --completions zsh > ~/.zsh/completions/_lup

# fish
lup --completions fish > ~/.config/fish/completions/lup.fish
```

## Compatibility

- **macOS** (Apple Silicon and Intel)
- **Linux** (x86_64, glibc and musl)
- **Other Unixes** — likely fine, untested. Open an issue.
- **Windows** — not supported and not planned. `lup` is built directly on POSIX `openat` / `faccessat` / `fstatat`.

## How it works (briefly)

`lup` opens `$PWD` as a directory file descriptor, then walks up using `openat(.., "..")` while probing each ancestor with `faccessat` (or `fstatat` when type filters are active). All queries go through the kernel; there's no per-iteration string concatenation in user space. The result is that the walk loop costs ~8 µs per ancestor — close to the kernel's syscall floor.

For deeper detail see [`docs/architecture.md`](docs/architecture.md).

## Development

If you want to hack on `lup`, see [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

Dual-licensed under either of:

- MIT — see [`LICENSE-MIT`](LICENSE-MIT)
- Apache 2.0 — see [`LICENSE-APACHE`](LICENSE-APACHE)

at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `lup` shall be dual-licensed as above, without any additional terms or conditions.
