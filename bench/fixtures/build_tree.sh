#!/usr/bin/env bash
# Build a depth-N directory tree at $1 with the .env planted at the top.
# Prints the deepest directory path on stdout (use as PWD for the search).
set -euo pipefail

if [ "$#" -lt 1 ]; then
    echo "usage: $0 <root> [depth]" >&2
    exit 2
fi
root="$1"
depth="${2:-8}"

mkdir -p "$root"
echo "FIXTURE_ENV=1" > "$root/.env"

current="$root"
for i in $(seq 1 "$depth"); do
    current="$current/d$i"
done
mkdir -p "$current"
printf '%s\n' "$current"
