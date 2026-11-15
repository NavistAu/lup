# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Because `lup` searches directories above the caller, any change to **where it
looks**, **what it matches**, **where it stops**, or **how it follows symlinks**
is called out explicitly whatever its size. Scripts depend on those four
behaviours for both correctness and safety — see [`SECURITY.md`](SECURITY.md).

## [Unreleased]

## [0.3.0] - 2026-10-24

The `lup` command is unchanged; library callers need to update.

### Changed

- **Breaking (library):** the package is split into `liblup`, the walk other
  programs depend on, and `lup-cli`, which builds the `lup` command. The crate
  name `lup` is taken on crates.io by an unrelated project. `Query` no longer
  carries `null_terminate` or `relative`; the command's output code owns them.

### Added

- `liblup` and `lup-cli` are published to crates.io on release, through
  trusted publishing, so the repository holds no registry secret.
  `cargo install lup-cli` installs the command.

## [0.2.0] - 2026-10-23

The `lup` command is unchanged; library callers need to update.

### Changed

- **Breaking (library):** `lookup()` takes the start directory, and
  `Boundary::Home` carries the home directory. A caller can walk from any
  directory without changing the process's working directory. `start` must be
  absolute and physical, as `getcwd` returns it. Only the command reads the
  working directory and `$HOME`.
- Release builds follow the org binary-tool convention: per-target
  `lup-vX.Y.Z-<target>.tar.gz` assets, each with a `.sha256` companion, and the
  Homebrew formula is regenerated on release.

## [0.1.0] - 2026-05-06

Initial release.

### Added

- `lup <QUERY>` walks up from the working directory and prints the nearest
  ancestor location where `<QUERY>` exists. Multi-component queries work, for
  example `lup .claude/settings.json`.
- Boundary control: `-g`/`--git` stops at the git root and errors if not in a
  repository; `-r`/`--root` walks to the filesystem root.
- Match filtering: `-f`/`--files` matches regular files only, `-d`/`--dirs`
  matches directories only.
- Symlink control: `-L`/`--follow` (the default) and `-P`/`--no-follow`.
- Output control: `-a`/`--all` prints every hit closest-first, `-e`/`--echo`
  prints file contents instead of paths, `--relative` prints paths relative to
  the working directory, and `-0`/`--null` terminates records with NUL.
- Shell completions for bash, zsh and fish via `--completions <SHELL>`.
- Distinct exit codes: `0` match found, `1` no match within the boundary, `2`
  usage error, `3` `-g` given outside a git repository, `4` I/O error during an
  `-e` read.
- Pre-built binaries for macOS (Apple Silicon) and Linux (x86_64 static musl),
  plus a Homebrew tap.

[Unreleased]: https://github.com/NavistAu/lup/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/NavistAu/lup/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/NavistAu/lup/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/NavistAu/lup/releases/tag/v0.1.0
