# shellcheck shell=bash
set -eu

setup_fixture() {
    local depth="$1"
    local plant_at="$2"  # 0 = at deepest, depth = at top
    local query="$3"

    FIXTURE_ROOT="$(mktemp -d)"
    local current="$FIXTURE_ROOT"
    local i
    for i in $(seq 1 "$depth"); do
        current="$current/d$i"
        mkdir -p "$current"
    done

    local plant_dir="$FIXTURE_ROOT"
    if [ "$plant_at" -gt 0 ]; then
        for i in $(seq 1 "$plant_at"); do
            plant_dir="$plant_dir/d$i"
        done
    fi
    echo "fixture-content" > "$plant_dir/$query"

    FIXTURE_PWD="$current"
    FIXTURE_PLANT="$plant_dir/$query"
}

teardown_fixture() {
    if [ -n "${FIXTURE_ROOT:-}" ]; then
        rm -rf "$FIXTURE_ROOT"
    fi
}
