#!/bin/sh
# Run a crate's tests and record the results as evidence in its store.
# Usage: scripts/dogfood.sh [crate]   (from the repository root; default mitra-lang)
#
# The revision is the git hash of the crate's code (src/, tests/,
# Cargo.toml), not of the commit: committing new evidence does not make it
# stale, and only a change to the code does. The code must be committed,
# so the hash names what was tested.
set -eu
crate=${1:-mitra-lang}
code=""
for p in src tests Cargo.toml; do
    [ -e "crates/$crate/$p" ] && code="$code crates/$crate/$p"
done
if ! git diff --quiet HEAD -- $code; then
    echo "dogfood: commit the changes to $crate first: the evidence names the committed code" >&2
    exit 1
fi
rev=$(git ls-tree HEAD $code | git hash-object --stdin | cut -c1-12)
report=$(mktemp)
trap 'rm -f "$report"' EXIT
cargo build -q -p mitra-cli
if ! cargo test -p "$crate" --no-run -q 2>/dev/null; then
    echo "dogfood: $crate's tests do not build" >&2
    exit 1
fi
status=0
# The tests that compare the stores with their expected verdicts skip
# themselves: this store's evidence is about to change.
export MITRA_BREAKS=1
cargo test -p "$crate" -- --test-threads=1 > "$report" 2>&1 || status=1
if ! grep -q '^test .* \.\.\. ' "$report"; then
    echo "dogfood: no test results in the report" >&2
    exit 1
fi
cd "crates/$crate"
../../target/debug/mitra ingest "$report" --rev "$rev"
# A model kept aside: what the reader of the code finds, as evidence.
if grep -rqs 'mitra: implements ' src tests; then
    ../../target/debug/mitra tie --rev "$rev"
fi
# Only the evidence from this code: an earlier pass does not hide a failure now.
../../target/debug/mitra eval --rev "$rev"
exit $status
