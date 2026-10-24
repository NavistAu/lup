#!/usr/bin/env bash
# Pure bash baseline: parameter expansion only, no subprocess per iteration.
# The realistic competitive baseline.
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <query>" >&2
    exit 2
fi
query="$1"

dir="$(pwd)"
home="${HOME:-/}"
while :; do
    if [[ -e "$dir/$query" ]]; then
        printf '%s\n' "$dir/$query"
        exit 0
    fi
    if [[ "$dir" == "$home" || "$dir" == "/" ]]; then
        break
    fi
    dir="${dir%/*}"
    [[ -z "$dir" ]] && dir="/"
done
echo "pure.sh: no match for '$query'" >&2
exit 1
