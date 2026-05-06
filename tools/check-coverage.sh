#!/usr/bin/env bash
# Read coverage.lcov and assert: lib lines ≥ 90%, whole binary lines ≥ 85%.
# Lib lines = any source file under src/ excluding src/main.rs.
set -euo pipefail

LCOV="${1:-coverage.lcov}"
if [ ! -f "$LCOV" ]; then
    echo "coverage file not found: $LCOV" >&2
    exit 2
fi

# Parse lcov: for each SF (source file) block, sum LH (lines hit) and LF (lines found).
awk '
    /^SF:/   { file=substr($0,4); lf=0; lh=0 }
    /^LF:/   { lf=substr($0,4) }
    /^LH:/   { lh=substr($0,4) }
    /^end_of_record$/ {
        total_lf += lf; total_lh += lh
        if (file !~ /src\/main\.rs$/ && file ~ /src\//) {
            lib_lf += lf; lib_lh += lh
        }
    }
    END {
        if (total_lf == 0) { print "no coverage data"; exit 2 }
        bin_pct = 100.0 * total_lh / total_lf
        lib_pct = (lib_lf > 0) ? 100.0 * lib_lh / lib_lf : 0
        printf "binary: %.1f%% (%d/%d)\n", bin_pct, total_lh, total_lf
        printf "lib:    %.1f%% (%d/%d)\n", lib_pct, lib_lh, lib_lf
        fail = 0
        if (lib_pct < 90.0) { print "FAIL: lib coverage <90%" > "/dev/stderr"; fail=1 }
        if (bin_pct < 85.0) { print "FAIL: binary coverage <85%" > "/dev/stderr"; fail=1 }
        exit fail
    }
' "$LCOV"
