#!/usr/bin/env bash
# Forky bash baseline: spawns dirname per iteration. Slowest realistic baseline.
# Mirrors lup default semantics: walk up from PWD, stop at HOME (inclusive),
# print absolute path of first hit, exit 0; or exit 1 with stderr on no match.
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <query>" >&2
    exit 2
fi
query="$1"

dir="$(pwd)"
home="${HOME:-/}"
while :; do
    if [ -e "$dir/$query" ]; then
        printf '%s\n' "$dir/$query"
        exit 0
    fi
    if [ "$dir" = "$home" ] || [ "$dir" = "/" ]; then
        break
    fi
    dir="$(dirname "$dir")"
done
echo "forky.sh: no match for '$query'" >&2
exit 1
