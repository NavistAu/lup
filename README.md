# lup

Walk up the directory hierarchy looking for a path. A fast Rust CLI that, given a literal path, finds any ancestor of `$PWD` at which that path exists.

## Examples

```sh
lup .git                          # nearest .git directory above PWD
lup .claude/settings.json         # nearest project Claude config
lup -g .env                       # nearest .env, but only inside the current git repo
lup -a .env                       # all .env files, closest-first
source $(lup .env)                # source the nearest .env
eval $(lup -e .env)               # eval the contents of the nearest .env
```

## Install

```sh
cargo install --path .
```

A static-musl Linux build is supported; see [`docs/architecture.md`](docs/architecture.md) and [`docs/roadmap.md`](docs/roadmap.md).

## Development

This project pins all tooling via [mise](https://mise.jdx.dev/):

```sh
mise install
cargo nextest run                          # unit + integration tests
cargo nextest run --release --test perf    # hyperfine-driven perf floors
cargo bench --bench lookup                 # criterion in-process bench
shellcheck bench/baselines/*.sh bench/fixtures/*.sh tools/*.sh
```

See:
- [`docs/architecture.md`](docs/architecture.md) — how the walk works
- [`docs/decisions.md`](docs/decisions.md) — design decisions log
- [`docs/wont-do.md`](docs/wont-do.md) — explicit non-goals
- [`docs/roadmap.md`](docs/roadmap.md) — deferred work
- [`docs/perf-decisions.md`](docs/perf-decisions.md) — measured perf changes
