#!/usr/bin/env bash
# bench-history.sh — Run `cyrius bench` and append the results to benches/history.csv.
#
# `cyrius bench` auto-discovers benches/*.bcyr and reports one line per benchmark:
#   dict english construction: 7.547ms avg (min=7.547ms max=7.547ms) [10 iters]
#   dict lookup hit: 130ns avg (min=117ns max=250ns) [200000 iters]
# The name is everything before the ": <avg>" and the avg is normalised to
# microseconds for the avg_us column.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

HISTORY_FILE="benches/history.csv"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_REV=$(git rev-parse --short HEAD 2>/dev/null || echo "uncommitted")

# Create header if file doesn't exist
if [ ! -f "$HISTORY_FILE" ]; then
    echo "timestamp,git_rev,benchmark,avg_us" > "$HISTORY_FILE"
fi

BENCH_OUT=$(mktemp)
trap 'rm -f "$BENCH_OUT"' EXIT

echo "Running cyrius bench (benches/*.bcyr)…"
if ! cyrius bench > "$BENCH_OUT" 2>&1; then
    cat "$BENCH_OUT" >&2
    echo "bench-history: 'cyrius bench' failed" >&2
    exit 1
fi

# Parse "<name>: <num><unit> avg …" into CSV rows, normalising <unit> to µs.
ROWS=$(awk -v ts="$TIMESTAMP" -v rev="$GIT_REV" '
    # RLENGTH spans ": " + <num><unit> + " avg", so the value is RLENGTH-6 bytes in.
    match($0, /: [0-9.]+[^ ]+ avg/) {
        name = substr($0, 1, RSTART - 1)
        val  = substr($0, RSTART + 2, RLENGTH - 6)

        sub(/^[[:space:]]+/, "", name)
        sub(/[[:space:]]+$/, "", name)

        num = val; sub(/[^0-9.].*$/, "", num)
        unit = val; sub(/^[0-9.]+/, "", unit)

        if      (unit == "ns") scale = 0.001
        else if (unit == "ms") scale = 1000
        else if (unit == "s")  scale = 1000000
        # "us" and "µs" alike; the octal form matches µ byte-wise in a C locale.
        else if (unit == "us" || unit == "µs" || unit == "\302\265s") scale = 1
        else next   # unknown unit — not a benchmark line

        if (name == "" || num == "" || num == ".") next

        # Fixed notation (never exponent), then trim the padding zeros.
        avg = sprintf("%.6f", num * scale)
        sub(/0+$/, "", avg)
        sub(/\.$/, "", avg)

        if (name ~ /[",]/) { gsub(/"/, "\"\"", name); name = "\"" name "\"" }
        printf "%s,%s,%s,%s\n", ts, rev, name, avg
    }
' "$BENCH_OUT")

if [ -z "$ROWS" ]; then
    cat "$BENCH_OUT" >&2
    echo "bench-history: no benchmark results parsed from 'cyrius bench' output" >&2
    exit 1
fi

printf '%s\n' "$ROWS" >> "$HISTORY_FILE"

echo "Recorded $(printf '%s\n' "$ROWS" | wc -l) benchmark(s) to $HISTORY_FILE"
column -t -s, "$HISTORY_FILE"
