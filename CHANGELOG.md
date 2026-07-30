# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Because `lup` searches directories above the caller, any change to **where it
looks**, **what it matches**, **where it stops**, or **how it follows symlinks**
is called out explicitly whatever its size. Scripts depend on those four
behaviours for both correctness and safety — see [`SECURITY.md`](SECURITY.md).

## [Unreleased]

Nothing yet.

## [0.1.0]

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

[Unreleased]: https://github.com/NavistAu/lup/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/NavistAu/lup/releases/tag/v0.1.0
